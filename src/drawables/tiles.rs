mod tileset;

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tiled::LayerType;
use tileset::TileSet;
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

use crate::graphics::{
    bindable::{self, Texture},
    camera::Camera,
    drawable::{DrawableEntry, GenericDrawable},
    shaders::{frag_textured, vert_tile},
    Graphics,
};

pub struct TileMap {
    pub layers: Vec<DrawableEntry>,
    pub name: &'static str,
}

pub struct TileMapLoader {
    loaded_tile_sets: HashMap<String, Arc<TileSet>>,
    loaded_tile_maps: Vec<Arc<TileMap>>,
}

impl TileMapLoader {
    pub fn new() -> Self {
        Self {
            loaded_tile_sets: HashMap::new(),
            loaded_tile_maps: vec![],
        }
    }

    pub fn load(&mut self, gfx: &Graphics, path: &'static str, camera: &Camera) -> Arc<TileMap> {
        let file_name = Path::new(path).file_name().unwrap().to_str().unwrap();

        if let Some(map) = self.loaded_tile_maps.iter().find(|p| p.name == file_name) {
            return map.clone();
        }

        let mut loader = tiled::Loader::new();
        let map = loader.load_tmx_map(path).unwrap();
        let tile_scale = map.tile_width as f32;

        let width = map.width;
        let height = map.height;
        let tile_count = width * height;

        let tile_sets = self.load_tilesets(gfx, map.tilesets(), map.tile_width);
        let mut layers = Vec::with_capacity(
            tile_sets.len() * map.layers().filter(|p| p.as_tile_layer().is_some()).count(),
        );

        for layer in map.layers() {
            let layer = match layer.as_tile_layer() {
                Some(v) => v,
                None => continue,
            };

            for (tile_set_name, tile_set) in &tile_sets {
                let mut vertices = Vec::with_capacity(4 * tile_count as usize);
                let mut indices = Vec::with_capacity(6 * tile_count as usize);

                for y in 0..height as i32 {
                    for x in 0..width as i32 {
                        if let Some(tile) = layer.get_tile(x, y) {
                            let current_tile_set_name = tile
                                .get_tileset()
                                .image
                                .as_ref()
                                .and_then(|p| p.source.file_name())
                                .and_then(OsStr::to_str);

                            if current_tile_set_name != Some(*tile_set_name) {
                                continue;
                            }

                            let left = x as f32 * tile_scale;
                            let right = (x + 1) as f32 * tile_scale;
                            let bottom = (y + 1) as f32 * -tile_scale;
                            let top = y as f32 * -tile_scale;

                            let uvs = tile_set.get_uv_of_sprite(tile.id());
                            let index_offset = vertices.len() as u32;

                            #[derive(BufferContents, Vertex)]
                            #[repr(C)]
                            struct VertexT {
                                #[format(R32G32_SFLOAT)]
                                pos: [f32; 2],
                                #[format(R32G32_SFLOAT)]
                                uv: [f32; 2],
                            }

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

                if index_count == 0 {
                    continue;
                }

                layers.push(GenericDrawable::new(
                    gfx,
                    || {
                        vec![
                            bindable::TextureBinding::new(tile_set.get_texture(), 1),
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
                            bindable::UniformBufferBinding::new(camera.uniform_buffer(), 2),
                        ]
                    },
                    index_count,
                ));
            }
        }

        Arc::new(TileMap {
            layers,
            name: file_name,
        })
    }

    fn load_tilesets<'a>(
        &mut self,
        gfx: &Graphics,
        tile_sets: &'a [Arc<tiled::Tileset>],
        tile_width: u32,
    ) -> Vec<(&'a str, Arc<TileSet>)> {
        let mut result = Vec::new();

        for tile_set in tile_sets {
            let source_path = match tile_set.image.as_ref().and_then(|p| p.source.to_str()) {
                Some(v) => v,
                None => continue,
            };

            let file_name = match Path::new(source_path).file_name().and_then(OsStr::to_str) {
                Some(v) => v,
                None => continue,
            };

            match self.loaded_tile_sets.get(file_name) {
                Some(v) => result.push((file_name, v.clone())),
                None => {
                    let new_tile_set = TileSet::new(gfx, source_path, tile_width);
                    result.push((file_name, new_tile_set.clone()));
                    self.loaded_tile_sets
                        .insert(file_name.to_string(), new_tile_set);
                }
            }
        }
        return result;
    }
}
