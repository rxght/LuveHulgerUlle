use std::sync::Arc;

use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

use crate::graphics::{
    bindable::{
        FragmentShader, IndexBuffer, Texture, TextureBinding, UniformBufferBinding, VertexBuffer,
        VertexShader,
    },
    drawable::Drawable,
    shaders::{frag_texture_2DArray, vert_ui},
    Graphics,
};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    uv: [f32; 3],
}

pub struct Window {
    drawable: Arc<Drawable>,
}

impl Window {
    pub fn draw(&mut self, gfx: &mut Graphics) {
        gfx.queue_drawable(self.drawable.clone());
    }

    pub fn new(gfx: &mut Graphics, dimensions: [u32; 2], scale: f32) -> Self {
        assert!(dimensions[0] >= 3);
        assert!(dimensions[1] >= 3);

        let mut vertices: Vec<VertexT> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut x = scale * -8.0 * dimensions[0] as f32;
        let mut y = scale * -8.0 * dimensions[1] as f32;

        // top row
        push_tile(&mut vertices, &mut indices, x, y, scale, 0);
        x += 16.0 * scale;
        for _ in 2..dimensions[0] {
            push_tile(&mut vertices, &mut indices, x, y, scale, 1);
            x += 16.0 * scale;
        }
        push_tile(&mut vertices, &mut indices, x, y, scale, 2);
        x = scale * -8.0 * dimensions[0] as f32;
        y += 16.0 * scale;

        //middle rows
        for _ in 2..dimensions[1] {
            push_tile(&mut vertices, &mut indices, x, y, scale, 3);
            x += 16.0 * scale;
            for _ in 2..dimensions[0] {
                push_tile(&mut vertices, &mut indices, x, y, scale, 4);
                x += 16.0 * scale;
            }
            push_tile(&mut vertices, &mut indices, x, y, scale, 5);
            x = scale * -8.0 * dimensions[0] as f32;
            y += 16.0 * scale;
        }

        //last row
        push_tile(&mut vertices, &mut indices, x, y, scale, 6);
        x += 16.0 * scale;
        for _ in 2..dimensions[0] {
            push_tile(&mut vertices, &mut indices, x, y, scale, 7);
            x += 16.0 * scale;
        }
        push_tile(&mut vertices, &mut indices, x, y, scale, 8);

        let index_count = indices.len() as u32;

        let drawable = Drawable::new(
            gfx,
            vec![
                VertexBuffer::new(gfx, vertices),
                IndexBuffer::new(gfx, indices),
            ],
            || {
                vec![
                    TextureBinding::new(
                        Texture::new_array(gfx, "assets/textures/ui/border.png", [16, 16]),
                        1,
                    ),
                    VertexShader::from_module(vert_ui::load(gfx.get_device()).unwrap()),
                    FragmentShader::from_module(
                        frag_texture_2DArray::load(gfx.get_device()).unwrap(),
                    ),
                    UniformBufferBinding::new(gfx.utils().cartesian_to_normalized(), 0),
                ]
            },
            index_count,
        );

        Self { drawable }
    }
}

fn push_tile(
    vertices: &mut Vec<VertexT>,
    indices: &mut Vec<u32>,
    x: f32,
    y: f32,
    scale: f32,
    layer: u32,
) {
    let min_x = x;
    let max_x = x + 16.0 * scale;
    let min_y = y;
    let max_y = y + 16.0 * scale;

    let index_offset = vertices.len() as u32;

    vertices.extend(
        [
            VertexT {
                pos: [min_x, min_y],
                uv: [0.0, 0.0, layer as f32],
            },
            VertexT {
                pos: [max_x, min_y],
                uv: [1.0, 0.0, layer as f32],
            },
            VertexT {
                pos: [min_x, max_y],
                uv: [0.0, 1.0, layer as f32],
            },
            VertexT {
                pos: [max_x, max_y],
                uv: [1.0, 1.0, layer as f32],
            },
        ]
        .into_iter(),
    );

    indices.extend([0, 1, 2, 2, 1, 3].into_iter().map(|i| i + index_offset));
}
