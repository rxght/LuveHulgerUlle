use std::{cell::Cell, collections::HashMap, hash::Hash, path::Path, sync::Arc};
use tiled::TileId;
use vulkano::{
    buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::ShaderStages,
};

use crate::graphics::{
    bindable::{self, PushConstant, Texture},
    camera::Camera,
    drawable::Drawable,
    shaders::{
        frag_texture_array,
        vert_tile::{self, FrameData},
    },
    Graphics,
};

#[derive(PartialEq, Eq, Hash, Clone)]
struct AnimationDesc {
    frames: Vec<(u32, u32)>,
}

struct TileAnimation {
    buffer: Arc<PushConstant<FrameData>>,
    last_frame_time: Cell<std::time::Instant>,
    current_frame: Cell<usize>,
}

impl TileAnimation {
    pub fn new() -> Self {
        Self {
            buffer: PushConstant::new(0, FrameData { layer_offset: 0.0 }, ShaderStages::VERTEX),
            last_frame_time: Cell::new(std::time::Instant::now()),
            current_frame: Cell::new(0),
        }
    }
}

struct TileGroup {
    pub tiles: Vec<Tile>,
    pub drawable: Arc<Drawable>,
}

#[derive(Clone)]
pub struct Tile {
    position: [u32; 2],
    texture_id: u32,
    type_str: Option<Arc<str>>,
}

pub struct TileMap {
    // draw times are much faster because the tiles are grouped together (90% sure this is true)
    groups: Vec<TileGroup>,
}

impl TileMap {
    pub fn draw(&self, gfx: &mut Graphics) {
        self.groups
            .iter()
            .for_each(|group| gfx.queue_drawable(group.drawable.clone()));
    }
}

pub struct TileMapLoader {
    loaded_tile_maps: HashMap<String, Arc<TileMap>>,
    animations: HashMap<AnimationDesc, Arc<TileAnimation>>,
    no_frame_offset_buffer: Arc<PushConstant<FrameData>>,
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    uv: [f32; 3],
}

impl TileMapLoader {
    pub fn new() -> Self {
        Self {
            loaded_tile_maps: HashMap::new(),
            animations: HashMap::new(),
            no_frame_offset_buffer: PushConstant::new(
                0,
                FrameData { layer_offset: 0.0 },
                ShaderStages::VERTEX,
            ),
        }
    }

    pub fn load(&mut self, gfx: &Graphics, path: &'static str, camera: &Camera) -> Arc<TileMap> {
        let file_name = Path::new(path).file_name().unwrap().to_str().unwrap();

        if let Some(cached_map) = self.loaded_tile_maps.get(file_name) {
            return cached_map.clone();
        }

        let mut loader = tiled::Loader::new();
        let map = loader.load_tmx_map(path).unwrap();
        let tile_scale = map.tile_width as f32;

        let width = map.width;
        let height = map.height;

        #[derive(Default)]
        struct TileGroupData {
            tiles: Vec<Tile>,
            animations: HashMap<TileId, AnimationDesc>,
        }

        let mut final_layers = Vec::new();

        for layer in map.layers() {
            let mut groups: HashMap<&str, TileGroupData> = HashMap::new();

            let layer = match layer.as_tile_layer() {
                Some(v) => v,
                None => continue,
            };
            for y in 0..height as i32 {
                for x in 0..width as i32 {
                    let tile = match layer.get_tile(x, y) {
                        Some(v) => v,
                        None => continue,
                    };

                    let texture_path = tile
                        .get_tileset()
                        .image
                        .as_ref()
                        .unwrap()
                        .source
                        .to_str()
                        .unwrap();

                    let group = groups.entry(texture_path).or_default();

                    if let Some(animation) =
                        tile.get_tile().as_ref().and_then(|p| p.animation.as_ref())
                    {
                        let tile_id = tile.id();
                        if !group.animations.contains_key(&tile_id) {
                            group.animations.insert(
                                tile_id,
                                AnimationDesc {
                                    frames: animation
                                        .iter()
                                        .map(|p| (p.tile_id - tile_id, p.duration))
                                        .collect(),
                                },
                            );
                        }
                    }

                    group.tiles.push(Tile {
                        position: [x as u32, y as u32],
                        texture_id: tile.id() as u32,
                        type_str: None,
                    });
                }
            }

            for (texture_path, group) in groups.into_iter() {
                let texture = Texture::new_array(gfx, texture_path, [16, 16]);

                let mut animated_groups: HashMap<&AnimationDesc, Vec<Tile>> = HashMap::new();

                let mut vertices = Vec::new();
                let mut indices = Vec::new();

                for tile in &group.tiles {
                    match group.animations.get(&tile.texture_id) {
                        Some(animation) => animated_groups
                            .entry(animation)
                            .or_default()
                            .push(tile.clone()),
                        None => {
                            Self::add_tile_to_mesh(&mut vertices, &mut indices, tile, tile_scale)
                        }
                    }
                }

                if let Some(drawable) = Self::create_drawable(
                    gfx,
                    vertices,
                    indices,
                    texture.clone(),
                    camera,
                    self.no_frame_offset_buffer.clone(),
                ) {
                    let layer = TileGroup {
                        drawable,
                        tiles: group.tiles,
                    };
                    final_layers.push(layer);
                }

                for (animation, tiles) in animated_groups {
                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();

                    for tile in &tiles {
                        Self::add_tile_to_mesh(&mut vertices, &mut indices, tile, tile_scale);
                    }

                    let animation_state = match self.animations.get(animation) {
                        Some(anim) => anim.clone(),
                        None => {
                            let tile_anim = Arc::new(TileAnimation::new());
                            self.animations.insert(animation.clone(), tile_anim.clone());
                            tile_anim
                        }
                    };

                    if let Some(drawable) = Self::create_drawable(
                        gfx,
                        vertices,
                        indices,
                        texture.clone(),
                        camera,
                        animation_state.buffer.clone(),
                    ) {
                        let layer = TileGroup { drawable, tiles };
                        final_layers.push(layer);
                    }
                }
            }
        }

        Arc::new(TileMap {
            groups: final_layers,
        })
    }

    fn add_tile_to_mesh(
        vertices: &mut Vec<VertexT>,
        indices: &mut Vec<u32>,
        tile_info: &Tile,
        tile_scale: f32,
    ) {
        let [x, y] = tile_info.position;
        let uv_layer = tile_info.texture_id as f32;

        let min_x = x as f32 * tile_scale;
        let max_x = (x + 1) as f32 * tile_scale;
        let min_y = y as f32 * tile_scale;
        let max_y = (y + 1) as f32 * tile_scale;

        let first_vertex_idx = vertices.len() as u32;

        vertices.extend([
            VertexT {
                pos: [min_x, min_y],
                uv: [0.0, 0.0, uv_layer],
            },
            VertexT {
                pos: [max_x, min_y],
                uv: [1.0, 0.0, uv_layer],
            },
            VertexT {
                pos: [min_x, max_y],
                uv: [0.0, 1.0, uv_layer],
            },
            VertexT {
                pos: [max_x, max_y],
                uv: [1.0, 1.0, uv_layer],
            },
        ]);

        indices.extend([0, 1, 2, 2, 1, 3].into_iter().map(|p| first_vertex_idx + p));
    }

    fn create_drawable(
        gfx: &Graphics,
        vertices: Vec<VertexT>,
        indices: Vec<u32>,
        texture: Arc<Texture>,
        camera: &Camera,
        frame_data: Arc<PushConstant<FrameData>>,
    ) -> Option<Arc<Drawable>> {
        let index_count = indices.len() as u32;
        if index_count == 0 {
            return None;
        }
        Some(Drawable::new(
            gfx,
            vec![
                bindable::TextureBinding::new(texture, 1),
                bindable::VertexBuffer::new(gfx, vertices),
                bindable::IndexBuffer::new(gfx, indices),
                frame_data,
            ],
            || {
                vec![
                    bindable::VertexShader::from_module(vert_tile::load(gfx.get_device()).unwrap()),
                    bindable::FragmentShader::from_module(
                        frag_texture_array::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::UniformBufferBinding::new(
                        gfx.utils().cartesian_to_normalized().clone(),
                        0,
                    ),
                    bindable::UniformBufferBinding::new(camera.uniform_buffer(), 2),
                ]
            },
            index_count,
        ))
    }

    pub fn update(&mut self) {
        for (offsets, state) in self.animations.iter() {
            let (_, frame_duration) = offsets.frames[state.current_frame.get()];
            if state.last_frame_time.get().elapsed().as_millis() as u32 > frame_duration {
                state
                    .current_frame
                    .set((state.current_frame.get() + 1) % offsets.frames.len());
                state.last_frame_time.set(std::time::Instant::now());
                let (layer_offset, _) = offsets.frames[state.current_frame.get()];
                state
                    .buffer
                    .access_data(|data| data.layer_offset = layer_offset as f32);
            }
        }
    }
}
