use std::{
    cell::UnsafeCell,
    collections::HashMap,
    hash::{BuildHasher, Hash, Hasher},
    sync::{Arc, LazyLock, RwLock, Weak},
};

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo,
        PrimaryCommandBufferAbstract,
    },
    descriptor_set::{
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerCreateInfo},
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageCreateInfo, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    shader::ShaderStages,
    sync::GpuFuture,
};

use crate::graphics::{pipeline::PipelineBuilder, Graphics};

use super::{Bindable, CommandBufferBuilder};

pub struct Texture {
    image_view: Arc<ImageView>,
    layout: Arc<DescriptorSetLayout>,
    descriptor_set: Arc<PersistentDescriptorSet>,
}

#[derive(Hash, PartialEq, Eq)]
struct LayoutCreateArgs {
    pub shader_strages: ShaderStages,
    pub min_filter: Filter,
    pub mag_filter: Filter,
}

impl Texture {
    pub fn extent(&self) -> [u32; 3] {
        self.image_view.image().extent()
    }

    pub fn new(gfx: &Graphics, path: &str, filter: Filter) -> Arc<Texture> {
        let source_file = std::fs::File::open(path).unwrap();

        let mut decoder = png::Decoder::new(source_file);
        let image_info = decoder.read_header_info().unwrap();

        let image_extent = [image_info.width, image_info.height, 1];

        assert!(image_info.bytes_per_pixel() == 4);

        Self::new_inner(
            gfx,
            path,
            || {
                let mut reader = decoder.read_info().unwrap();
                let mut buffer = vec![0; reader.output_buffer_size()];
                reader.next_frame(&mut buffer).unwrap();
                buffer
            },
            image_extent,
            None,
            Format::R8G8B8A8_SRGB,
            LayoutCreateArgs {
                shader_strages: ShaderStages::FRAGMENT,
                min_filter: filter,
                mag_filter: filter,
            },
        )
    }

    pub fn new_array(gfx: &Graphics, path: &str, layer_dimensions: [u32; 2]) -> Arc<Texture> {
        let source_file = std::fs::File::open(path).unwrap();

        let mut decoder = png::Decoder::new(source_file);
        let image_info = decoder.read_header_info().unwrap();

        let cols = image_info.width / layer_dimensions[0];
        let rows = image_info.height / layer_dimensions[1];

        let extent = [layer_dimensions[0], layer_dimensions[1], 1];

        let array_layers = rows * cols;

        assert!(image_info.bytes_per_pixel() == 4);

        let bytes_closure = || {
            let mut reader = decoder.read_info().unwrap();
            let mut buffer = vec![0; reader.output_buffer_size()];
            reader.next_frame(&mut buffer).unwrap();

            let mut rearranged_image_data = vec![0u8; buffer.len()];
            let chunk_size = 4 * layer_dimensions[0] as usize;

            for (i, source_chunk) in buffer.chunks(chunk_size).enumerate() {
                let chunks_per_layer = layer_dimensions[1] as usize;
                let chunks_per_row = cols as usize;
                let layers_per_row = chunks_per_row;

                let chunk_x = i % chunks_per_row;
                let chunk_y = i / chunks_per_row;

                let layer_x = chunk_x;
                let layer_y = chunk_y / chunks_per_layer;

                let layer_idx = layer_x + layer_y * layers_per_row;

                let target_chunk_idx = chunks_per_layer * layer_idx + chunk_y % chunks_per_layer;

                let target_chunk_start = target_chunk_idx * chunk_size;
                let target_chunk_end = target_chunk_start + chunk_size;

                rearranged_image_data[target_chunk_start..target_chunk_end]
                    .clone_from_slice(source_chunk);
            }
            rearranged_image_data
        };

        Self::new_inner(
            gfx,
            path,
            bytes_closure,
            extent,
            Some(array_layers),
            Format::R8G8B8A8_SRGB,
            LayoutCreateArgs {
                shader_strages: ShaderStages::FRAGMENT,
                min_filter: Filter::Nearest,
                mag_filter: Filter::Nearest,
            },
        )
    }

    fn new_inner(
        gfx: &Graphics,
        source_file_name: &str,
        bytes: impl FnOnce() -> Vec<u8>,
        extent: [u32; 3],
        array_layers: Option<u32>,
        image_format: Format,
        layout: LayoutCreateArgs,
    ) -> Arc<Self> {
        static TEXTURE_CACHE: LazyLock<RwLock<HashMap<u64, Weak<Texture>>>> =
            LazyLock::new(|| RwLock::new(HashMap::new()));
        let hasher = &mut TEXTURE_CACHE.read().unwrap().hasher().build_hasher();
        source_file_name.hash(hasher);
        extent.hash(hasher);
        array_layers.hash(hasher);
        let texture_id = hasher.finish();

        if let Some(texture) = TEXTURE_CACHE
            .read()
            .unwrap()
            .get(&texture_id)
            .and_then(Weak::upgrade)
        {
            return texture;
        }

        let mut uploads = AutoCommandBufferBuilder::primary(
            gfx.get_cmd_allocator(),
            gfx.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let bytes = bytes();

        let staging_buffer = Buffer::new_slice(
            gfx.get_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            bytes.len() as u64,
        )
        .unwrap();

        staging_buffer.write().unwrap().copy_from_slice(&bytes);

        let image = Image::new(
            gfx.get_allocator(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: image_format,
                extent,
                array_layers: array_layers.unwrap_or(1),
                usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();

        uploads
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                staging_buffer,
                image.clone(),
            ))
            .unwrap();

        uploads
            .build()
            .unwrap()
            .execute(gfx.graphics_queue())
            .unwrap()
            .flush()
            .unwrap();

        let view_type = match (extent, array_layers.is_some()) {
            ([_, _, 1], false) => ImageViewType::Dim2d,
            ([_, _, 1], true) => ImageViewType::Dim2dArray,
            (_, false) => ImageViewType::Dim3d,
            (_, true) => panic!("A 3d texture can't have multiple layers."),
        };

        let image_view = ImageView::new(
            image.clone(),
            ImageViewCreateInfo {
                view_type,
                ..ImageViewCreateInfo::from_image(&image)
            },
        )
        .unwrap();

        let layout = Self::get_descriptor_set_layout(gfx, layout);

        let set = PersistentDescriptorSet::new(
            gfx.get_descriptor_set_allocator(),
            layout.clone(),
            [WriteDescriptorSet::image_view(0, image_view.clone())],
            [],
        )
        .unwrap();

        let texture = Arc::new(Self {
            image_view,
            layout: layout,
            descriptor_set: set,
        });

        TEXTURE_CACHE
            .write()
            .unwrap()
            .insert(texture_id, Arc::downgrade(&texture));

        return texture;
    }

    fn get_descriptor_set_layout(
        gfx: &Graphics,
        args: LayoutCreateArgs,
    ) -> Arc<DescriptorSetLayout> {
        static LAYOUT_CACHE: LazyLock<
            RwLock<HashMap<LayoutCreateArgs, Weak<DescriptorSetLayout>>>,
        > = LazyLock::new(|| RwLock::new(HashMap::new()));

        if let Some(weak_ptr) = LAYOUT_CACHE.read().unwrap().get(&args) {
            match weak_ptr.upgrade() {
                Some(arc) => return arc,
                None => {
                    LAYOUT_CACHE.write().unwrap().remove(&args);
                }
            }
        }

        let sampler = Sampler::new(
            gfx.get_device(),
            SamplerCreateInfo {
                min_filter: args.min_filter,
                mag_filter: args.mag_filter,
                ..SamplerCreateInfo::simple_repeat_linear_no_mipmap()
            },
        )
        .unwrap();

        let layout = DescriptorSetLayout::new(
            gfx.get_device(),
            DescriptorSetLayoutCreateInfo {
                bindings: [(
                    0,
                    DescriptorSetLayoutBinding {
                        stages: args.shader_strages,
                        descriptor_count: 1,
                        immutable_samplers: vec![sampler],
                        ..DescriptorSetLayoutBinding::descriptor_type(
                            DescriptorType::CombinedImageSampler,
                        )
                    },
                )]
                .into(),
                ..Default::default()
            },
        )
        .unwrap();

        LAYOUT_CACHE
            .write()
            .unwrap()
            .insert(args, Arc::downgrade(&layout));
        return layout;
    }
}

pub struct TextureBinding {
    texture_ref: UnsafeCell<Arc<Texture>>,
    set_num: u32,
}

impl TextureBinding {
    pub fn new(texture: Arc<Texture>, set_num: u32) -> Arc<Self> {
        Arc::new(Self {
            texture_ref: UnsafeCell::new(texture),
            set_num: set_num,
        })
    }

    pub fn set_texture(&self, texture: Arc<Texture>) {
        unsafe { *self.texture_ref.get() = texture };
    }
    pub fn get_texture(&self) -> Arc<Texture> {
        unsafe { &*self.texture_ref.get() }.clone()
    }
}

impl Bindable for TextureBinding {
    fn bind_to_pipeline(&self, builder: &mut PipelineBuilder) {
        builder.add_descriptor_set_layout(
            self.set_num,
            unsafe { &*self.texture_ref.get() }.layout.clone(),
        );
    }

    fn bind(
        &self,
        _gfx: &Graphics,
        builder: &mut CommandBufferBuilder,
        pipeline_layout: Arc<vulkano::pipeline::PipelineLayout>,
    ) {
        builder
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics,
                pipeline_layout,
                self.set_num,
                unsafe { &*self.texture_ref.get() }.descriptor_set.clone(),
            )
            .unwrap();
    }
}
