use character::CharacterController;
use hud::Hotbar;
use winit::dpi::PhysicalSize;

use crate::drawables::tiles::TileMap;
use crate::drawables::tiles::TileMapLoader;
use crate::graphics::camera::Camera;
use crate::graphics::Graphics;
use crate::input::Input;
use std::sync::Arc;
use std::time::Duration;

mod character;
mod hud;

pub struct App {
    tile_map_loader: TileMapLoader,
    tile_map: Arc<TileMap>,
    player: CharacterController,
    camera: Camera,
    hotbar: Hotbar,
}

impl App {
    pub fn new(gfx: &mut Graphics) -> Self {
        let camera = Camera::new(gfx, [0.0, 0.0], 1.0, 0.0);

        let mut loader = TileMapLoader::new();
        let tile_map = loader.load(gfx, "assets/tilemaps/bigmap.tmx", &camera);

        let hotbar = Hotbar::new(gfx);

        gfx.get_window().set_inner_size(PhysicalSize {
            width: 600,
            height: 400,
        });

        let player = CharacterController::new(gfx, &camera);

        Self {
            tile_map_loader: loader,
            tile_map,
            player,
            camera,
            hotbar,
        }
    }

    pub fn resize_callback(&self, gfx: &mut Graphics) {
        gfx.recreate_swapchain();
    }

    pub fn run(&mut self, gfx: &mut Graphics, input: &Input, delta_time: Duration) {
        self.player.update(input, delta_time);
        self.camera.position = *self.player.position();
        self.editor_camera_movement(input);
        self.tile_map_loader.update();

        // hotbar functionality
        if input.keyboard.is_key_pressed(57) {
            let slot = (self.hotbar.selected_slot() + 1) % 9;
            self.hotbar.set_selected_slot(slot);
        }

        self.tile_map.draw(gfx);
        self.player.draw(gfx);
        self.hotbar.draw(gfx);
    }

    fn editor_camera_movement(&mut self, input: &Input) {
        if input.keyboard.is_key_pressed(12) {
            self.camera.zoom = (self.camera.zoom + 1.0).round();
        }
        if self.camera.zoom > 1.5 && input.keyboard.is_key_pressed(53) {
            self.camera.zoom = (self.camera.zoom - 1.0).round();
        }
        self.camera.update_buffer();
    }
}
