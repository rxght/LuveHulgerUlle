use std::sync::Arc;

use vulkano::{
    buffer::BufferContents, image::ImageViewAbstract, pipeline::graphics::vertex_input::Vertex,
    shader::ShaderStages,
};

use crate::graphics::{
    bindable::{self, PushConstant},
    camera::Camera,
    drawable::{DrawableEntry, GenericDrawable},
    shaders::{frag_textured, vert_animated_tile, vert_tile},
    Graphics,
};

use super::TileSet;

pub struct AnimatedTile {
    pub data: Arc<PushConstant<vert_animated_tile::ObjectData>>,
    descriptor: AnimatedTileDesc,
    tile_set: Arc<TileSet>,
    entry: DrawableEntry,
}

pub struct AnimatedTileDesc {
    pub tile_position: [u32; 2],
    pub first_sprite_idx: u32,
    pub frame_stride: u32,
}

impl AnimatedTile {
    pub fn new(
        gfx: &mut Graphics,
        tile_set: Arc<TileSet>,
        tile_desc: AnimatedTileDesc,
        scale: f32,
        camera: &Camera,
    ) -> Self {
        let data = bindable::PushConstant::new(
            gfx,
            0,
            vert_animated_tile::ObjectData {
                object_position: [
                    tile_desc.tile_position[0] as f32 * scale,
                    tile_desc.tile_position[1] as f32 * -scale,
                ],
                base_uv_offset: tile_set.get_uv_of_sprite(tile_desc.first_sprite_idx)[0],
                frame_uv_stride: tile_desc.frame_stride as f32 * tile_set.tile_width as f32
                    / tile_set.get_texture().image.dimensions().width() as f32,
                frame_offset: 0,
            },
            ShaderStages::VERTEX,
        );

        let mut entry = GenericDrawable::new(
            gfx,
            || {
                vec![
                    bindable::TextureBinding::new(tile_set.get_texture(), 1),
                    data.clone(),
                ]
            },
            || {
                #[derive(BufferContents, Vertex)]
                #[repr(C)]
                struct VertexT {
                    #[format(R32G32_SFLOAT)]
                    pos: [f32; 2],
                    #[format(R32G32_SFLOAT)]
                    uv: [f32; 2],
                }

                let uvs = tile_set.get_uv_of_sprite(0);
                let vertices = vec![
                    VertexT {
                        pos: [0.0, 0.0],
                        uv: uvs[0],
                    },
                    VertexT {
                        pos: [scale, 0.0],
                        uv: uvs[1],
                    },
                    VertexT {
                        pos: [0.0, -scale],
                        uv: uvs[2],
                    },
                    VertexT {
                        pos: [scale, -scale],
                        uv: uvs[3],
                    },
                ];

                let indices = vec![0, 1, 2, 2, 1, 3];

                vec![
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                    bindable::VertexShader::from_module(
                        vert_animated_tile::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::FragmentShader::from_module(
                        frag_textured::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::UniformBufferBinding::new(
                        gfx.get_utils().cartesian_to_normalized.clone(),
                        0,
                    ),
                    bindable::UniformBufferBinding::new(camera.uniform_buffer(), 2),
                ]
            },
        );

        gfx.register_drawable(&mut entry);

        Self {
            data: data,
            tile_set: tile_set,
            entry: entry,
            descriptor: tile_desc,
        }
    }
}

pub struct Tile {
    descriptor: TileDesc,
    tile_set: Arc<TileSet>,
    entry: DrawableEntry,
}

pub struct TileDesc {
    pub tile_position: [u32; 2],
    pub sprite_idx: u32,
}

impl Tile {
    pub fn new(
        gfx: &mut Graphics,
        tile_set: Arc<TileSet>,
        tile_desc: TileDesc,
        camera: &Camera,
    ) -> Self {
        let mut entry = GenericDrawable::new(
            gfx,
            || vec![bindable::TextureBinding::new(tile_set.get_texture(), 1)],
            || {
                #[derive(BufferContents, Vertex)]
                #[repr(C)]
                struct VertexT {
                    #[format(R32G32_SFLOAT)]
                    pos: [f32; 2],
                    #[format(R32G32_SFLOAT)]
                    uv: [f32; 2],
                }

                let uvs = tile_set.get_uv_of_sprite(0);
                let vertices = vec![
                    VertexT {
                        pos: [0.0, 0.0],
                        uv: uvs[0],
                    },
                    VertexT {
                        pos: [1.0, 0.0],
                        uv: uvs[1],
                    },
                    VertexT {
                        pos: [0.0, 1.0],
                        uv: uvs[2],
                    },
                    VertexT {
                        pos: [1.0, 1.0],
                        uv: uvs[3],
                    },
                ];

                let indices = vec![0, 1, 2, 2, 1, 3];

                vec![
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                    bindable::VertexShader::from_module(vert_tile::load(gfx.get_device()).unwrap()),
                    bindable::FragmentShader::from_module(
                        frag_textured::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::UniformBufferBinding::new(
                        gfx.get_utils().cartesian_to_normalized.clone(),
                        0,
                    ),
                    bindable::UniformBufferBinding::new(camera.uniform_buffer(), 2),
                ]
            },
        );

        gfx.register_drawable(&mut entry);

        Self {
            tile_set: tile_set,
            entry: entry,
            descriptor: tile_desc,
        }
    }
}
