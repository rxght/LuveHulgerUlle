use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    hash::Hash,
    path::Path,
    sync::{Arc, Weak},
};

use tiled::FiniteTileLayer;
use vulkano::{image::sampler::Filter, pipeline::graphics::vertex_input::Vertex, shader::ShaderStages};

use crate::graphics::{
    bindable::{self, IndexBuffer, PushConstant, Texture, VertexBuffer}, camera::Camera, drawable::Drawable, shaders::{frag_texture_array, vert_tile}, Graphics
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
    layer_idx: u32,
    position: [u32; 2],
}

struct TileMapLayer {
    tiles: Vec<Tile>,
}

pub struct TileMap {
    layers: Vec<TileMapLayer>,
    tile_dimensions: [u32; 2],
    map_dimensions: [u32; 2],
    drawables: Vec<TileMapDrawable>,
}

pub struct TileMapDrawable {
    tile_set: Arc<TileSet>,
    vertex_buffer: Arc<VertexBuffer<VertexT>>,
    index_buffer: Arc<IndexBuffer>,
    drawable: Arc<Drawable>,
}

#[derive(Clone)]
pub struct TileSet {
    texture: Arc<Texture>,
    tile_dimensions: [u32; 2],
    atlas_dimensions: [u32; 2],
    tile_count: u32,
}

impl PartialEq for TileSet {
    fn eq(&self, other: &Self) -> bool {
        self.texture.image_view() == other.texture.image_view()
            && self.tile_dimensions == other.tile_dimensions
            && self.atlas_dimensions == other.atlas_dimensions
            && self.tile_count == other.tile_count
    }
}

impl Eq for TileSet {}

impl Hash for TileSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.texture.image_view().hash(state);
        self.tile_dimensions.hash(state);
        self.atlas_dimensions.hash(state);
        self.tile_count.hash(state);
    }
}

impl TileSet {
    pub fn new(gfx: &mut Graphics, set: &tiled::Tileset) -> Self {
        let atlas_texture = Texture::new(gfx, set.source.to_str().unwrap(), Filter::Nearest);
        let atlas_dimensions = atlas_texture.extent_2d();

        TileSet {
            texture: atlas_texture.clone(),
            tile_dimensions: [set.tile_width, set.tile_height],
            atlas_dimensions: [
                atlas_dimensions[0] / set.tile_width,
                atlas_dimensions[1] / set.tile_height,
            ],
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

    pub fn load_tilemap(
        &mut self,
        gfx: &mut Graphics,
        path: impl AsRef<Path>,
    ) -> Result<TileMap, Box<dyn Error>> {
        let mut loader = tiled::Loader::new();
        let map = loader.load_tmx_map(path.as_ref())?;

        let map_dimensions = [map.width, map.height];
        let tile_dimensions = [map.tile_width, map.tile_height];

        let layers = self.load_layers(gfx, map);

        let parsed_map = TileMap {
            layers: layers,
            drawables: vec![],
            tile_dimensions,
            map_dimensions,
        };

        Ok(parsed_map)
    }

    fn load_layers(&mut self, gfx: &mut Graphics, map: tiled::Map) -> Vec<TileMapLayer> {
        let mut result = Vec::new();
        for (layer_idx, layer) in map.layers().enumerate() {
            use tiled::LayerType;
            use tiled::TileLayer;
            match layer.layer_type() {
                LayerType::Tiles(TileLayer::Finite(tile_layer)) => {
                    let parsed_layer = self.load_tile_layer(gfx, layer_idx as u32, tile_layer);
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
        layer_idx: u32,
        tile_layer: FiniteTileLayer<'_>,
    ) -> TileMapLayer {
        let mut parsed_tiles = Vec::new();
        let width = tile_layer.width();
        let height = tile_layer.height();

        for y in 0..height {
            for x in 0..width {
                if let Some(tile) = tile_layer.get_tile(x as i32, y as i32) {
                    let tile_set = self.load_tileset(gfx, tile.get_tileset());
                    if let Some(tile) = self.create_tile([x, y], tile, layer_idx, tile_set) {
                        parsed_tiles.push(tile);
                    }
                }
            }
        }
        TileMapLayer {
            tiles: parsed_tiles,
        }
    }

    fn create_drawables(&self, gfx: &mut Graphics, camera: &Camera, layers: &Vec<TileMapLayer>) -> Vec<TileMapDrawable> {

        let mut groups: HashMap<GroupInfo, Vec<&Tile>> = HashMap::new();

        // sort tiles by which set they use and their animation
        for layer in layers {
            for tile in &layer.tiles {
                let group_info = GroupInfo {
                    tile_set: tile.tile_set.clone(),
                    animation: tile.animation.clone(),
                };
                groups.entry(group_info).or_insert_with(Vec::new).push(tile);
            }
        }

        let mut drawables = Vec::<TileMapDrawable>::new();

        // create one drawable per group
        for (group_info, tiles) in groups {
            let mesh = self.create_mesh(group_info.tile_set.clone(), tiles);
            let drawable = self.create_drawable(gfx, camera, group_info, mesh);
            drawables.push(drawable);
        }

        drawables
    }

    fn create_mesh(&self, tile_set: Arc<TileSet>, tiles: Vec<&Tile>) -> Mesh {
        let mut vertices = vec![];
        let mut indices = vec![];

        for tile in tiles {
            let [x, y] = tile.position;

            let min_x = x as f32 * tile_set.tile_dimensions[0] as f32;
            let max_x = (x + 1) as f32 * tile_set.tile_dimensions[0] as f32;
            let min_y = y as f32 * tile_set.tile_dimensions[1] as f32;
            let max_y = (y + 1) as f32 * tile_set.tile_dimensions[1] as f32;

            println!("remember to change this!!");
            let depth = 1.0 - 0.01 * tile.layer_idx as f32;

            let first_vertex_idx = vertices.len() as u32;

            vertices.extend([
                VertexT {
                    pos: [min_x, min_y, depth],
                    uv: [0.0, 0.0, tile.layer_idx as f32],
                },
                VertexT {
                    pos: [max_x, min_y, depth],
                    uv: [1.0, 0.0, tile.layer_idx as f32],
                },
                VertexT {
                    pos: [min_x, max_y, depth],
                    uv: [0.0, 1.0, tile.layer_idx as f32],
                },
                VertexT {
                    pos: [max_x, max_y, depth],
                    uv: [1.0, 1.0, tile.layer_idx as f32],
                },
            ]);

            indices.extend([0, 1, 2, 2, 1, 3].into_iter().map(|p| first_vertex_idx + p));
        }

        Mesh {
            vertices,
            indices,
        }
    }

    fn create_drawable(&self, gfx: &mut Graphics, camera: &Camera, group_info: GroupInfo, mesh: Mesh) -> TileMapDrawable {
        
        let animation = &group_info.animation;
        let frame_data = FrameData {
            
        };

        let push_constants = PushConstant::new(0, frame_data, ShaderStages::VERTEX);

        let index_count = mesh.indices.len() as u32;

        let tile_set = group_info.tile_set;
        let vertex_buffer = bindable::VertexBuffer::new(gfx, mesh.vertices);
        let index_buffer: Arc<IndexBuffer> = bindable::IndexBuffer::new(gfx, mesh.indices);

        let drawable = Drawable::new(
            gfx,
            vec![
                bindable::TextureBinding::new(group_info.tile_set.texture.clone(), 1),
                vertex_buffer.clone(),
                index_buffer.clone(),
                push_constants,
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
        );

        TileMapDrawable{
            tile_set,
            vertex_buffer,
            index_buffer,
            drawable,
        }
    }

    pub fn load_tileset(&mut self, gfx: &mut Graphics, set: &tiled::Tileset) -> Arc<TileSet> {
        let key = set.source.as_os_str();
        if let Some(arc) = self.loaded_tilesets.get(key).and_then(Weak::upgrade) {
            return arc;
        }
        let arc = Arc::new(TileSet::new(gfx, &set));
        self.loaded_tilesets
            .insert(key.to_os_string(), Arc::downgrade(&arc));
        return arc;
    }

    pub fn create_tile(
        &mut self,
        position: [u32; 2],
        tile: tiled::LayerTile,
        layer_idx: u32,
        tile_set: Arc<TileSet>,
    ) -> Option<Tile> {
        let set_tile = tile.get_tile()?;
        let tile_type = self.get_tile_type(&set_tile);
        let animation = self.get_animation(&set_tile);

        Some(Tile {
            tile_type,
            animation,
            tile_set,
            tile_id: tile.id() as u32,
            layer_idx,
            position,
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
        let shared_frames: Arc<[AnimationFrame]> = Arc::from(parsed_frames.clone().into_boxed_slice());
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
