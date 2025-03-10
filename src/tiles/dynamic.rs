use std::sync::Arc;

use vulkano::{
    buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::ShaderStages,
};

use crate::graphics::{
    bindable::{self, PushConstant, Texture, TextureBinding},
    camera::Camera,
    drawable::Drawable,
    shaders::{frag_texture_array, vert_dynamic_tile},
    Graphics,
};

#[derive(BufferContents, Vertex, Clone)]
#[repr(C)]
struct VertexT {
    #[format(R32G32_SFLOAT)]
    pos: [f32; 2],
}

pub struct DynamicTile {
    texture: Arc<TextureBinding>,
    object_data: Arc<bindable::PushConstant<vert_dynamic_tile::ObjectData>>,
    drawable: Arc<Drawable>,
}

impl DynamicTile {
    pub fn new(gfx: &Graphics, initial_texture: Arc<Texture>, camera: &Camera) -> Self {
        let tile_dimensions = initial_texture.extent();

        let object_data = bindable::PushConstant::new(
            0,
            vert_dynamic_tile::ObjectData {
                position: [0.0, 0.0],
                dimensions: [tile_dimensions[0] as f32, tile_dimensions[1] as f32],
                layer_idx: 0.0,
            },
            ShaderStages::VERTEX,
        );

        let texture_binding = bindable::TextureBinding::new(initial_texture, 1);

        let drawable = Drawable::new(
            gfx,
            vec![object_data.clone()],
            || {
                let vertices = vec![
                    VertexT { pos: [0.0, 0.0] },
                    VertexT { pos: [1.0, 0.0] },
                    VertexT { pos: [0.0, 1.0] },
                    VertexT { pos: [1.0, 1.0] },
                ];

                let indices = vec![0, 1, 2, 2, 1, 3];
                vec![
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                    bindable::VertexShader::from_module(
                        vert_dynamic_tile::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::FragmentShader::from_module(
                        frag_texture_array::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::UniformBufferBinding::new(
                        gfx.utils().cartesian_to_normalized().clone(),
                        0,
                    ),
                    bindable::UniformBufferBinding::new(camera.uniform_buffer(), 2),
                    texture_binding.clone(),
                ]
            },
            6,
        );

        Self {
            texture: texture_binding,
            object_data,
            drawable,
        }
    }

    pub fn set_texture(&self, texture: Arc<Texture>) {
        self.texture.set_texture(texture);
    }

    pub fn set_dimensions(&self, dimensions: [f32; 2]) {
        self.object_data.access_data(|data| {
            data.dimensions = dimensions;
        });
    }

    pub fn set_position(&self, position: [f32; 2]) {
        self.object_data.access_data(|data| {
            data.position = position;
        });
    }

    pub fn set_layer(&self, layer: u32) {
        self.object_data.access_data(|data| {
            data.layer_idx = layer as f32;
        });
    }

    pub fn dimensions(&self) -> [f32; 2] {
        let mut dimensions = [0.0, 0.0];
        self.object_data.access_data(|data| {
            dimensions = data.dimensions;
        });
        return dimensions;
    }

    pub fn object_data(&self) -> &PushConstant<vert_dynamic_tile::ObjectData> {
        &self.object_data
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        gfx.queue_drawable(self.drawable.clone());
    }
}
