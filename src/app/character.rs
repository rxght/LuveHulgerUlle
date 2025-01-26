use std::{cmp::Ordering, f32::consts::SQRT_2, path::Path, sync::Arc, time::Duration};

use crate::{
    drawables::tiles::DynamicTile,
    graphics::{bindable::Texture, camera::Camera, Graphics},
    input::Input,
};

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Clone, Copy)]
enum CharacterState {
    Idle,
    Walking,
}

pub struct CharacterController {
    textures: Vec<Arc<Texture>>,
    pub tile_renderer: DynamicTile,
    state: CharacterState,
    direction: Direction,
    last_frame_time: std::time::Instant,
    frame_idx: u32,
    position: [f32; 2],
}

impl CharacterController {
    pub fn new(gfx: &Graphics, camera: &Camera) -> Self {
        let folder = Path::new("assets/textures/characters/test_character/");
        let dimensions = [32, 48];

        let textures = vec![
            Texture::new_array(gfx, folder.join("up.png").to_str().unwrap(), dimensions),
            Texture::new_array(gfx, folder.join("down.png").to_str().unwrap(), dimensions),
            Texture::new_array(gfx, folder.join("left.png").to_str().unwrap(), dimensions),
            Texture::new_array(gfx, folder.join("right.png").to_str().unwrap(), dimensions),
            Texture::new_array(gfx, folder.join("upleft.png").to_str().unwrap(), dimensions),
            Texture::new_array(
                gfx,
                folder.join("upright.png").to_str().unwrap(),
                dimensions,
            ),
            Texture::new_array(
                gfx,
                folder.join("downleft.png").to_str().unwrap(),
                dimensions,
            ),
            Texture::new_array(
                gfx,
                folder.join("downright.png").to_str().unwrap(),
                dimensions,
            ),
            Texture::new_array(gfx, folder.join("idle.png").to_str().unwrap(), dimensions),
        ];

        let tile_renderer = DynamicTile::new(gfx, textures[8].clone(), camera);
        tile_renderer.set_layer(1);

        Self {
            textures,
            tile_renderer,
            state: CharacterState::Idle,
            direction: Direction::Down,
            last_frame_time: std::time::Instant::now(),
            frame_idx: 1,
            position: [0.0, 0.0],
        }
    }

    pub fn position(&mut self) -> &mut [f32; 2] {
        &mut self.position
    }

    pub fn update(&mut self, input: &Input, delta_time: Duration) {
        let mut x_movement = 0;
        let mut y_movement = 0;
        if input.keyboard.is_key_held(17).is_some() {
            y_movement += 1;
        }
        if input.keyboard.is_key_held(30).is_some() {
            x_movement -= 1;
        }
        if input.keyboard.is_key_held(31).is_some() {
            y_movement -= 1;
        }
        if input.keyboard.is_key_held(32).is_some() {
            x_movement += 1;
        }

        let is_moving;

        if x_movement == 0 && y_movement == 0 {
            is_moving = false;
        } else {
            is_moving = true;
            self.direction = match (x_movement.cmp(&0), y_movement.cmp(&0)) {
                (Ordering::Equal, Ordering::Greater) => Direction::Up,
                (Ordering::Equal, Ordering::Less) => Direction::Down,
                (Ordering::Less, Ordering::Equal) => Direction::Left,
                (Ordering::Greater, Ordering::Equal) => Direction::Right,
                (Ordering::Less, Ordering::Greater) => Direction::UpLeft,
                (Ordering::Greater, Ordering::Greater) => Direction::UpRight,
                (Ordering::Less, Ordering::Less) => Direction::DownLeft,
                (Ordering::Greater, Ordering::Less) => Direction::DownRight,
                (Ordering::Equal, Ordering::Equal) => unreachable!(),
            };
        }

        let texture_idx;
        let frame_idx;
        let frame_interval = 80;

        if is_moving {
            match self.state {
                CharacterState::Idle => {
                    self.last_frame_time = std::time::Instant::now();
                    self.state = CharacterState::Walking;
                    frame_idx = 0;
                }
                CharacterState::Walking => {
                    if self.last_frame_time.elapsed().as_millis() > frame_interval {
                        self.last_frame_time = std::time::Instant::now();
                        frame_idx = (self.frame_idx + 1) % 8;
                    } else {
                        frame_idx = self.frame_idx;
                    }
                }
            };
            texture_idx = self.direction as usize;
        } else {
            self.state = CharacterState::Idle;
            frame_idx = match self.direction {
                Direction::Up => 5,
                Direction::Down => 1,
                Direction::Left => 3,
                Direction::Right => 7,
                Direction::UpLeft => 4,
                Direction::UpRight => 6,
                Direction::DownLeft => 2,
                Direction::DownRight => 0,
            };
            texture_idx = 8;
        }

        self.frame_idx = frame_idx;

        let movement_speed = 120.0;
        let movement_amount;

        if x_movement != 0 && y_movement != 0 {
            movement_amount = delta_time.as_secs_f32() * movement_speed / SQRT_2;
        } else {
            movement_amount = delta_time.as_secs_f32() * movement_speed;
        }

        let [x_pos, y_pos] = &mut self.position;

        *x_pos += x_movement as f32 * movement_amount;
        *y_pos += y_movement as f32 * movement_amount;

        let [width, height] = self.tile_renderer.dimensions();

        self.tile_renderer.object_data().access_data(|data| {
            data.layer_idx = frame_idx as f32;
            data.position = [*x_pos - width * 0.5, *y_pos + height * 0.2];
        });

        self.tile_renderer
            .set_texture(self.textures[texture_idx].clone());
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        self.tile_renderer.draw(gfx);
    }
}
