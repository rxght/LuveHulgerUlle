use core::f32;
use std::{collections::HashMap, sync::Arc};

use cgmath::Vector2;
use vulkano::{
    buffer::BufferContents, image::ImageViewAbstract, pipeline::graphics::vertex_input::Vertex,
    shader::ShaderStages,
};

use crate::{
    graphics::{
        bindable::{self, PushConstant, Texture, UniformBuffer},
        camera::Camera,
        drawable::{DrawableEntry, GenericDrawable},
        shaders::frag_textured,
        Graphics,
    },
    ui::Rectangle,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationDesc {
    pub length: u32,
    pub interval: u32,
    pub offset: u32,
}

pub struct TileSet {
    atlas: Arc<Texture>,
    pub tile_width: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
}

impl TileSet {
    pub fn new(gfx: &Graphics, sheet_texture: &str, tile_width: u32) -> Arc<Self> {
        let atlas = Texture::new(gfx, sheet_texture, 0, true);
        let atlas_dimensions = atlas.image.dimensions().width_height();

        let atlas_width = atlas_dimensions[0] / tile_width;
        let atlas_height = atlas_dimensions[1] / tile_width;

        Arc::new(Self {
            atlas: atlas,
            tile_width: tile_width,
            atlas_width: atlas_width,
            atlas_height: atlas_height,
        })
    }

    pub fn get_uv_of_sprite(&self, sprite_idx: u32) -> [[f32; 2]; 4] {
        let y = sprite_idx / self.atlas_width;
        let x = sprite_idx % self.atlas_width;

        let uv_width = self.tile_width as f32 / self.atlas.image.dimensions().width() as f32;
        let uv_height = self.tile_width as f32 / self.atlas.image.dimensions().height() as f32;

        let left = (x) as f32 * uv_width;
        let right = (x + 1) as f32 * uv_width;
        let top = (y) as f32 * uv_height;
        let bottom = (y + 1) as f32 * uv_height;

        [[left, top], [right, top], [left, bottom], [right, bottom]]
    }

    pub fn get_sprite_rectangle(&self, sprite_idx: u32) -> Rectangle {
        let x = sprite_idx % self.atlas_width;
        let y = sprite_idx / self.atlas_width;
        Rectangle {
            x_position: (x * self.tile_width) as i32,
            y_position: (y * self.tile_width) as i32,
            width: self.tile_width,
            height: self.tile_width,
        }
    }

    pub fn get_texture(&self) -> Arc<Texture> {
        self.atlas.clone()
    }
}
