use std::{cell::Cell, sync::Arc};

use vulkano::{
    buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::ShaderStages,
};

use crate::graphics::{
    bindable::{self, Texture, TextureBinding, UniformBuffer, UniformBufferBinding},
    drawable::Drawable,
    shaders::{frag_textured, vert_ui_textured},
    Graphics,
};

use super::{NormalizedRectangle, Rectangle, UiElement, UiLayout};

pub struct UiImage {
    layout: UiLayout,
    descriptor: Cell<NormalizedRectangle>,
    pub drawable: Arc<Drawable>,
    pub layout_data: Arc<UniformBuffer<vert_ui_textured::LayoutData>>,
    pub texture_mapping_data: Arc<UniformBuffer<vert_ui_textured::TextureMappingData>>,
}

impl UiImage {
    pub fn new(
        gfx: &mut Graphics,
        texture: Arc<Texture>,
        texture_mapping: Rectangle,
        layout: UiLayout,
    ) -> Arc<Self> {
        let window_size: [u32; 2] = gfx.get_window().inner_size().into();
        let descriptor = layout.normalize(window_size);

        let layout_data = UniformBuffer::new(
            gfx,
            0,
            vert_ui_textured::LayoutData {
                position: [descriptor.x_position, descriptor.y_position],
                dimensions: [descriptor.width, descriptor.height],
            },
            ShaderStages::VERTEX,
        );

        let texture_size = texture.dimensions().width_height();
        let uv_offset = [
            texture_mapping.x_position as f32 / texture_size[0] as f32,
            texture_mapping.y_position as f32 / texture_size[1] as f32,
        ];
        let uv_scaling = [
            texture_mapping.width as f32 / texture_size[0] as f32,
            texture_mapping.height as f32 / texture_size[1] as f32,
        ];

        let texture_mapping_data = UniformBuffer::new(
            gfx,
            0,
            vert_ui_textured::TextureMappingData {
                uv_offset,
                uv_scaling,
            },
            ShaderStages::VERTEX,
        );

        let drawable = Drawable::new(
            gfx,
            vec![
                UniformBufferBinding::new(layout_data.clone(), 0),
                UniformBufferBinding::new(texture_mapping_data.clone(), 2),
                TextureBinding::new(texture.clone(), 1),
            ],
            || {
                #[derive(BufferContents, Vertex)]
                #[repr(C)]
                struct Vertex {
                    #[format(R32G32_SFLOAT)]
                    pos: [f32; 2],
                }

                let vertices = vec![
                    Vertex { pos: [0.0, 0.0] },
                    Vertex { pos: [1.0, 0.0] },
                    Vertex { pos: [0.0, 1.0] },
                    Vertex { pos: [1.0, 1.0] },
                ];

                let indices: Vec<u32> = vec![0, 1, 3, 0, 3, 2];

                vec![
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                    bindable::VertexShader::from_module(
                        vert_ui_textured::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::FragmentShader::from_module(
                        frag_textured::load(gfx.get_device()).unwrap(),
                    ),
                ]
            },
            6,
        );

        Arc::new(Self {
            layout,
            descriptor: Cell::new(descriptor),
            drawable,
            layout_data,
            texture_mapping_data,
        })
    }
}

impl UiElement for UiImage {
    fn handle_resize(&self, new_size: [u32; 2]) {
        let descriptor = self.layout.normalize(new_size);
        self.layout_data.access_data(|data| {
            data.position = [descriptor.x_position, descriptor.y_position];
            data.dimensions = [descriptor.width, descriptor.height];
        });
        self.descriptor.set(descriptor);
    }
    fn get_drawable(&self) -> Arc<Drawable> {
        self.drawable.clone()
    }
    fn get_layout(&self) -> NormalizedRectangle {
        self.descriptor.get()
    }
}
