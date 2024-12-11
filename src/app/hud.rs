use egui_winit_vulkano::egui::{
    self, load::SizedTexture, Id, ImageSource, LayerId, Pos2, TextureId, Ui, Vec2,
};
use vulkano::image::sampler::{Filter, SamplerCreateInfo};

use crate::graphics::{bindable::Texture, Graphics};

pub struct Hotbar {
    image_id: TextureId,
    selector_id: TextureId,
    image_size: Vec2,
    selector_size: Vec2,
}

impl Hotbar {
    pub fn new(gfx: &mut Graphics) -> Self {
        let texture = Texture::new(
            gfx,
            "assets/textures/ui/hud/hotbar.png",
            vulkano::image::sampler::Filter::Nearest,
        );
        let selector_texture = Texture::new(
            gfx,
            "assets/textures/ui/hud/hotbar_selector.png",
            vulkano::image::sampler::Filter::Nearest,
        );

        let gui = gfx.gui();
        let image_id = gui.register_user_image_view(
            texture.image_view(),
            SamplerCreateInfo {
                mag_filter: vulkano::image::sampler::Filter::Nearest,
                ..Default::default()
            },
        );
        let selector_id = gui.register_user_image_view(
            selector_texture.image_view(),
            SamplerCreateInfo {
                mag_filter: vulkano::image::sampler::Filter::Nearest,
                ..Default::default()
            },
        );

        let image_size = texture.extent_2d();
        let selector_size = selector_texture.extent_2d();

        Self {
            image_id,
            image_size: [image_size[0] as f32, image_size[1] as f32].into(),
            selector_id,
            selector_size: [selector_size[0] as f32, selector_size[1] as f32].into(),
        }
    }

    pub fn draw(&mut self, gfx: &mut Graphics, slot: u32, scale: f32) {
        let window_size = gfx.get_window().inner_size();
        let image_size = self.image_size * scale;

        let hotbar_rectangle = egui::Rect::from_min_size(
            egui::Pos2 {
                x: (window_size.width as f32 - image_size[0]) / 2.0,
                y: window_size.height as f32 - image_size[1],
            },
            image_size,
        );

        let selector_size = self.selector_size * scale;
        let slot_width = (hotbar_rectangle.width() - 2.0 * scale) / 9.0;

        let selector_rectangle = egui::Rect::from_min_size(
            hotbar_rectangle.min + Vec2::new(slot_width * slot as f32 - scale, -scale),
            selector_size,
        );

        let ctx = gfx.gui().context();
        let mut ui = Ui::new(
            ctx,
            LayerId::background(),
            Id::new("Hotbar"),
            hotbar_rectangle,
            hotbar_rectangle,
        );
        egui::Image::new(ImageSource::Texture(SizedTexture {
            id: self.image_id,
            size: image_size,
        }))
        .paint_at(&mut ui, hotbar_rectangle);
        egui::Image::new(ImageSource::Texture(SizedTexture {
            id: self.selector_id,
            size: selector_size,
        }))
        .paint_at(&mut ui, selector_rectangle);
    }
}

pub struct Healthbar {
    heart_id: TextureId,
    container_id: TextureId,
    half_heart_id: TextureId,
    heart_size: egui::Vec2,
}

impl Healthbar {
    pub fn new(gfx: &mut Graphics) -> Self {
        let container_texture = Texture::new(
            gfx,
            "assets/textures/ui/hud/heartcontainer.png",
            Filter::Nearest,
        );
        let heart_texture = Texture::new(gfx, "assets/textures/ui/hud/heart.png", Filter::Nearest);
        let half_heart_texture =
            Texture::new(gfx, "assets/textures/ui/hud/halfheart.png", Filter::Nearest);

        let gui = gfx.gui();

        let container_id = gui.register_user_image_view(
            container_texture.image_view(),
            SamplerCreateInfo {
                mag_filter: vulkano::image::sampler::Filter::Nearest,
                ..Default::default()
            },
        );
        let heart_id = gui.register_user_image_view(
            heart_texture.image_view(),
            SamplerCreateInfo {
                mag_filter: vulkano::image::sampler::Filter::Nearest,
                ..Default::default()
            },
        );
        let half_heart_id = gui.register_user_image_view(
            half_heart_texture.image_view(),
            SamplerCreateInfo {
                mag_filter: vulkano::image::sampler::Filter::Nearest,
                ..Default::default()
            },
        );

        let c_heart = heart_texture.extent_2d();

        Self {
            heart_id,
            container_id,
            half_heart_id,
            heart_size: [c_heart[0] as f32, c_heart[1] as f32].into(),
        }
    }

    pub fn draw(&mut self, gfx: &mut Graphics, health: u32, scale: f32) {
        let window_size = gfx.get_window().inner_size();

        let heart_size = self.heart_size * scale;
        let gap_size = 1.0 * scale;
        let total_width = heart_size.x * 10.0 + gap_size * 9.0;
        let total_height = heart_size.y;
        let position_offset = Vec2::new(-40.0, -22.0) * scale;
        let max_rect = egui::Rect::from_min_size(
            Pos2::new(
                (window_size.width as f32 - total_width) / 2.0,
                window_size.height as f32 - total_height,
            ) + position_offset,
            Vec2::new(total_width, total_height),
        );

        let heart = egui::Image::new(ImageSource::Texture(SizedTexture {
            id: self.heart_id,
            size: heart_size,
        }));
        let half_heart = egui::Image::new(ImageSource::Texture(SizedTexture {
            id: self.half_heart_id,
            size: heart_size,
        }));
        let container = egui::Image::new(ImageSource::Texture(SizedTexture {
            id: self.container_id,
            size: heart_size,
        }));

        let ctx = gfx.gui().context();
        let ui = Ui::new(
            ctx.clone(),
            LayerId::background(),
            Id::new("Healthbar"),
            max_rect,
            max_rect,
        );

        // render containers
        let mut rect = egui::Rect::from_min_size(max_rect.min, heart_size);
        for _ in 0..10 {
            container.paint_at(&ui, rect);
            rect.min.x += heart_size.x + gap_size;
            rect.max.x += heart_size.x + gap_size;
        }

        // render hearts
        let mut rect = egui::Rect::from_min_size(max_rect.min, heart_size);
        for _ in 0..(health / 2) {
            heart.paint_at(&ui, rect);
            rect.min.x += heart_size.x + gap_size;
            rect.max.x += heart_size.x + gap_size;
        }
        if health % 2 == 1 {
            half_heart.paint_at(&ui, rect);
        }
    }
}
