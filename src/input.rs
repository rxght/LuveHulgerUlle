use std::sync::Arc;

use winit::{event::Event, window::Window};

mod keyboard;
pub use keyboard::Keyboard;

mod mouse;
pub use mouse::Mouse;

#[derive(Clone, Debug)]
pub enum ButtonState {
    Pressed(std::time::Instant),
    Held(std::time::Instant),
    Released,
}

pub struct Input {
    pub keyboard: Keyboard,
    keyboard_event_handler: fn(&Keyboard, &Event<'_, ()>) -> bool,

    pub mouse: Mouse,
    mouse_event_handler: fn(&Mouse, &Event<'_, ()>, Arc<Window>) -> bool,
}

impl Input {
    pub fn new() -> Arc<Self> {
        let (keyboard, keyboard_event_handler) = Keyboard::new();
        let (mouse, mouse_event_handler) = Mouse::new();

        Arc::new(Self {
            keyboard: keyboard,
            keyboard_event_handler: keyboard_event_handler,
            mouse: mouse,
            mouse_event_handler: mouse_event_handler,
        })
    }

    /// returns true if the event was handled and false if it should be passed on.
    pub fn handle_event(&self, event: &Event<'_, ()>, window: Arc<Window>) -> bool {
        (self.keyboard_event_handler)(&self.keyboard, event)
            | (self.mouse_event_handler)(&self.mouse, event, window)
    }

    /// call this at the end of each frame to make sure every key press is only counted as a press for one frame
    pub fn clear_presses(&self) {
        self.mouse.clear_presses();
        self.keyboard.clear_presses();
    }
}
