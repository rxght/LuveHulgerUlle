use std::{cell::Cell, sync::Arc};

use vulkano::{
    buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::ShaderStages,
};

use crate::graphics::{
    bindable::{self, UniformBuffer, UniformBufferBinding},
    drawable::Drawable,
    shaders::{frag_color, vert_square},
    Graphics,
};

use super::{NormalizedRectangle, UiElement, UiLayout};

pub struct UiSquare {
    layout: UiLayout,
    descriptor: Cell<NormalizedRectangle>,
    pub drawable: Arc<Drawable>,
    pub data: Arc<UniformBuffer<vert_square::LayoutData>>,
}

impl UiSquare {
    pub fn new(gfx: &mut Graphics, color: [f32; 4], layout: UiLayout) -> Arc<Self> {
        let window_size: [u32; 2] = gfx.get_window().inner_size().into();
        let descriptor = layout.normalize(window_size);

        let data = UniformBuffer::new(
            gfx,
            0,
            vert_square::LayoutData {
                position: [descriptor.x_position, descriptor.y_position],
                dimensions: [descriptor.width, descriptor.height],
            },
            ShaderStages::VERTEX,
        );

        let drawable = Drawable::new(
            gfx,
            vec![
                UniformBufferBinding::new(data.clone(), 0),
                UniformBufferBinding::new(
                    UniformBuffer::new(
                        gfx,
                        0,
                        frag_color::ColorData { color },
                        ShaderStages::FRAGMENT,
                    ),
                    1,
                ),
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
                        vert_square::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::FragmentShader::from_module(
                        frag_color::load(gfx.get_device()).unwrap(),
                    ),
                ]
            },
            6,
        );

        Arc::new(Self {
            layout,
            descriptor: Cell::new(descriptor),
            drawable,
            data: data,
        })
    }
}

impl UiElement for UiSquare {
    fn handle_resize(&self, new_size: [u32; 2]) {
        let descriptor = self.layout.normalize(new_size);
        self.data.access_data(|data| {
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
