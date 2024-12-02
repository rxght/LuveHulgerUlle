use std::{ops::Add, sync::Arc};

use winit::event::{DeviceEvent, ElementState, Event};

use crate::{
    graphics::drawable::Drawable,
    input::Input,
};

pub struct UiRenderer {
    elements: Vec<Arc<dyn UiElement>>,
    clicked_element: Option<Arc<dyn UiElement>>,
    captured_buttons: Vec<u32>,
}

impl UiRenderer {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            clicked_element: None,
            captured_buttons: Vec::new(),
        }
    }

    pub fn clear_last_frame(&mut self) {
        self.elements.truncate(0);
    }

    pub fn queue_drawable(&mut self, drawable: Arc<dyn UiElement>) {
        self.elements.push(drawable);
    }

    pub(super) fn elements(&self) -> &[Arc<dyn UiElement>] {
        &self.elements
    }

    pub fn handle_event(
        &mut self,
        event: &Event<'_, ()>,
        input_state: &Input,
        window_extent: [u32; 2],
    ) -> bool {
        match event {
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::Button { button, state } = event {
                    if *state == ElementState::Pressed {
                        let mut consume_event = false;
                        for elem in self.elements.iter().rev() {
                            let elem_hitbox = elem.get_layout();
                            let cursor_position = input_state.mouse.cursor_position.get();
                            let cursor_position = [
                                2.0 * cursor_position.x as f32 / window_extent[0] as f32,
                                -2.0 * cursor_position.y as f32 / window_extent[1] as f32,
                            ];

                            if cursor_position[0] >= elem_hitbox.x
                                && cursor_position[0] <= elem_hitbox.x + elem_hitbox.width
                                && cursor_position[1] >= elem_hitbox.y
                                && cursor_position[1] <= elem_hitbox.y + elem_hitbox.height
                            {
                                consume_event = true;
                                let event_handled = elem.handle_event(event);
                                if event_handled {
                                    self.clicked_element = Some(elem.clone());
                                    break;
                                }
                            }
                        }
                        if consume_event {
                            self.captured_buttons.push(*button);
                        }
                        return consume_event;
                    }
                    if *state == ElementState::Released {
                        if let Some(elem) = self.clicked_element.take() {
                            elem.handle_event(event);
                            return true;
                        }
                        if self.captured_buttons.contains(button) {
                            self.captured_buttons.retain(|p| *p != *button);
                            return true;
                        }
                        return false;
                    }
                }
            }
            _ => return false,
        };

        return false;
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rectangle<T> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

#[derive(Clone, Copy)]
pub enum UiUnit {
    Percentage(f32),
    Pixels(f32),
    Combined(f32, f32),
}

impl UiUnit {
    pub fn to_normalized(&self, px_max: f32) -> f32 {
        match self {
            UiUnit::Percentage(p) => p / 50.0,
            UiUnit::Pixels(px) => (2.0 * px) / px_max,
            UiUnit::Combined(p, px) => p / 50.0 + (2.0 * px) / px_max,
        }
    }
}

impl Add<Self> for UiUnit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (p, px) = match self {
            UiUnit::Percentage(p) => (p, 0.0),
            UiUnit::Pixels(px) => (0.0, px),
            UiUnit::Combined(p, px) => (p, px),
        };

        match rhs {
            UiUnit::Percentage(p2) => UiUnit::Combined(p + p2, px),
            UiUnit::Pixels(px2) => UiUnit::Combined(p, px + px2),
            UiUnit::Combined(p2, px2) => UiUnit::Combined(p + p2, px + px2),
        }
    }
}

pub struct UiLayout {
    pub x: UiUnit,
    pub y: UiUnit,
    pub width: UiUnit,
    pub height: UiUnit,
}

impl UiLayout {
    pub fn to_normalized(&self, window_extent: [u32; 2]) -> Rectangle<f32> {
        Rectangle {
            x: self.x.to_normalized(window_extent[0] as f32) - 1.0,
            y: self.y.to_normalized(window_extent[1] as f32) - 1.0,
            width: self.width.to_normalized(window_extent[0] as f32),
            height: self.height.to_normalized(window_extent[1] as f32),
        }
    }
}

pub trait UiElement {
    fn handle_resize(&self, new_size: [u32; 2]);
    fn get_drawable(&self) -> Arc<Drawable>;
    fn get_layout(&self) -> Rectangle<f32> {
        Rectangle::default()
    }
    fn handle_event(&self, _event: &DeviceEvent) -> bool {
        false
    }
}
