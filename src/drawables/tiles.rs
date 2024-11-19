mod tileset;

use std::{
    cell::Cell, collections::{HashMap, HashSet}, ffi::OsStr, hash::{BuildHasher, DefaultHasher, Hash, Hasher, RandomState}, mem::MaybeUninit, path::{Path, PathBuf}, ptr::addr_of, str::FromStr, sync::{Arc, LazyLock, OnceLock}
};
use tiled::{Frame, Image, LayerTile, LayerType};
use tileset::TileSet;
use vulkano::{buffer::BufferContents, image::ImageViewAbstract, pipeline::graphics::vertex_input::Vertex, shader::{ShaderStage, ShaderStages}};

use crate::graphics::{
    bindable::{self, Texture, UniformBuffer},
    camera::Camera,
    drawable::{DrawableEntry, GenericDrawable},
    shaders::{frag_textured, vert_tile},
    Graphics,
};

#[derive(BufferContents, Clone)]
#[repr(C)]
struct FrameData {
    frame_uv_offset: [f32; 2],
}

struct TileAnimation {
    buffer: Arc<UniformBuffer<FrameData>>,
    frames: Vec<([f32; 2], u32)>,
    last_frame_time: Cell<std::time::Instant>,
    current_frame: Cell<usize>,
    id: u64,
}

impl TileAnimation {
    pub fn new<'a>(gfx: &Graphics, id: u64, tile_set: &Arc<TileSet>, frames: Vec<Frame>) -> Self {
        Self {
            buffer: UniformBuffer::new(gfx, 0, FrameData{ frame_uv_offset: [0.0, 0.0] }, ShaderStages::VERTEX),
            last_frame_time: Cell::new(std::time::Instant::now()),
            frames: frames.into_iter().map(|p| (tile_set.get_uv_of_sprite(p.tile_id)[0], p.duration)).collect(),
            current_frame: Cell::new(0),
            id,
        }
    }
}

struct AnimatedLayer {
    animation: Arc<TileAnimation>,
    vertices: Vec<VertexT>,
    indices: Vec<u32>,
}

pub struct TileMap {
    pub layers: Vec<DrawableEntry>,
}

pub struct TileMapLoader {
    loaded_tile_sets: HashMap<String, Arc<TileSet>>,
    loaded_tile_maps: HashMap<String, Arc<TileMap>>,
    animations: HashMap<u64, Arc<TileAnimation>>,
    no_frame_offset_buffer: Arc<UniformBuffer<FrameData>>,
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
}

impl TileMapLoader {
    pub fn new(gfx: &Graphics) -> Self {
        Self {
            loaded_tile_sets: HashMap::new(),
            loaded_tile_maps: HashMap::new(),
            animations: HashMap::new(),
            no_frame_offset_buffer: UniformBuffer::new(gfx, 0, FrameData{ frame_uv_offset: [0.0; 2]}, ShaderStages::VERTEX),
        }
    }

    pub fn load(&mut self, gfx: &Graphics, path: &'static str, camera: &Camera) -> Arc<TileMap> {
        let file_name = Path::new(path).file_name().unwrap().to_str().unwrap();

        if let Some(map) = self.loaded_tile_maps.get(file_name) {
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

                let mut animation_layers: HashMap<u64, AnimatedLayer> = HashMap::new();

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

                            if let Some(animation) = self.get_animation(gfx, tile) {
                                let layer = animation_layers.entry(animation.id).or_insert(AnimatedLayer{ vertices: Vec::new(), indices: Vec::new(), animation: animation.clone(), });
                                
                                let index_offset = layer.vertices.len() as u32;

                                let uv_width = tile_set.tile_width as f32 / tile_set.get_texture().image.dimensions().width() as f32;
                                let uv_height = tile_set.tile_width as f32 / tile_set.get_texture().image.dimensions().height() as f32;

                                layer.vertices.push(VertexT {
                                    pos: [left, top],
                                    uv: [0.0, 0.0],
                                });
                                layer.vertices.push(VertexT {
                                    pos: [right, top],
                                    uv: [uv_width, 0.0],
                                });
                                layer.vertices.push(VertexT {
                                    pos: [left, bottom],
                                    uv: [0.0, uv_height],
                                });
                                layer.vertices.push(VertexT {
                                    pos: [right, bottom],
                                    uv: [uv_width, uv_height],
                                });

                                layer.indices.extend(
                                    [0, 1, 2, 2, 1, 3]
                                        .into_iter()
                                        .map(|elem| elem + index_offset),
                                );
                            } else {
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
                }

                if let Some(layer) = Self::create_layer(gfx, vertices, indices, tile_set, camera, self.no_frame_offset_buffer.clone()) {
                    layers.push(layer);
                }
                
                for (_, animated_layer) in animation_layers {
                    if let Some(layer) = Self::create_layer(gfx, animated_layer.vertices, animated_layer.indices, tile_set, camera, animated_layer.animation.buffer.clone()) {
                        layers.push(layer);
                    }
                }
            }
        }

        Arc::new(TileMap {
            layers,
        })
    }

    fn get_animation<'a>(&mut self, gfx: &Graphics, tile: LayerTile<'a>) -> Option<Arc<TileAnimation>> {
        let frames = tile.get_tile()?.clone().animation?;
        let hasher = &mut self.animations.hasher().build_hasher();
        tile.tileset_index().hash(hasher);
        tile.id().hash(hasher);
        frames.iter().for_each(|p| {
            p.tile_id.hash(hasher);
            p.duration.hash(hasher);
        });
        let id = hasher.finish();
        if self.animations.contains_key(&id) {
            return self.animations.get(&id).cloned();
        }
        else {
            let tile_set = self.loaded_tile_sets.get(tile.get_tileset().image.as_ref()?.source.file_name()?.to_str()?)?;
            self.animations.insert(id, Arc::new(TileAnimation::new(gfx, id, tile_set, frames)));
            return self.animations.get(&id).cloned();
        }
    }

    fn create_layer(gfx: &Graphics, vertices: Vec<VertexT>, indices: Vec<u32>, tile_set: &Arc<TileSet>, camera: &Camera, frame_data: Arc<UniformBuffer<FrameData>>) -> Option<DrawableEntry> {
        let index_count = indices.len() as u32;
        if index_count == 0 {
            return None;
        }
        Some(GenericDrawable::new(
            gfx,
            || {
                vec![
                    bindable::TextureBinding::new(tile_set.get_texture(), 1),
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                    bindable::UniformBufferBinding::new(frame_data, 3),
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
        ))
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

    pub fn update(&mut self) {
        self.animations.values_mut().for_each(|animation| {
            let frame_idx = animation.current_frame.get();
            let (_, frame_duration) = animation.frames[frame_idx];
            if animation.last_frame_time.get().elapsed().as_millis() as u32 > frame_duration {
                animation.current_frame.set((frame_idx + 1) % animation.frames.len());
                animation.last_frame_time.set(std::time::Instant::now());
                let (uv_offset, _) = animation.frames[animation.current_frame.get()];
                animation.buffer.access_data(|data| data.frame_uv_offset = uv_offset);
            }
        });
    }
}
