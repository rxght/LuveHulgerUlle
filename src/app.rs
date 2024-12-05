use crate::drawables::tiles::TileMap;
use crate::drawables::tiles::TileMapLoader;
use crate::graphics::camera::Camera;
use crate::graphics::Graphics;
use crate::input::Input;
use character::CharacterController;
use egui_winit_vulkano::egui::epaint::Shadow;
use egui_winit_vulkano::egui::Color32;
use egui_winit_vulkano::egui::Frame;
use egui_winit_vulkano::egui::Stroke;
use egui_winit_vulkano::egui::Window;
use std::sync::Arc;
use std::time::Duration;

mod character;

pub struct App {
    tile_map_loader: TileMapLoader,
    tile_map: Arc<TileMap>,
    player: CharacterController,
    camera: Camera,
}

impl App {
    pub fn new(gfx: &mut Graphics) -> Self {
        let camera = Camera::new(gfx, [0.0, 0.0], 1.0, 0.0);

        let mut loader = TileMapLoader::new();
        let tile_map = loader.load(gfx, "assets/tilemaps/bigmap.tmx", &camera);

        let player = CharacterController::new(gfx, &camera);

        Self {
            tile_map_loader: loader,
            tile_map,
            player,
            camera,
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

        let ctx = gfx.gui().context();

        Window::new("Performance")
            .resizable(false)
            .frame(
                Frame::none()
                    .inner_margin(3.0)
                    .fill(Color32::from_black_alpha(170))
                    .stroke(Stroke::new(2.0, Color32::from_black_alpha(180)))
                    .shadow(Shadow {
                        extrusion: 5.0,
                        color: Color32::from_black_alpha(100),
                    })
                    .rounding(5.0),
            )
            .show(&ctx, |ui| {
                let frame_time = delta_time.as_secs_f64();
                ui.label(format!("frame time: {:.1} ms", frame_time * 1000.0));
                ui.label(format!("fps: {:.0}", 1.0 / frame_time));
            });

        self.tile_map.draw(gfx);
        self.player.draw(gfx);
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
