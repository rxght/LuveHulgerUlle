use std::sync::Arc;

use vulkano::{
    buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::ShaderStages,
};

use crate::graphics::{
    bindable::{self, Texture, TextureBinding},
    camera::Camera,
    drawable::{DrawableEntry, GenericDrawable},
    shaders::{frag_texture_2DArray, vert_tile2},
    Graphics,
};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
}

pub struct DynamicTile {
    texture: Arc<TextureBinding>,
    frame_data: Arc<bindable::PushConstant<vert_tile2::FrameData>>,
    pub drawable: DrawableEntry,
}

impl DynamicTile {
    pub fn new(gfx: &Graphics, initial_texture: Arc<Texture>, camera: &Camera) -> Self {
        let tile_dimensions = initial_texture.dimensions().width_height();
        assert!(
            initial_texture.dimensions().array_layers() > 1,
            "Dynamic tile requires a texture with multiple layers."
        );

        let frame_data = bindable::PushConstant::new(
            0,
            vert_tile2::FrameData {
                frame_idx: 0.0,
                tile_width: tile_dimensions[0] as f32,
                tile_height: tile_dimensions[1] as f32,
            },
            ShaderStages::VERTEX,
        );

        let texture_binding = bindable::TextureBinding::new(initial_texture, 1);

        let entry = GenericDrawable::new(
            gfx,
            || vec![frame_data.clone()],
            || {
                let vertices = vec![
                    VertexT { pos: [0.0, 0.0] },
                    VertexT { pos: [1.0, 0.0] },
                    VertexT { pos: [0.0, 1.0] },
                    VertexT { pos: [1.0, 1.0] },
                ];

                let indices = vec![0, 1, 2, 2, 1, 3];

                vec![
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                    bindable::VertexShader::from_module(
                        vert_tile2::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::FragmentShader::from_module(
                        frag_texture_2DArray::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::UniformBufferBinding::new(
                        gfx.get_utils().cartesian_to_normalized.clone(),
                        0,
                    ),
                    bindable::UniformBufferBinding::new(camera.uniform_buffer(), 2),
                    texture_binding.clone(),
                ]
            },
            6,
        );

        Self {
            texture: texture_binding,
            frame_data,
            drawable: entry,
        }
    }

    pub fn set_texture(&self, texture: Arc<Texture>) {
        let new_dimensions = texture.dimensions().width_height();
        self.frame_data.access_data(|data| {
            data.tile_width = new_dimensions[0] as f32;
            data.tile_width = new_dimensions[1] as f32;
        });
        self.texture.set_texture(texture);
    }

    pub fn set_layer(&self, layer: u32) {
        self.frame_data.access_data(|data| {
            data.frame_idx = layer as f32;
        });
    }
}
