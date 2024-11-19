use std::{
    cell::{Cell, UnsafeCell},
    collections::HashMap,
    sync::Arc,
};

use cgmath::Vector2;
use winit::{
    event::{DeviceEvent, ElementState, Event, WindowEvent},
    window::Window,
};

use super::ButtonState;

pub struct Mouse {
    pub cursor_position: Cell<Vector2<f64>>,
    pub mouse_movement: Cell<Vector2<f64>>,
    pub scroll_wheel_movement: Cell<f32>,
    button_map: UnsafeCell<HashMap<u32, ButtonState>>,
}

impl Mouse {
    pub fn new() -> (Self, fn(&Mouse, &Event<'_, ()>, Arc<Window>) -> bool) {
        (
            Self {
                cursor_position: Cell::new(Vector2 { x: 0.0, y: 0.0 }),
                mouse_movement: Cell::new(Vector2 { x: 0.0, y: 0.0 }),
                button_map: UnsafeCell::new(HashMap::new()),
                scroll_wheel_movement: Cell::new(0.0),
            },
            Mouse::_event_handler,
        )
    }

    pub fn is_button_pressed(&self, button_id: u32) -> bool {
        match self.get_button_state(button_id) {
            Some(ButtonState::Pressed(_)) => true,
            _ => false,
        }
    }

    pub fn is_button_held(&self, button_id: u32) -> Option<std::time::Duration> {
        match self.get_button_state(button_id) {
            Some(ButtonState::Held(start)) => Some(std::time::Instant::now() - start),
            _ => None,
        }
    }

    pub fn get_button_state(&self, button_id: u32) -> Option<ButtonState> {
        let button_map = unsafe { self.button_map.get().as_ref()? };
        button_map.get(&button_id).cloned()
    }

    fn _event_handler(&self, event: &Event<'_, ()>, window: Arc<Window>) -> bool {
        let button_map = unsafe { self.button_map.get().as_mut().unwrap() };
        match event {
            Event::DeviceEvent {
                event,
                device_id: _,
            } => {
                if let DeviceEvent::Button { button, state } = event {
                    let previous_state = button_map.get(button).cloned();

                    //match *state {
                    //    ElementState::Pressed => println!("Button: {button}, Pressed"),
                    //    ElementState::Released => println!("Button: {button}, Released"),
                    //};

                    match previous_state {
                        Some(ButtonState::Pressed(_)) | Some(ButtonState::Held(_)) => {
                            if *state == ElementState::Released {
                                button_map.insert(*button, ButtonState::Released);
                            }
                        }
                        _ => {
                            if *state == ElementState::Pressed {
                                button_map.insert(
                                    *button,
                                    ButtonState::Pressed(std::time::Instant::now()),
                                );
                            }
                        }
                    }
                }
                if let DeviceEvent::MouseMotion { delta } = event {
                    self.mouse_movement
                        .set(self.mouse_movement.get() + Vector2::from(*delta));
                    return true;
                }
                if let DeviceEvent::MouseWheel { delta } = event {
                    match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, val) => self
                            .scroll_wheel_movement
                            .set(self.scroll_wheel_movement.get() + val),
                        winit::event::MouseScrollDelta::PixelDelta(_) => unimplemented!(),
                    }
                }
                return false;
            }
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::CursorMoved { position, .. } = event {
                    let window_size = window.inner_size();

                    self.cursor_position.set(Vector2 {
                        x: -((window_size.width / 2) as f64) + position.x,
                        y: (window_size.height / 2) as f64 - position.y,
                    });
                    return true;
                }

                return false;
            }
            _ => false,
        }
    }

    pub fn clear_presses(&self) {
        self.scroll_wheel_movement.set(0.0);
        self.mouse_movement.set(Vector2::new(0.0, 0.0));
        let button_map = unsafe { self.button_map.get().as_mut().unwrap() };
        for (_, state) in button_map.iter_mut() {
            if let ButtonState::Pressed(inst) = *state {
                *state = ButtonState::Held(inst);
            }
        }
    }
}
