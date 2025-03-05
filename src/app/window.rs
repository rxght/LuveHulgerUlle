use std::sync::Arc;

use vulkano::{
    buffer::BufferContents, image::sampler::Filter, pipeline::graphics::vertex_input::Vertex,
    shader::ShaderStages,
};

use crate::graphics::{
    bindable::{
        FragmentShader, IndexBuffer, PushConstant, Texture, TextureBinding, UniformBufferBinding,
        VertexBuffer, VertexShader,
    },
    drawable::Drawable,
    shaders::{frag_textured, vert_ui_textured2},
    Graphics,
};

use super::item::ItemId;

#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
}

pub struct InventoryWindow {
    items: Vec<ItemId>,
    scale_pcr: Arc<PushConstant<vert_ui_textured2::PCR>>,
    drawable: Arc<Drawable>,
}

impl InventoryWindow {
    pub fn new(gfx: &mut Graphics) -> Self {
        let scale_pcr = PushConstant::new(
            0,
            vert_ui_textured2::PCR { ui_scale: 4.0 },
            ShaderStages::VERTEX,
        );
        let texture = Texture::new(gfx, "assets/textures/ui/inventory.png", Filter::Nearest);

        let image_width = texture.extent_2d()[0] as f32;
        let image_height = texture.extent_2d()[1] as f32;

        let min_x = image_width / -2.0;
        let max_x = image_width / 2.0;
        let min_y = image_height / -2.0;
        let max_y = image_height / 2.0;

        let vertices = vec![
            VertexT {
                pos: [min_x, min_y],
                uv: [0.0, 0.0],
            },
            VertexT {
                pos: [max_x, min_y],
                uv: [1.0, 0.0],
            },
            VertexT {
                pos: [min_x, max_y],
                uv: [0.0, 1.0],
            },
            VertexT {
                pos: [max_x, max_y],
                uv: [1.0, 1.0],
            },
        ];

        let indices = vec![0, 1, 2, 2, 1, 3];

        let drawable = Drawable::new(
            gfx,
            vec![
                VertexBuffer::new(gfx, vertices),
                IndexBuffer::new(gfx, indices),
                scale_pcr.clone(),
            ],
            || {
                vec![
                    TextureBinding::new(texture, 1),
                    VertexShader::from_module(vert_ui_textured2::load(gfx.get_device()).unwrap()),
                    FragmentShader::from_module(frag_textured::load(gfx.get_device()).unwrap()),
                    UniformBufferBinding::new(gfx.utils().cartesian_to_normalized(), 0),
                ]
            },
            6,
        );

        Self {
            drawable,
            scale_pcr,
            items: vec![],
        }
    }

    pub fn draw(&self, gfx: &mut Graphics, scale: f32) {
        gfx.queue_drawable(self.drawable.clone());
        self.scale_pcr.access_data(|data| {
            data.ui_scale = scale;
        });
    }
}
