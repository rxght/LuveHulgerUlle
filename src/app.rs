use character::CharacterController;
use winit::dpi::PhysicalSize;

use crate::drawables::tiles::TileMap;
use crate::drawables::tiles::TileMapLoader;
use crate::graphics::camera::Camera;
use crate::graphics::Graphics;
use crate::input::Input;
use crate::ui::Ui;
use crate::ui::UiScene;
use std::sync::Arc;
use std::time::Duration;

mod character;

pub struct App {
    input: Arc<Input>,
    tile_map_loader: TileMapLoader,
    tile_map: Arc<TileMap>,
    player: CharacterController,
    last_frame_change: std::time::Instant,
    camera: Camera,
    main_ui_scene: Arc<UiScene>,
    ui: Arc<Ui>,
}

impl App {
    pub fn new(gfx: &mut Graphics, input: Arc<Input>, ui: Arc<Ui>) -> Self {
        let camera = Camera::new(gfx, [0.0, 0.0], 1.0, 0.0);

        let mut loader = TileMapLoader::new();
        let tile_map = loader.load(gfx, "assets\\tilemaps\\animated.tmx", &camera);

        tile_map
            .layers
            .iter()
            .for_each(|p| gfx.register_drawable(p));

        let main_ui_scene = UiScene(vec![]);

        let main_ui_scene = Arc::new(main_ui_scene);

        gfx.get_window().set_inner_size(PhysicalSize {
            width: 1200,
            height: 800,
        });
        ui.set_scene(gfx, main_ui_scene.clone());

        let player = CharacterController::new(gfx, &camera);
        gfx.register_drawable(&player.tile_renderer.drawable);

        Self {
            input,
            tile_map_loader: loader,
            tile_map,
            last_frame_change: std::time::Instant::now(),
            player,
            camera,
            main_ui_scene,
            ui,
        }
    }

    pub fn resize_callback(&self, gfx: &mut Graphics) {
        gfx.recreate_swapchain();
    }

    pub fn run(&mut self, _gfx: &Graphics, delta_time: Duration) {
        self.player.update(&self.input, delta_time);
        self.camera.position = *self.player.position();
        self.editor_camera_movement();
        self.tile_map_loader.update();
    }

    fn editor_camera_movement(&mut self) {
        self.camera.zoom *= 1.0 + self.input.mouse.scroll_wheel_movement.get() / 10.0;
        if self.input.keyboard.is_key_pressed(57) {
            self.camera.zoom = self.camera.zoom.ceil();
        }
        self.camera.update_buffer();
    }
}
