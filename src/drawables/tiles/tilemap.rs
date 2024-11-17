use std::{arch::asm, collections::HashMap, ops::Mul, sync::Arc};

use tiled::TileLayer;
use vulkano::{
    buffer::BufferContents, image::ImageViewAbstract, pipeline::graphics::vertex_input::Vertex,
};

use crate::graphics::{
    bindable,
    camera::Camera,
    drawable::{DrawableEntry, GenericDrawable},
    shaders::{frag_textured, vert_tile},
    Graphics,
};

use super::TileSet;

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
}

pub struct TileMap {
    tile_set: Arc<TileSet>,
    pub layers: Vec<DrawableEntry>,
}

impl TileMap {
    pub fn new(
        gfx: &mut Graphics,
        map_file: &str,
        tile_set: Arc<TileSet>,
        camera: &Camera,
    ) -> Self {
        let mut loader = tiled::Loader::new();
        let map = loader.load_tmx_map(map_file).unwrap();
        let tile_scale = map.tile_width as f32;

        let width = map.width;
        let height = map.height;
        let tile_count = width * height;

        let mut layers = Vec::with_capacity(map.layers().count());

        for layer in map.layers() {
            let layer = match layer.as_tile_layer() {
                Some(v) => v,
                None => continue,
            };

            let mut vertices = Vec::with_capacity(4 * tile_count as usize);
            let mut indices = Vec::with_capacity(6 * tile_count as usize);

            for y in 0..height as i32 {
                for x in 0..width as i32 {
                    if let Some(tile) = layer.get_tile(x, y) {
                        let left = x as f32 * tile_scale;
                        let right = (x + 1) as f32 * tile_scale;
                        let bottom = (y + 1) as f32 * -tile_scale;
                        let top = y as f32 * -tile_scale;

                        let uvs = tile_set.get_uv_of_sprite(tile.id());
                        let index_offset = vertices.len() as u32;

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

                        indices.extend(
                            [0, 1, 2, 2, 1, 3]
                                .into_iter()
                                .map(|elem| elem + index_offset),
                        );
                    }
                }
            }

            let index_count = indices.len() as u32;

            layers.push(GenericDrawable::new(
                gfx,
                || {
                    vec![
                        bindable::VertexBuffer::new(gfx, vertices),
                        bindable::IndexBuffer::new(gfx, indices),
                    ]
                },
                || {
                    vec![
                        bindable::VertexShader::from_module(
                            vert_tile::load(gfx.get_device()).unwrap(),
                        ),
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
                index_count,
            ));
        }

        Self {
            tile_set: tile_set,
            layers,
        }
    }
}
