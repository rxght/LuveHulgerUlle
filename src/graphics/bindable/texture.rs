use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, LazyLock, RwLock},
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

static NEAREST_NEIGHBOR_LAYOUTS: LazyLock<RwLock<HashMap<u32, Arc<DescriptorSetLayout>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static LINEAR_LAYOUTS: LazyLock<RwLock<HashMap<u32, Arc<DescriptorSetLayout>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

impl Texture {
    pub fn new(gfx: &Graphics, path: &str, binding: u32, use_nearest_neighbor: bool) -> Arc<Self> {
        let mut uploads = AutoCommandBufferBuilder::primary(
            gfx.get_cmd_allocator(),
            gfx.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let image = {
            let bytes = std::fs::read(path).expect("Texture file not found.");
            let cursor = Cursor::new(bytes);
            let decoder = png::Decoder::new(cursor);
            let mut reader = decoder.read_info().unwrap();
            let info = reader.info();
            let dimensions = ImageDimensions::Dim2d {
                width: info.width,
                height: info.height,
                array_layers: 1,
            };

            assert_eq!(
                info.bit_depth,
                png::BitDepth::Eight,
                "Only 32bit colors are supported"
            );

            let mut image_data = vec![0; (info.width * info.height * 4) as usize];
            reader.next_frame(&mut image_data).unwrap();

            let image = ImmutableImage::from_iter(
                gfx.get_allocator(),
                image_data,
                dimensions,
                vulkano::image::MipmapsCount::One,
                Format::R8G8B8A8_SRGB,
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

        let layout = Self::get_descriptor_set_layout(gfx, binding, use_nearest_neighbor);

        fence.wait(None).unwrap();

        let set = PersistentDescriptorSet::new(
            gfx.get_descriptor_set_allocator(),
            layout.clone(),
            [WriteDescriptorSet::image_view(binding, image.clone())],
        )
        .unwrap();

        Arc::new(Self {
            image: image,
            layout: layout,
            descriptor_set: set,
        })
    }

    fn get_descriptor_set_layout(
        gfx: &Graphics,
        binding: u32,
        use_nearest_neighbor: bool,
    ) -> Arc<DescriptorSetLayout> {
        let layout_map = match use_nearest_neighbor {
            true => &NEAREST_NEIGHBOR_LAYOUTS,
            false => &LINEAR_LAYOUTS,
        };

        if let Some(layout) = layout_map.read().unwrap().get(&binding) {
            return layout.clone();
        }

        let filter = match use_nearest_neighbor {
            true => Filter::Nearest,
            false => Filter::Linear,
        };

        let sampler = Sampler::new(
            gfx.get_device(),
            SamplerCreateInfo {
                min_filter: vulkano::sampler::Filter::Linear,
                mag_filter: filter,
                ..SamplerCreateInfo::simple_repeat_linear()
            },
        )
        .unwrap();

        let layout = DescriptorSetLayout::new(
            gfx.get_device(),
            DescriptorSetLayoutCreateInfo {
                bindings: [(
                    binding,
                    DescriptorSetLayoutBinding {
                        stages: ShaderStages::FRAGMENT,
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

        layout_map.write().unwrap().insert(binding, layout.clone());
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
