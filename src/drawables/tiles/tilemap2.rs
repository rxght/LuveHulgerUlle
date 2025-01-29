use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    hash::Hash,
    path::Path,
    sync::{Arc, Weak},
};

use tiled::FiniteTileLayer;
use vulkano::pipeline::graphics::vertex_input::Vertex;

use crate::graphics::{
    bindable::{IndexBuffer, Texture, TextureBinding, VertexBuffer},
    camera::Camera,
    drawable::Drawable,
    shaders::{frag_texture_array, vert_tile3},
    Graphics,
};

#[derive(Vertex, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable)]
#[repr(C)]
struct VertexT {
    #[format(R32G32B32_SFLOAT)]
    pos: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    uv: [f32; 3],
}

struct Mesh {
    vertices: Vec<VertexT>,
    indices: Vec<u32>,
}

pub struct Tile {
    pub tile_type: Option<Arc<str>>,
    pub animation: Option<Arc<TileAnimation>>,
    pub tile_set: Arc<TileSet>,
    pub tile_id: u32,
}

pub struct TileMapLayer {
    tiles: Vec<Option<Tile>>,
}

impl TileMapLayer {
    pub fn new(tiles: Vec<Option<Tile>>) -> Self {
        Self { tiles }
    }
}

pub struct TileMap {
    position_offset: [f32; 2],
    scale: f32,
    layers: Vec<TileMapLayer>,
    tile_dimensions: [u32; 2],
    map_dimensions: [u32; 2],

    drawable: TileMapDrawable,
}

impl TileMap {
    pub fn draw(&self, gfx: &mut Graphics) {
        self.drawable.draw(gfx);
    }

    pub fn dimensions(&self) -> [u32; 2] {
        self.map_dimensions
    }

    pub fn tile_dimensions(&self) -> [u32; 2] {
        self.tile_dimensions
    }

    pub fn layers(&self) -> &[TileMapLayer] {
        &self.layers
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn position(&self) -> [f32; 2] {
        self.position_offset
    }

    pub fn set_position(&mut self, position: [f32; 2]) {
        self.position_offset = position;
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
        position: [f32; 2],
        scale: f32,
        camera: &Camera,
    ) -> Result<TileMap, Box<dyn Error>> {
        let mut loader = tiled::Loader::new();
        let map = loader.load_tmx_map(path.as_ref())?;

        let map_dimensions = [map.width, map.height];
        let tile_dimensions = [map.tile_width, map.tile_height];

        let layers = self.load_layers(gfx, map);

        let drawable = TileMapDrawable::new(gfx, position, scale, &layers, map_dimensions, camera);

        let parsed_map = TileMap {
            position_offset: position,
            scale,
            layers: layers,
            tile_dimensions,
            map_dimensions,
            drawable,
        };

        Ok(parsed_map)
    }

    fn load_layers(&mut self, gfx: &mut Graphics, map: tiled::Map) -> Vec<TileMapLayer> {
        let mut result = Vec::new();
        for layer in map.layers() {
            use tiled::LayerType;
            use tiled::TileLayer;
            match layer.layer_type() {
                LayerType::Tiles(TileLayer::Finite(tile_layer)) => {
                    let parsed_layer = self.load_tile_layer(gfx, tile_layer);
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
    ) -> TileMapLayer {
        let width = tile_layer.width();
        let height = tile_layer.height();
        let tile_count = width as usize * height as usize;

        let mut parsed_tiles = Vec::with_capacity(tile_count);
        for y in 0..height {
            for x in 0..width {
                if let Some(tile) = tile_layer.get_tile(x as i32, y as i32) {
                    let tile_set = self.load_tileset(gfx, tile.get_tileset());
                    if let Some(tile) = self.create_tile(tile, tile_set) {
                        parsed_tiles.push(Some(tile));
                        continue;
                    }
                }
                parsed_tiles.push(None);
            }
        }
        TileMapLayer {
            tiles: parsed_tiles,
        }
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

    fn create_tile(&mut self, tile: tiled::LayerTile, tile_set: Arc<TileSet>) -> Option<Tile> {
        let set_tile = tile.get_tile()?;
        let tile_type = self.get_tile_type(&set_tile);
        let animation = self.get_animation(&set_tile);
        let tile_id = tile.id() as u32;

        Some(Tile {
            tile_type,
            animation,
            tile_set,
            tile_id,
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
pub struct TileAnimation {
    last_frame_time: std::time::Instant,
    current_frame_idx: u32,
    frames: Arc<[AnimationFrame]>,
}

struct TileMapDrawable {
    groups: HashMap<Arc<TileSet>, TileGroupDrawable>,
}

struct TileGroupDrawable {
    drawable: Arc<Drawable>,
    vertex_buffer: Arc<VertexBuffer<VertexT>>,
    index_buffer: Arc<IndexBuffer>,
    texture_binding: Arc<TextureBinding>,
}

struct PositionedTile<'a> {
    tile: &'a Tile,
    position: [u32; 3],
}

impl TileMapDrawable {
    pub fn new(
        gfx: &mut Graphics,
        position: [f32; 2],
        scale: f32,
        layers: &[TileMapLayer],
        map_dimensions: [u32; 2],
        camera: &Camera,
    ) -> Self {
        // gruppera all tiles som använder samma tileset
        let mut grouped_tiles: HashMap<Arc<TileSet>, Vec<PositionedTile>> = HashMap::new();
        for (layer_idx, layer) in layers.iter().enumerate() {
            for (idx, tile) in layer.tiles.iter().enumerate() {
                if let Some(tile) = tile {
                    let x = idx as u32 % map_dimensions[0];
                    let y = idx as u32 / map_dimensions[0];
                    let position = [x, y, layer_idx as u32];
                    let positioned_tile = PositionedTile { tile, position };
                    grouped_tiles
                        .entry(tile.tile_set.clone())
                        .or_insert_with(Vec::new)
                        .push(positioned_tile);
                }
            }
        }

        let mut drawable_groups = HashMap::new();
        for (tile_set, tiles) in grouped_tiles {
            let mesh = Self::create_mesh(tiles, &tile_set, position, scale);
            let drawable = Self::create_drawable(gfx, &tile_set, mesh, camera);
            drawable_groups.insert(tile_set, drawable);
        }

        TileMapDrawable {
            groups: drawable_groups,
        }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        for group in self.groups.values() {
            gfx.queue_drawable(group.drawable.clone());
        }
    }

    fn create_drawable(
        gfx: &mut Graphics,
        tile_set: &TileSet,
        mesh: Mesh,
        camera: &Camera,
    ) -> TileGroupDrawable {
        let vertex_buffer = VertexBuffer::new(gfx, mesh.vertices);
        let index_count = mesh.indices.len() as u32;
        let index_buffer = IndexBuffer::new(gfx, mesh.indices);
        let texture_binding = TextureBinding::new(tile_set.texture.clone(), 1);

        let drawable = Drawable::new(
            gfx,
            vec![
                vertex_buffer.clone(),
                index_buffer.clone(),
                texture_binding.clone(),
            ],
            || {
                use crate::graphics::bindable::*;
                vec![
                    VertexShader::from_module(vert_tile3::load(gfx.get_device()).unwrap()),
                    FragmentShader::from_module(
                        frag_texture_array::load(gfx.get_device()).unwrap(),
                    ),
                    UniformBufferBinding::new(gfx.utils().cartesian_to_normalized(), 0),
                    UniformBufferBinding::new(camera.uniform_buffer(), 2),
                ]
            },
            index_count,
        );

        TileGroupDrawable {
            drawable,
            vertex_buffer,
            index_buffer,
            texture_binding,
        }
    }

    fn create_mesh<'a>(
        tiles: Vec<PositionedTile<'a>>,
        tile_set: &TileSet,
        position: [f32; 2],
        scale: f32,
    ) -> Mesh {
        let mut vertices = Vec::with_capacity(tiles.len() * 4);
        let mut indices = Vec::with_capacity(tiles.len() * 6);

        let [width, height] = tile_set.tile_dimensions;
        let [x_offset, y_offset] = [position[0] * width as f32, position[1] * height as f32];

        for tile in tiles {
            let [x, y, z] = tile.position;
            let tile = tile.tile;

            let min_x = ((x * width) as f32 + x_offset) * scale;
            let max_x = (((x + 1) * width) as f32 + x_offset) * scale;
            let min_y = -(((y + 1) * height) as f32 + y_offset) * scale;
            let max_y = -((y * height) as f32 + y_offset) * scale;

            let z = -1.0 + 0.01 * z as f32;

            let uv_z = tile.tile_id as f32;

            let index_offset = vertices.len() as u32;
            vertices.extend([
                VertexT {
                    pos: [min_x, min_y, z],
                    uv: [0.0, 1.0, uv_z],
                },
                VertexT {
                    pos: [min_x, max_y, z],
                    uv: [0.0, 0.0, uv_z],
                },
                VertexT {
                    pos: [max_x, min_y, z],
                    uv: [1.0, 1.0, uv_z],
                },
                VertexT {
                    pos: [max_x, max_y, z],
                    uv: [1.0, 0.0, uv_z],
                },
            ]);

            indices.extend([0, 1, 2, 2, 1, 3].into_iter().map(|i| i + index_offset));
        }

        Mesh { vertices, indices }
    }
}
