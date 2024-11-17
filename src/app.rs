use winit::dpi::PhysicalSize;

use crate::drawables::tiles::TileMap;
use crate::drawables::tiles::TileSet;
use crate::drawables::Square;
use crate::drawables::SquareDesc;
use crate::graphics::camera::Camera;
use crate::graphics::Graphics;
use crate::input::Input;
use crate::ui::ui_square::UiSquare;
use crate::ui::Ui;
use crate::ui::UiLayout;
use crate::ui::UiScene;
use crate::ui::UiUnit;
use std::sync::Arc;

pub struct App {
    input: Arc<Input>,
    tile_set: Arc<TileSet>,
    tile_map: TileMap,
    last_frame_change: std::time::Instant,
    camera: Camera,
    main_ui_scene: Arc<UiScene>,
    ui: Arc<Ui>,
}

impl App {
    pub fn new(gfx: &mut Graphics, input: Arc<Input>, ui: Arc<Ui>) -> Self {
        let tile_set = TileSet::new(gfx, "assets/textures/tileset_1.png", 16);

        let camera = Camera::new(gfx, [0.0, 0.0], 1.0, 0.0);

        let tile_map = TileMap::new(gfx, "assets/tilemaps/bigmap.tmx", tile_set.clone(), &camera);

        tile_map
            .layers
            .iter()
            .for_each(|p| gfx.register_drawable(p));

        let main_ui_scene = UiScene(vec![UiSquare::new(
            gfx,
            [0.7, 0.72, 0.75, 1.0],
            UiLayout {
                x: UiUnit::Combined(50.0, -500.0),
                y: UiUnit::Percentage(0.0),
                width: UiUnit::Pixels(1000.0),
                height: UiUnit::Pixels(64.0),
            },
        )]);

        let main_ui_scene = Arc::new(main_ui_scene);

        gfx.get_window().set_inner_size(PhysicalSize {
            width: 1200,
            height: 800,
        });
        ui.set_scene(gfx, main_ui_scene.clone());

        let mut tile_cursor = Square::new(
            gfx,
            SquareDesc {
                pos: [0, 0],
                width: 64,
                height: 64,
            },
            [1.0, 1.0, 1.0, 0.3],
        );
        gfx.register_drawable(&mut tile_cursor.drawable_entry);

        Self {
            input,
            tile_set,
            tile_map,
            last_frame_change: std::time::Instant::now(),
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
    }
}
