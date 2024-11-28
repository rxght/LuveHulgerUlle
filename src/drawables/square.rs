use std::sync::Arc;

use vulkano::{
    buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::ShaderStages,
};

use crate::graphics::{
    bindable::{self, PushConstant, UniformBuffer, UniformBufferBinding},
    drawable::Drawable,
    shaders::{frag_color, vert_square},
    Graphics,
};

pub struct Square {
    drawable: Arc<Drawable>,
    pub data: Arc<PushConstant<vert_square::LayoutData>>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct SquareDesc {
    pub pos: [i32; 2],
    pub width: u32,
    pub height: u32,
}

impl Square {
    pub fn new(gfx: &Graphics, square: SquareDesc, color: [f32; 4]) -> Self {
        let data = PushConstant::new(
            0,
            vert_square::LayoutData {
                position: [square.pos[0] as f32, square.pos[1] as f32],
                dimensions: [square.width as f32, square.height as f32],
            },
            ShaderStages::VERTEX,
        );

        let drawable = Drawable::new(
            gfx,
            vec![
                data.clone(),
                bindable::UniformBufferBinding::new(
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

                let indices: Vec<u32> = vec![0, 3, 1, 0, 2, 3];

                vec![
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                    bindable::VertexShader::from_module(
                        vert_square::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::FragmentShader::from_module(
                        frag_color::load(gfx.get_device()).unwrap(),
                    ),
                    UniformBufferBinding::new(gfx.get_utils().cartesian_to_normalized.clone(), 0),
                ]
            },
            6,
        );
        Self {
            drawable,
            data: data,
        }
    }
}
