use crate::drawables::tiles::TileMap;
use crate::drawables::tiles::TileMapLoader;
use crate::graphics::camera::Camera;
use crate::graphics::Graphics;
use crate::input::Input;
use character::CharacterController;
use egui_winit_vulkano::egui::epaint::Shadow;
use egui_winit_vulkano::egui::Color32;
use egui_winit_vulkano::egui::Frame;
use egui_winit_vulkano::egui::Slider;
use egui_winit_vulkano::egui::Stroke;
use egui_winit_vulkano::egui::Window;
use hud::Healthbar;
use hud::Hotbar;
use std::time::Duration;

mod character;
mod hud;
mod item;
mod window;

pub struct App {
    tile_map_loader: TileMapLoader,
    tile_map: TileMap,
    player: CharacterController,
    camera: Camera,
    hotbar: Hotbar,
    healthbar: Healthbar,
    health_level: u32,
    hotbar_slot: u32,
}

impl App {
    pub fn new(gfx: &mut Graphics) -> Self {
        let camera = Camera::new(gfx, [0.0, 0.0], 1.0, 0.0);

        let mut loader = TileMapLoader::new();
        let tile_map = loader.load(gfx, "assets/tilemaps/ollemap.tmx", &camera).unwrap();

        let player = CharacterController::new(gfx, &camera);

        Self {
            tile_map_loader: loader,
            tile_map,
            player,
            camera,
            hotbar: Hotbar::new(gfx),
            healthbar: Healthbar::new(gfx),
            health_level: 20,
            hotbar_slot: 1,
        }
    }

    pub fn run(&mut self, gfx: &mut Graphics, input: &Input, delta_time: Duration) {
        self.player.update(input, delta_time);
        self.camera.position = *self.player.position();
        self.editor_camera_movement(input);
        //self.tile_map_loader.update();
        self.debug_window(gfx, delta_time);
        self.tile_map.draw_all_layers(gfx);
        self.player.draw(gfx);
        self.hotbar.draw(gfx, self.hotbar_slot - 1, 4.0);
        self.healthbar.draw(gfx, self.health_level, 4.0);
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

    fn debug_window(&mut self, gfx: &mut Graphics, delta_time: Duration) {
        let window_size = gfx.get_window().inner_size();
        let ctx = gfx.gui().context();
        Window::new("Debug Window")
            .resizable(false)
            .frame(
                Frame::none()
                    .inner_margin(3.0)
                    .fill(Color32::from_black_alpha(170))
                    .stroke(Stroke::new(2.0, Color32::from_black_alpha(220)))
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
                ui.label(format!(
                    "window size: {}x{}",
                    window_size.width, window_size.height
                ));
                ui.add(Slider::new(&mut self.health_level, 0..=20).text("health"));
                ui.add(Slider::new(&mut self.hotbar_slot, 1..=9).text("hotbar slot"));
            });
    }
}
