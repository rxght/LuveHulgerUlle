use std::sync::Arc;

use vulkano::shader::ShaderStages;

use crate::graphics::{
    bindable::{self, UniformBuffer},
    camera::CameraUbo,
    drawable::Drawable,
    shaders::{frag_color, vert_world_pos2},
    Graphics,
};

use super::Mesh;

pub struct MeshDrawable {
    base_drawable: Arc<Drawable>,
}

impl MeshDrawable {
    pub fn new(
        gfx: &mut Graphics,
        mesh: Mesh<super::vertex_types::VertexPos2>,
        color: [f32; 4],
        camera: Arc<UniformBuffer<CameraUbo>>,
    ) -> Self {
        let index_count = mesh.indices.len() as u32;

        let vb = bindable::VertexBuffer::new(gfx, mesh.vertices);
        let ib = bindable::IndexBuffer::new(gfx, mesh.indices);
        let topology = mesh.topology;
        let color_buffer = UniformBuffer::new(gfx, 0, frag_color::ColorData{ color }, ShaderStages::FRAGMENT);

        let base_drawable = Drawable::new(
            gfx,
            vec![
                vb,
                ib,
                bindable::Topology::new(topology.into()),
                bindable::UniformBufferBinding::new(color_buffer, 1),
                bindable::UniformBufferBinding::new(camera, 2),
            ],
            || {
                vec![
                    bindable::VertexShader::from_module(
                        vert_world_pos2::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::FragmentShader::from_module(
                        frag_color::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::UniformBufferBinding::new(gfx.utils().cartesian_to_normalized(), 0),
                ]
            },
            index_count,
        );

        Self { base_drawable }
    }

    pub fn draw(&self, gfx: &mut Graphics) {
        gfx.queue_drawable(self.base_drawable.clone());
    }
}
