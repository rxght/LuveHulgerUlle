use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    hash::Hash,
    path::Path,
    sync::{Arc, Weak},
};

use tiled::FiniteTileLayer;
use vulkano::{
    pipeline::graphics::vertex_input::Vertex, shader::ShaderStages,
};

use crate::graphics::{
    bindable::{PushConstant, Texture, TextureBinding},
    camera::Camera,
    drawable::Drawable,
    shaders::{frag_texture_array, vert_tile2},
    Graphics,
};

#[derive(Vertex, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
}

struct Mesh {
    vertices: Vec<VertexT>,
    indices: Vec<u32>,
}

#[derive(Hash, Clone, PartialEq, Eq)]
struct GroupInfo {
    pub tile_set: Arc<TileSet>,
    pub animation: Option<Arc<TileAnimation>>,
}

pub struct Tile {
    tile_type: Option<Arc<str>>,
    animation: Option<Arc<TileAnimation>>,
    tile_set: Arc<TileSet>,
    tile_id: u32,
    position: [u32; 2],
    drawable: Arc<Drawable>,
    object_data: Arc<PushConstant<vert_tile2::ObjectData>>,
    texture_binding: Arc<TextureBinding>,
}

struct TileMapLayer {
    tiles: Vec<Tile>,
}

impl TileMapLayer {
    pub fn draw(&self, gfx: &mut Graphics) {
        self.tiles.iter().for_each(|tile| gfx.queue_drawable(tile.drawable.clone()));
    }
}

pub struct TileMap {
    layers: Vec<TileMapLayer>,
    tile_dimensions: [u32; 2],
    map_dimensions: [u32; 2],
}

impl TileMap {
    pub fn draw_all_layers(&self, gfx: &mut Graphics) {
        self.layers.iter().for_each(|layer| layer.draw(gfx));
    }
}

#[derive(Clone)]
pub struct TileSet {
    texture: Arc<Texture>,
    tile_dimensions: [u32; 2],
    tile_count: u32,
}

impl PartialEq for TileSet {
    fn eq(&self, other: &Self) -> bool {
        self.texture.image_view() == other.texture.image_view()
            && self.tile_dimensions == other.tile_dimensions
            && self.tile_count == other.tile_count
    }
}

impl Eq for TileSet {}

impl Hash for TileSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.texture.image_view().hash(state);
        self.tile_dimensions.hash(state);
        self.tile_count.hash(state);
    }
}

impl TileSet {
    fn new(gfx: &mut Graphics, set: &tiled::Tileset) -> Self {
        let tile_dimensions = [set.tile_width, set.tile_height];

        let image_path = set.image.as_ref().unwrap().source.to_str().unwrap();
        let atlas_texture = Texture::new_array(gfx, image_path, tile_dimensions);

        TileSet {
            texture: atlas_texture.clone(),
            tile_dimensions,
            tile_count: set.tilecount,
        }
    }
}

pub struct TileMapLoader {
    loaded_tilesets: HashMap<OsString, Weak<TileSet>>,
    animations: HashMap<Vec<AnimationFrame>, Weak<TileAnimation>>,
    tile_types: HashMap<String, Weak<str>>,
}

impl TileMapLoader {
    pub fn new() -> Self {
        Self {
            loaded_tilesets: HashMap::new(),
            animations: HashMap::new(),
            tile_types: HashMap::new(),
        }
    }

    pub fn load(
        &mut self,
        gfx: &mut Graphics,
        path: impl AsRef<Path>,
        camera: &Camera,
    ) -> Result<TileMap, Box<dyn Error>> {
        let mut loader = tiled::Loader::new();
        let map = loader.load_tmx_map(path.as_ref())?;

        let map_dimensions = [map.width, map.height];
        let tile_dimensions = [map.tile_width, map.tile_height];

        let layers = self.load_layers(gfx, map, camera);

        let parsed_map = TileMap {
            layers: layers,
            tile_dimensions,
            map_dimensions,
        };

        Ok(parsed_map)
    }

    fn load_layers(
        &mut self,
        gfx: &mut Graphics,
        map: tiled::Map,
        camera: &Camera,
    ) -> Vec<TileMapLayer> {
        let mut result = Vec::new();
        for layer in map.layers() {
            use tiled::LayerType;
            use tiled::TileLayer;
            match layer.layer_type() {
                LayerType::Tiles(TileLayer::Finite(tile_layer)) => {
                    let parsed_layer = self.load_tile_layer(gfx, tile_layer, camera);
                    result.push(parsed_layer);
                }
                LayerType::Tiles(TileLayer::Infinite(_)) => {
                    println!("Infinite tile layers are not supported.");
                    continue;
                }
                LayerType::Objects(_object_layer) => {
                    println!("Object layers are not supported yet!");
                    // TODO
                    continue;
                }
                LayerType::Image(_) => {
                    println!("Image layers are not supported.");
                    continue;
                }
                LayerType::Group(_) => {
                    println!("Group layers are not supported.");
                    continue;
                }
            }
        }
        return result;
    }

    fn load_tile_layer(
        &mut self,
        gfx: &mut Graphics,
        tile_layer: FiniteTileLayer<'_>,
        camera: &Camera,
    ) -> TileMapLayer {
        let mut parsed_tiles = Vec::new();
        let width = tile_layer.width();
        let height = tile_layer.height();

        for y in 0..height {
            for x in 0..width {
                if let Some(tile) = tile_layer.get_tile(x as i32, y as i32) {
                    let tile_set = self.load_tileset(gfx, tile.get_tileset());
                    if let Some(tile) = self.create_tile(gfx, [x, y], tile, tile_set, camera) {
                        parsed_tiles.push(tile);
                    }
                }
            }
        }
        TileMapLayer {
            tiles: parsed_tiles,
        }
    }

    fn create_tile_mesh() -> Mesh {
        let vertices = vec![
            VertexT { pos: [0.0, 0.0] },
            VertexT { pos: [1.0, 0.0] },
            VertexT { pos: [0.0, 1.0] },
            VertexT { pos: [1.0, 1.0] },
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        Mesh { vertices, indices }
    }

    fn load_tileset(&mut self, gfx: &mut Graphics, set: &tiled::Tileset) -> Arc<TileSet> {
        let key = set.source.as_os_str();
        if let Some(arc) = self.loaded_tilesets.get(key).and_then(Weak::upgrade) {
            return arc;
        }
        let arc = Arc::new(TileSet::new(gfx, &set));
        self.loaded_tilesets
            .insert(key.to_os_string(), Arc::downgrade(&arc));
        return arc;
    }

    fn create_tile(
        &mut self,
        gfx: &mut Graphics,
        position: [u32; 2],
        tile: tiled::LayerTile,
        tile_set: Arc<TileSet>,
        camera: &Camera,
    ) -> Option<Tile> {
        let set_tile = tile.get_tile()?;
        let tile_type = self.get_tile_type(&set_tile);
        let animation = self.get_animation(&set_tile);
        let tile_id = tile.id() as u32;

        let [width, height] = tile_set.tile_dimensions;
        let [x, y] = position;

        let object_data = vert_tile2::ObjectData {
            position: [(x * width) as f32, -1.0 * (y * height) as f32],
            dimensions: [width as f32, height as f32],
            layer_idx: tile_id as f32,
        };

        let object_data = PushConstant::new(0, object_data, ShaderStages::VERTEX);

        let texture_binding = TextureBinding::new(tile_set.texture.clone(), 1);

        use crate::graphics::bindable::*;
        let drawable = Drawable::new(
            gfx,
            vec![object_data.clone(), texture_binding.clone()],
            || {
                let mesh = Self::create_tile_mesh();
                vec![
                    VertexBuffer::new(gfx, mesh.vertices),
                    IndexBuffer::new(gfx, mesh.indices),
                    VertexShader::from_module(vert_tile2::load(gfx.get_device()).unwrap()),
                    FragmentShader::from_module(
                        frag_texture_array::load(gfx.get_device()).unwrap(),
                    ),
                    UniformBufferBinding::new(gfx.utils().cartesian_to_normalized(), 0),
                    UniformBufferBinding::new(camera.uniform_buffer(), 2),
                ]
            },
            6,
        );

        Some(Tile {
            tile_type,
            animation,
            tile_set,
            tile_id,
            position,
            drawable,
            object_data,
            texture_binding,
        })
    }

    fn get_tile_type(&mut self, set_tile: &tiled::Tile<'_>) -> Option<Arc<str>> {
        let type_string = set_tile.user_type.clone()?;
        if let Some(arc) = self.tile_types.get(&type_string).and_then(Weak::upgrade) {
            return Some(arc);
        }
        let arc = Arc::from(type_string.clone().into_boxed_str());
        self.tile_types.insert(type_string, Arc::downgrade(&arc));
        return Some(arc);
    }

    fn get_animation(&mut self, set_tile: &tiled::Tile<'_>) -> Option<Arc<TileAnimation>> {
        let frames = set_tile.animation.as_ref()?;
        let start_id = frames[0].tile_id;
        let parsed_frames: Vec<AnimationFrame> = frames
            .iter()
            .map(|frame| AnimationFrame {
                duration: frame.duration,
                frame_offset: frame.tile_id - start_id,
            })
            .collect();

        if let Some(arc) = self.animations.get(&parsed_frames).and_then(Weak::upgrade) {
            return Some(arc);
        }
        let shared_frames: Arc<[AnimationFrame]> =
            Arc::from(parsed_frames.clone().into_boxed_slice());
        let animation = Arc::new(TileAnimation {
            last_frame_time: std::time::Instant::now(),
            current_frame_idx: 0,
            frames: shared_frames,
        });
        let weak_animation = Arc::downgrade(&animation);

        self.animations.insert(parsed_frames, weak_animation);

        return Some(animation);
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct AnimationFrame {
    /// milliseconds
    pub duration: u32,
    pub frame_offset: u32,
}

#[derive(Hash, PartialEq, Eq)]
struct TileAnimation {
    last_frame_time: std::time::Instant,
    current_frame_idx: u32,
    frames: Arc<[AnimationFrame]>,
}
