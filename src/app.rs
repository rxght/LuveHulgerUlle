use winit::dpi::PhysicalSize;

use crate::drawables::tiles::DynamicTile;
use crate::drawables::tiles::TileMap;
use crate::drawables::tiles::TileMapLoader;
use crate::graphics::bindable::Texture;
use crate::graphics::camera::Camera;
use crate::graphics::Graphics;
use crate::input::Input;
use crate::ui::Ui;
use crate::ui::UiScene;
use std::sync::Arc;

pub struct App {
    input: Arc<Input>,
    tile_map_loader: TileMapLoader,
    tile_map: Arc<TileMap>,
    dynamic_tile: DynamicTile,
    last_frame_change: std::time::Instant,
    camera: Camera,
    main_ui_scene: Arc<UiScene>,
    ui: Arc<Ui>,
}

impl App {
    pub fn new(gfx: &mut Graphics, input: Arc<Input>, ui: Arc<Ui>) -> Self {
        let camera = Camera::new(gfx, [0.0, 0.0], 1.0, 0.0);

        let mut loader = TileMapLoader::new(gfx);
        let tile_map = loader.load(gfx, "assets\\tilemaps\\bigmap.tmx", &camera);

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

        let character_texture =
            Texture::new_array(gfx, "assets\\textures\\character_test.png", [32, 48]);
        let character = DynamicTile::new(gfx, character_texture, &camera);

        gfx.register_drawable(&character.drawable);

        Self {
            input,
            tile_map_loader: loader,
            tile_map,
            last_frame_change: std::time::Instant::now(),
            dynamic_tile: character,
            camera,
            main_ui_scene,
            ui,
        }
    }

    pub fn resize_callback(&self, gfx: &mut Graphics) {
        gfx.recreate_swapchain();
    }

    pub fn run(&mut self, _gfx: &Graphics) {
        self.editor_camera_movement();
        self.tile_map_loader.update();
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
        if self.input.keyboard.is_key_pressed(57) {
            self.camera.zoom = self.camera.zoom.ceil();
        }
        self.camera.update_buffer();

        if self.input.keyboard.is_key_held(17).is_some() {
            self.camera.position[1] -= 0.2;
        }
        if self.input.keyboard.is_key_held(30).is_some() {
            self.camera.position[0] -= 0.2;
        }
        if self.input.keyboard.is_key_held(31).is_some() {
            self.camera.position[1] += 0.2;
        }
        if self.input.keyboard.is_key_held(32).is_some() {
            self.camera.position[0] += 0.2;
        }
    }
}
