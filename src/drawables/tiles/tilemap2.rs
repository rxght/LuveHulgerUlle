use std::{
    cell::{Cell, Ref, RefCell, RefMut, UnsafeCell},
    collections::HashMap,
    error::Error,
    ffi::OsString,
    hash::Hash,
    path::Path,
    sync::{Arc, Weak},
};

use egui_winit_vulkano::egui::mutex::RwLock;
use tiled::FiniteTileLayer;
use vulkano::pipeline::graphics::vertex_input::Vertex;

use crate::graphics::{
    bindable::{IndexBuffer, Texture, TextureBinding, UniformBuffer, VertexBufferMut},
    camera::CameraUbo,
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

#[derive(Clone)]
pub struct Tile {
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

    pub fn tiles(&self) -> &[Option<Tile>] {
        &self.tiles
    }

    pub fn tiles_mut(&mut self) -> &mut Vec<Option<Tile>> {
        &mut self.tiles
    }
}

pub struct TileMap {
    position_offset: Cell<[f32; 2]>,
    scale: Cell<f32>,
    layers: RefCell<Vec<TileMapLayer>>,
    tile_dimensions: [u32; 2],
    map_dimensions: [u32; 2],

    camera: RefCell<Arc<UniformBuffer<CameraUbo>>>,
    up_to_date: Cell<bool>,
    drawable: UnsafeCell<TileMapDrawable>,
}

impl TileMap {
    pub fn draw(&self, gfx: &mut Graphics) {
        unsafe { self.drawable.get().as_ref().unwrap().draw(gfx) };
    }

    pub fn dimensions(&self) -> [u32; 2] {
        self.map_dimensions
    }

    pub fn tile_dimensions(&self) -> [u32; 2] {
        self.tile_dimensions
    }

    pub fn layers(&self) -> Ref<'_, Vec<TileMapLayer>> {
        self.layers.borrow()
    }

    pub fn layers_mut(&self) -> RefMut<Vec<TileMapLayer>> {
        self.up_to_date.set(false);
        self.layers.borrow_mut()
    }

    pub fn scale(&self) -> f32 {
        self.scale.get()
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale.set(scale);
        self.up_to_date.set(false);
    }

    pub fn position(&self) -> [f32; 2] {
        self.position_offset.get()
    }

    pub fn set_position(&mut self, position: [f32; 2]) {
        self.position_offset.set(position);
        self.up_to_date.set(false);
    }

    fn update(&self, gfx: &mut Graphics) {
        if !self.up_to_date.get() {
            self.up_to_date.set(true);
            let drawable = unsafe { self.drawable.get().as_mut().unwrap() };
            *drawable = TileMapDrawable::new(
                gfx,
                self.position_offset.get(),
                self.scale.get(),
                &self.layers.borrow(),
                self.map_dimensions,
                self.camera.borrow().clone(),
            );
        }
    }
}

pub struct TileSet {
    texture: Arc<Texture>,
    descriptor: tiled::Tileset,
    animation_mapping: RwLock<AnimationMapping>,
}

impl PartialEq for TileSet {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor.source == other.descriptor.source
    }
}

impl Eq for TileSet {}

impl Hash for TileSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.descriptor.source.hash(state);
    }
}

impl TileSet {
    fn new(gfx: &mut Graphics, set: tiled::Tileset) -> Self {
        let tile_dimensions = [set.tile_width, set.tile_height];
        let image_path = set.image.as_ref().unwrap().source.to_str().unwrap();
        let atlas_texture = Texture::new_array(gfx, image_path, tile_dimensions);

        TileSet {
            texture: atlas_texture.clone(),
            animation_mapping: RwLock::new(AnimationMapping::from(&set)),
            descriptor: set,
        }
    }
}

pub struct TileMapLoader {
    loaded_tilesets: HashMap<OsString, Weak<TileSet>>,
    loaded_tilemaps: HashMap<OsString, Weak<TileMap>>,
}

impl TileMapLoader {
    pub fn new() -> Self {
        Self {
            loaded_tilesets: HashMap::new(),
            loaded_tilemaps: HashMap::new(),
        }
    }

    fn update_tileset_mappings(&mut self) {
        for tile_set in self.loaded_tilesets.values().filter_map(|f| f.upgrade()) {
            let mut updated_ids: Vec<(u32, f32)> = Vec::new();

            let read_mapping = tile_set.animation_mapping.read();
            let new_mapping = AnimationMapping::from(&tile_set.descriptor);

            for (key, value) in new_mapping.0.iter() {
                match read_mapping.0.get(key) {
                    Some(old_value) => {
                        if old_value != value {
                            updated_ids.push((*key, *value));
                        }
                    }
                    None => {
                        updated_ids.push((*key, *value));
                    }
                }
            }

            if updated_ids.is_empty() {
                continue;
            }

            // this drop is necessary to avoid a deadlock
            drop(read_mapping);
            let mut write_mapping = tile_set.animation_mapping.write();
            for (updated_id, value) in updated_ids.iter().cloned() {
                write_mapping.0.insert(updated_id, value);
            }

            for tile_map in self.loaded_tilemaps.values().filter_map(|f| f.upgrade()) {
                let drawable_groups = unsafe { &tile_map.drawable.get().as_ref().unwrap().groups };
                if let Some(drawable) = drawable_groups.get(&tile_set) {
                    drawable.vertex_buffer.write(|vertices| {
                        for (idx, positioned_tile) in drawable.source_tiles.iter().enumerate() {
                            let tile = &positioned_tile.tile;
    
                            for (updated_id, uv_z) in updated_ids.iter() {
                                if tile.tile_id == *updated_id {
                                    vertices[idx].uv[2] = *uv_z;
                                }
                            }
                        }
                    });
                }
            }
        }
    }

    fn update_tilemap_drawables(&mut self, gfx: &mut Graphics) {
        self.loaded_tilemaps
            .values()
            .filter_map(|f| f.upgrade())
            .for_each(|tile_map| tile_map.update(gfx));
    }

    pub fn update(&mut self, gfx: &mut Graphics) {
        self.loaded_tilesets
            .retain(|_, tile_set| tile_set.upgrade().is_some());
        self.loaded_tilemaps
            .retain(|_, tile_set| tile_set.upgrade().is_some());

        self.update_tileset_mappings();
        self.update_tilemap_drawables(gfx);
    }

    pub fn load(
        &mut self,
        gfx: &mut Graphics,
        path: impl AsRef<Path>,
        position: [f32; 2],
        scale: f32,
        camera: Arc<UniformBuffer<CameraUbo>>,
    ) -> Result<Arc<TileMap>, Box<dyn Error>> {
        let mut loader = tiled::Loader::new();
        let map = loader.load_tmx_map(path.as_ref())?;

        let map_dimensions = [map.width, map.height];
        let tile_dimensions = [map.tile_width, map.tile_height];

        let layers = self.load_layers(gfx, map);

        let drawable = TileMapDrawable::new(
            gfx,
            position,
            scale,
            &layers,
            map_dimensions,
            camera.clone(),
        );

        let parsed_map = Arc::new(TileMap {
            position_offset: Cell::new(position),
            scale: Cell::new(scale),
            layers: RefCell::new(layers),
            tile_dimensions,
            map_dimensions,
            drawable: UnsafeCell::new(drawable),
            camera: RefCell::new(camera),
            up_to_date: Cell::new(true),
        });

        self.loaded_tilemaps.insert(
            path.as_ref().as_os_str().to_os_string(),
            Arc::downgrade(&parsed_map),
        );
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
                    let tile_set = self.load_tileset(gfx, tile.get_tileset().clone());
                    parsed_tiles.push(Some(Tile {
                        tile_set,
                        tile_id: tile.id(),
                    }));
                    continue;
                }
                parsed_tiles.push(None);
            }
        }
        TileMapLayer {
            tiles: parsed_tiles,
        }
    }

    fn load_tileset(&mut self, gfx: &mut Graphics, set: tiled::Tileset) -> Arc<TileSet> {
        let key = set.source.as_os_str().to_os_string();
        if let Some(arc) = self.loaded_tilesets.get(&key).and_then(Weak::upgrade) {
            return arc;
        }
        let arc = Arc::new(TileSet::new(gfx, set));
        self.loaded_tilesets
            .insert(key.to_os_string(), Arc::downgrade(&arc));
        return arc;
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct AnimationFrame {
    /// milliseconds
    pub duration: u32,
    pub frame_offset: i32,
}

struct TileMapDrawable {
    groups: HashMap<Arc<TileSet>, TileGroupDrawable>,
}

struct TileGroupDrawable {
    // the source tiles are cached for fast modifications
    source_tiles: Vec<PositionedTile>,

    drawable: Arc<Drawable>,
    vertex_buffer: Arc<VertexBufferMut<VertexT>>,
    index_buffer: Arc<IndexBuffer>,
    texture_binding: Arc<TextureBinding>,
}

struct PositionedTile {
    tile: Tile,
    position: [u32; 3],
}

#[repr(transparent)]
struct AnimationMapping(HashMap<u32, f32>);

impl From<&tiled::Tileset> for AnimationMapping {
    fn from(value: &tiled::Tileset) -> Self {
        let mut mapping = HashMap::new();

        for (id, tile) in value.tiles() {
            if let Some(animation) = &tile.animation {
                let animation_length = animation.iter().map(|frame| frame.duration).sum::<u32>();
                let now = std::time::SystemTime::now();
                let mut sub_duration = now
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    % animation_length as u128;
                let mut frame_idx = 0;
                while sub_duration > animation[frame_idx].duration as u128 {
                    sub_duration -= animation[frame_idx].duration as u128;
                    frame_idx += 1;
                }
                mapping.insert(id, animation[frame_idx].tile_id as f32);
            }
        }

        AnimationMapping(mapping)
    }
}

impl AnimationMapping {
    pub fn get_texture_z(&self, tile_id: u32) -> f32 {
        match self.0.get(&tile_id) {
            Some(mapped_id) => *mapped_id,
            None => tile_id as f32,
        }
    }
}

impl TileMapDrawable {
    pub fn new(
        gfx: &mut Graphics,
        position: [f32; 2],
        scale: f32,
        layers: &[TileMapLayer],
        map_dimensions: [u32; 2],
        camera: Arc<UniformBuffer<CameraUbo>>,
    ) -> Self {
        // gruppera all tiles som använder samma tileset
        let mut grouped_tiles: HashMap<Arc<TileSet>, Vec<PositionedTile>> = HashMap::new();
        for (layer_idx, layer) in layers.iter().enumerate() {
            for (idx, tile) in layer.tiles.iter().enumerate() {
                if let Some(tile) = tile {
                    let x = idx as u32 % map_dimensions[0];
                    let y = idx as u32 / map_dimensions[0];
                    let position = [x, y, layer_idx as u32];
                    let positioned_tile = PositionedTile {
                        tile: tile.clone(),
                        position,
                    };
                    grouped_tiles
                        .entry(tile.tile_set.clone())
                        .or_insert_with(Vec::new)
                        .push(positioned_tile);
                }
            }
        }

        let mut drawable_groups = HashMap::new();
        for (tile_set, tiles) in grouped_tiles {
            let mesh = Self::create_mesh(&tiles, &tile_set.descriptor, position, scale);
            let drawable =
                Self::create_drawable(gfx, tile_set.clone(), mesh, tiles, camera.clone());
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
        tile_set: Arc<TileSet>,
        mesh: Mesh,
        source_tiles: Vec<PositionedTile>,
        camera: Arc<UniformBuffer<CameraUbo>>,
    ) -> TileGroupDrawable {
        let vertex_buffer = VertexBufferMut::new(gfx, mesh.vertices);
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
                    UniformBufferBinding::new(camera, 2),
                ]
            },
            index_count,
        );

        TileGroupDrawable {
            source_tiles,
            drawable,
            vertex_buffer,
            index_buffer,
            texture_binding,
        }
    }

    fn create_mesh<'a>(
        tiles: &[PositionedTile],
        tile_set: &tiled::Tileset,
        position: [f32; 2],
        scale: f32,
    ) -> Mesh {
        let mut vertices = Vec::with_capacity(tiles.len() * 4);
        let mut indices = Vec::with_capacity(tiles.len() * 6);

        let [width, height] = [tile_set.tile_width, tile_set.tile_height];
        let [x_offset, y_offset] = [position[0] * width as f32, position[1] * height as f32];

        for positioned_tile in tiles {
            let [x, y, z] = positioned_tile.position;
            let tile = &positioned_tile.tile;

            let min_x = ((x * width) as f32 + x_offset) * scale;
            let max_x = (((x + 1) * width) as f32 + x_offset) * scale;
            let min_y = -(((y + 1) * height) as f32 + y_offset) * scale;
            let max_y = -((y * height) as f32 + y_offset) * scale;

            let z = -1.0 + 0.01 * z as f32;

            let set_tile = tile_set.get_tile(tile.tile_id).unwrap();

            let now = std::time::SystemTime::now();

            let uv_z = match &set_tile.animation {
                Some(animation) => {
                    let animation_length =
                        animation.iter().map(|frame| frame.duration).sum::<u32>();
                    let mut sub_duration = now
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                        % animation_length as u128;
                    let mut frame_idx = 0;
                    while sub_duration > animation[frame_idx].duration as u128 {
                        sub_duration -= animation[frame_idx].duration as u128;
                        frame_idx += 1;
                    }
                    animation[frame_idx].tile_id as f32
                }
                None => tile.tile_id as f32,
            };

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
