use std::{cell::UnsafeCell, sync::Arc};

use winit::event::{DeviceEvent, ElementState, Event};

use crate::{
    graphics::{drawable::DrawableEntry, Graphics},
    input::Input,
};

pub mod button;
pub mod image;
pub mod ui_square;

pub struct UiScene(pub Vec<Arc<dyn UiElement>>);

pub struct Ui {
    scene: UnsafeCell<Arc<UiScene>>,
    clicked_element: UnsafeCell<Option<Arc<dyn UiElement>>>,
    captured_buttons: UnsafeCell<Vec<u32>>,
}

impl Ui {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            scene: UnsafeCell::new(Arc::new(UiScene(Vec::new()))),
            clicked_element: UnsafeCell::new(None),
            captured_buttons: UnsafeCell::new(Vec::new()),
        })
    }

    pub fn set_scene(&self, gfx: &mut Graphics, scene: Arc<UiScene>) {
        let current_scene = unsafe { self.scene.get().as_mut().unwrap() };
        for old_element in current_scene.0.iter() {
            // unregister drawables from the old scene
            gfx.unregister_drawable(old_element.get_drawable_entry());
        }
        *current_scene = scene;
        for new_elem in current_scene.0.iter() {
            // register drawables from the new scene
            gfx.register_drawable(new_elem.get_drawable_entry());
        }
    }

    pub fn handle_resize(&self, new_size: [u32; 2]) {
        let current_scene = unsafe { self.scene.get().as_ref().unwrap() };

        for elem in current_scene.0.iter() {
            elem.handle_resize(new_size);
        }
    }

    pub fn handle_event(
        &self,
        event: &Event<'_, ()>,
        input_state: Arc<Input>,
        window_extent: [u32; 2],
    ) -> bool {
        match event {
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::Button { button, state } = event {
                    if *state == ElementState::Pressed {
                        let scene_elements = &unsafe { &*self.scene.get() }.0;
                        let mut consume_event = false;
                        for elem in scene_elements.iter().rev() {
                            let elem_hitbox = elem.get_layout();
                            let cursor_position = input_state.mouse.cursor_position.get();
                            let cursor_position = [
                                2.0 * cursor_position.x as f32 / window_extent[0] as f32,
                                -2.0 * cursor_position.y as f32 / window_extent[1] as f32,
                            ];

                            if cursor_position[0] >= elem_hitbox.x_position
                                && cursor_position[0] <= elem_hitbox.x_position + elem_hitbox.width
                                && cursor_position[1] >= elem_hitbox.y_position
                                && cursor_position[1] <= elem_hitbox.y_position + elem_hitbox.height
                            {
                                consume_event = true;
                                let event_handled = elem.handle_event(event);
                                if event_handled {
                                    unsafe { *self.clicked_element.get() = Some(elem.clone()) };
                                    break;
                                }
                            }
                        }
                        if consume_event {
                            (unsafe { &mut *self.captured_buttons.get() }).push(*button);
                        }
                        return consume_event;
                    }
                    if *state == ElementState::Released {
                        unsafe {
                            if let Some(elem) = &*self.clicked_element.get() {
                                elem.handle_event(event);
                                *self.clicked_element.get() = None;
                                return true;
                            }
                            let captured_buttons = &mut *self.captured_buttons.get();
                            if captured_buttons.contains(button) {
                                captured_buttons.retain(|p| *p != *button);
                                return true;
                            }
                            return false;
                        }
                    }
                }
            }
            _ => return false,
        };

        return false;
    }
}

#[derive(Clone, Copy)]
pub struct NormalizedRectangle {
    x_position: f32,
    y_position: f32,
    width: f32,
    height: f32,
}

pub struct Rectangle {
    pub x_position: i32,
    pub y_position: i32,
    pub width: u32,
    pub height: u32,
}

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

pub struct UiLayout {
    pub x: UiUnit,
    pub y: UiUnit,
    pub width: UiUnit,
    pub height: UiUnit,
}

impl UiLayout {
    pub fn normalize(&self, window_extent: [u32; 2]) -> NormalizedRectangle {
        NormalizedRectangle {
            x_position: self.x.to_normalized(window_extent[0] as f32) - 1.0,
            y_position: self.y.to_normalized(window_extent[1] as f32) - 1.0,
            width: self.width.to_normalized(window_extent[0] as f32),
            height: self.height.to_normalized(window_extent[1] as f32),
        }
    }
}

pub trait UiElement {
    fn handle_resize(&self, new_size: [u32; 2]);
    fn get_drawable_entry(&self) -> &DrawableEntry;
    fn get_layout(&self) -> NormalizedRectangle;
    fn handle_event(&self, _event: &DeviceEvent) -> bool {
        false
    }
}
