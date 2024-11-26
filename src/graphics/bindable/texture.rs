use std::{
    collections::HashMap, hash::{BuildHasher, Hash, Hasher}, sync::{Arc, LazyLock, RwLock, Weak}
};

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract},
    descriptor_set::{
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage},
    sampler::{Filter, Sampler, SamplerCreateInfo},
    shader::ShaderStages,
    sync::GpuFuture,
};

use crate::graphics::{pipeline::PipelineBuilder, Graphics};

use super::Bindable;

pub struct Texture {
    pub image: Arc<ImageView<ImmutableImage>>,
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
    pub fn new(gfx: &Graphics, path: &str, filter: Filter) -> Arc<Texture> {
        let source_file = std::fs::File::open(path).unwrap();

        let mut decoder = png::Decoder::new(source_file);
        let image_info = decoder.read_header_info().unwrap();

        let image_dimensions = ImageDimensions::Dim2d {
            width: image_info.width,
            height: image_info.height,
            array_layers: 1,
        };

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
            image_dimensions,
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

        let image_dimensions = ImageDimensions::Dim2d {
            width: layer_dimensions[0],
            height: layer_dimensions[1],
            array_layers: cols * rows,
        };
        
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

                rearranged_image_data[target_chunk_start..target_chunk_end].clone_from_slice(source_chunk);
            }
            rearranged_image_data
        };

        Self::new_inner(
            gfx,
            path,
            bytes_closure,
            image_dimensions,
            Format::R8G8B8A8_SRGB,
            LayoutCreateArgs {
                shader_strages: ShaderStages::FRAGMENT,
                min_filter: Filter::Nearest,
                mag_filter: Filter::Nearest,
            },
        )
    }

    fn new_inner<I>(
        gfx: &Graphics,
        source_file_name: &str,
        bytes: impl FnOnce() -> I,
        dimensions: ImageDimensions,
        image_format: Format,
        layout: LayoutCreateArgs,
    ) -> Arc<Self>
    where
        I: IntoIterator<Item = u8>,
        I::IntoIter: ExactSizeIterator,
    {
        static TEXTURE_CACHE: LazyLock<RwLock<HashMap<u64, Weak<Texture>>>> =
            LazyLock::new(|| RwLock::new(HashMap::new()));
        let hasher = &mut TEXTURE_CACHE.read().unwrap().hasher().build_hasher();
        source_file_name.hash(hasher);
        dimensions.width_height_depth().hash(hasher);
        let texture_id = hasher.finish();

        if let Some(weak_ptr) = TEXTURE_CACHE.read().unwrap().get(&texture_id) {
            match weak_ptr.upgrade() {
                Some(arc) => return arc,
                None => {
                    TEXTURE_CACHE.write().unwrap().remove(&texture_id);
                }
            }
        }

        let mut uploads = AutoCommandBufferBuilder::primary(
            gfx.get_cmd_allocator(),
            gfx.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let image = {
            let dimensions = ImageDimensions::Dim2d {
                width: 0,
                height: 0,
                array_layers: 1,
            };

            let image = ImmutableImage::from_iter(
                gfx.get_allocator(),
                bytes(),
                dimensions,
                vulkano::image::MipmapsCount::One,
                image_format,
                &mut uploads,
            )
            .unwrap();
            ImageView::new_default(image).unwrap()
        };

        let fence = uploads
            .build()
            .unwrap()
            .execute(gfx.graphics_queue())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        let layout = Self::get_descriptor_set_layout(gfx, layout);

        fence.wait(None).unwrap();

        let set = PersistentDescriptorSet::new(
            gfx.get_descriptor_set_allocator(),
            layout.clone(),
            [WriteDescriptorSet::image_view(0, image.clone())],
        )
        .unwrap();

        let texture = Arc::new(Self {
            image: image,
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
                        variable_descriptor_count: false,
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
    texture_ref: Arc<Texture>,
    set_num: u32,
}

impl TextureBinding {
    pub fn new(texture: Arc<Texture>, set_num: u32) -> Arc<Self> {
        Arc::new(Self {
            texture_ref: texture,
            set_num: set_num,
        })
    }
}

impl Bindable for TextureBinding {
    fn bind_to_pipeline(&self, builder: &mut PipelineBuilder) {
        builder.add_descriptor_set_layout(self.set_num, self.texture_ref.layout.clone());
    }

    fn bind(
        &self,
        _gfx: &Graphics,
        builder: &mut AutoCommandBufferBuilder<
            vulkano::command_buffer::PrimaryAutoCommandBuffer,
            vulkano::command_buffer::allocator::StandardCommandBufferAllocator,
        >,
        pipeline_layout: Arc<vulkano::pipeline::PipelineLayout>,
    ) {
        builder.bind_descriptor_sets(
            vulkano::pipeline::PipelineBindPoint::Graphics,
            pipeline_layout,
            self.set_num,
            self.texture_ref.descriptor_set.clone(),
        );
    }
}
