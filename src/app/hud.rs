use std::sync::Arc;

use crate::{
    graphics::{bindable::Texture, Graphics},
    ui::{image::UiImage, UiElement, UiLayout, UiScene, UiUnit},
};

pub struct Hotbar {
    base: Arc<UiImage>,
    selector: Arc<UiImage>,

    ui_scene: Arc<UiScene>,
    selected_slot: u32,
    hud_scale: f32,
}

impl Hotbar {
    pub fn new(gfx: &mut Graphics) -> Self {

        let hud_scale = 2.0;

        let hotbar_texture = Texture::new(gfx, "assets/textures/ui/hud/hotbar.png", vulkano::sampler::Filter::Nearest);
        let hotbar_width = hotbar_texture.dimensions().width() as f32 * hud_scale;
        let hotbar_height = hotbar_texture.dimensions().height() as f32 * hud_scale;

        let base = UiImage::new(
            gfx,
            hotbar_texture,
            UiLayout{
                x: UiUnit::Combined(50.0, -hotbar_width / 2.0),
                y: UiUnit::Combined(100.0, -hotbar_height),
                width: UiUnit::Pixels(hotbar_width),
                height: UiUnit::Pixels(hotbar_height),
            }
        );

        let selector_texture = Texture::new(gfx, "assets/textures/ui/hud/hotbar_selector.png", vulkano::sampler::Filter::Nearest);
        let width = selector_texture.dimensions().width() as f32 * hud_scale;
        let height = selector_texture.dimensions().height() as f32 * hud_scale;


        let selector = UiImage::new(
            gfx,
            selector_texture,
            UiLayout {
                x: UiUnit::Combined(50.0, -hotbar_width / 2.0 - hud_scale),
                y: UiUnit::Combined(100.0, -hotbar_height - hud_scale),
                width: UiUnit::Pixels(width),
                height: UiUnit::Pixels(height),
            }
        );

        Self {
            base: base.clone(),
            selector: selector.clone(),
            ui_scene: Arc::new(UiScene(vec![base, selector])),
            selected_slot: 0,
            hud_scale,
        }
    }

    pub fn ui_scene(&self) -> &Arc<UiScene> {
        &self.ui_scene
    }
    
    pub fn selected_slot(&self) -> u32 {
        self.selected_slot
    }
    
    pub fn set_selected_slot(&mut self, gfx: &Graphics, selected_slot: u32) {
        self.selected_slot = selected_slot;
        let selector_layout = self.selector.layout_mut();

        let hotbar_x = self.base.layout().x;
        let hotbar_y = self.base.layout().y;

        const SLOT_WIDTH: f32 = 20.0;
        let slot_idx = selected_slot as f32;

        selector_layout.x = hotbar_x + UiUnit::Pixels(self.hud_scale * (slot_idx * SLOT_WIDTH - 1.0));
        selector_layout.y = hotbar_y + UiUnit::Pixels(-self.hud_scale);

        self.selector.handle_resize(gfx.get_window().inner_size().into());
    }
}
