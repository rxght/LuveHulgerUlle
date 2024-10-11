use std::{collections::HashMap, ops::Mul, sync::Arc};

use vulkano::{
    buffer::BufferContents, image::ImageViewAbstract, pipeline::graphics::vertex_input::Vertex,
};

use crate::graphics::{
    bindable,
    camera::Camera,
    drawable::{DrawableEntry, GenericDrawable},
    shaders::{frag_textured, vert_textured, vert_tile},
    Graphics,
};

use super::{tile::AnimatedTile, AnimationDesc, TileSet};

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
}

pub struct StaticTileGroup {
    tile_set: Arc<TileSet>,
    dimensions: [u32; 2],
    tiles: Vec<Option<u32>>,
    drawable: DrawableEntry,
}

impl StaticTileGroup {
    pub fn new(
        gfx: &mut Graphics,
        tile_set: Arc<TileSet>,
        dimensions: [u32; 2],
        tiles: Vec<Option<u32>>,
        tile_scale: f32,
        camera: &Camera,
    ) -> Self {
        let tile_count = u32::min(tiles.len() as u32, dimensions[0] * dimensions[1]);
        let mut vertices = Vec::with_capacity(4 * tile_count as usize);
        let mut indices = Vec::with_capacity(6 * tile_count as usize);

        for y in 0..dimensions[1] {
            for x in 0..dimensions[0] {
                if let Some(tile_idx) = tiles[(y * dimensions[0] + x) as usize] {
                    let left = x as f32 * tile_scale;
                    let right = (x + 1) as f32 * tile_scale;
                    let bottom = (y + 1) as f32 * -tile_scale;
                    let top = y as f32 * -tile_scale;

                    let uvs = tile_set.get_uv_of_sprite(tile_idx);

                    vertices.push(VertexT {
                        pos: [left, top],
                        uv: uvs[0],
                    });
                    vertices.push(VertexT {
                        pos: [right, top],
                        uv: uvs[1],
                    });
                    vertices.push(VertexT {
                        pos: [left, bottom],
                        uv: uvs[2],
                    });
                    vertices.push(VertexT {
                        pos: [right, bottom],
                        uv: uvs[3],
                    });

                    let index_offset = 4 * (y * dimensions[0] + x);
                    indices.extend(
                        [0, 1, 2, 2, 1, 3]
                            .into_iter()
                            .map(|elem| elem + index_offset),
                    );
                }
            }
        }

        let mut drawable_entry = GenericDrawable::new(
            gfx,
            || {
                vec![
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                ]
            },
            || {
                vec![
                    bindable::VertexShader::from_module(vert_tile::load(gfx.get_device()).unwrap()),
                    bindable::FragmentShader::from_module(
                        frag_textured::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::UniformBufferBinding::new(
                        gfx.get_utils().cartesian_to_normalized.clone(),
                        0,
                    ),
                    bindable::TextureBinding::new(tile_set.get_texture(), 1),
                    bindable::UniformBufferBinding::new(camera.uniform_buffer(), 2),
                ]
            },
        );

        gfx.register_drawable(&mut drawable_entry);

        Self {
            tile_set: tile_set,
            dimensions: dimensions,
            tiles: tiles,
            drawable: drawable_entry,
        }
    }
}
