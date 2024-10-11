use crate::drawables::tiles::AnimatedTile;
use crate::drawables::tiles::AnimatedTileDesc;
use crate::drawables::tiles::StaticTileGroup;
use crate::drawables::tiles::TileSet;
use crate::graphics::camera::Camera;
use crate::graphics::Graphics;
use crate::input::Input;
use std::sync::Arc;

mod ui;

pub struct App {
    input: Arc<Input>,
    tile_set: Arc<TileSet>,
    tile_map: StaticTileGroup,
    animated_tiles: Vec<AnimatedTile>,
    last_frame_change: std::time::Instant,
    camera: Camera,
}

impl App {
    pub fn new(gfx: &mut Graphics, input: Arc<Input>) -> Self {
        let tile_set = TileSet::new(gfx, "textures/tile_sheet2.png", 16);

        let camera = Camera::new(gfx, [0.0, 0.0], 1.0, 0.0);

        let tile_map = StaticTileGroup::new(
            gfx,
            tile_set.clone(),
            [8, 5],
            [
                5, 2, 3, 4, 23, 0, 0, 0, 0, 7, 8, 9, 0, 23, 0, 0, 0, 12, 13, 14, 6, 0, 5, 0, 0, 23,
                18, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5,
            ]
            .into_iter()
            .map(|elem| Some(elem))
            .collect(),
            64.0,
            &camera,
        );

        let animated_tile_set = TileSet::new(gfx, "textures/3x3_vatten_test.png", 16);
        let animated_tiles = vec![
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [1, 0],
                    first_sprite_idx: 0,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [2, 0],
                    first_sprite_idx: 1,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [3, 0],
                    first_sprite_idx: 2,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [1, 1],
                    first_sprite_idx: 24,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [2, 1],
                    first_sprite_idx: 25,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [3, 1],
                    first_sprite_idx: 26,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [1, 2],
                    first_sprite_idx: 48,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [2, 2],
                    first_sprite_idx: 49,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
            AnimatedTile::new(
                gfx,
                animated_tile_set.clone(),
                AnimatedTileDesc {
                    tile_position: [3, 2],
                    first_sprite_idx: 50,
                    frame_stride: 3,
                },
                64.0,
                &camera,
            ),
        ];

        Self {
            input: input,
            tile_set: tile_set,
            tile_map: tile_map,
            animated_tiles: animated_tiles,
            last_frame_change: std::time::Instant::now(),
            camera: camera,
        }
    }

    pub fn resize_callback(&self, gfx: &mut Graphics) {
        gfx.recreate_swapchain();
    }

    pub fn run(&mut self, _gfx: &Graphics) {
        self.editor_camera_movement();

        if self.last_frame_change.elapsed().as_millis() > 150 {
            self.last_frame_change = std::time::Instant::now();
            for tile in &self.animated_tiles {
                tile.data.access_data(|data| {
                    data.frame_offset = (data.frame_offset + 1) % 7; 
                });
            }
        }
    }

    fn editor_camera_movement(&mut self) {
        self.camera.zoom *= 1.0 + self.input.mouse.scroll_wheel_movement.get() / 10.0;
        if self.input.keyboard.is_key_held(56).is_some() {
            if self.input.mouse.is_button_held(1).is_some() {
                let mouse_movement = self.input.mouse.mouse_movement.get();
                self.camera.position[0] -= mouse_movement.x as f32 / self.camera.zoom;
                self.camera.position[1] -= mouse_movement.y as f32 / self.camera.zoom;
            }
        }
        self.camera.update_buffer();
    }
}
