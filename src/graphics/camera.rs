use std::sync::Arc;

use cgmath::{Deg, Vector3};
use vulkano::{buffer::BufferContents, shader::ShaderStages};

use super::{bindable::UniformBuffer, Graphics};

#[derive(Debug, Clone, Copy, BufferContents)]
#[repr(C)]
pub struct CameraUbo {
    pub camera: [[f32; 4]; 4],
}

pub struct Camera {
    pub position: [f32; 2],
    pub zoom: f32,
    pub rotation: f32,

    buffer: Arc<UniformBuffer<CameraUbo>>,
}

impl Camera {
    pub fn new(gfx: &mut Graphics, position: [f32; 2], zoom: f32, rotation: f32) -> Self {
        let buffer = UniformBuffer::new(
            gfx,
            0,
            CameraUbo {
                camera: (cgmath::Matrix4::from_scale(zoom)
                    * cgmath::Matrix4::from_angle_z(Deg(rotation))
                    * cgmath::Matrix4::from_translation(Vector3::new(
                        -position[0].round(),
                        position[1].round(),
                        0.0,
                    )))
                .into(),
            },
            ShaderStages::VERTEX,
        );

        Self {
            position: position,
            zoom: zoom,
            rotation: rotation,
            buffer: buffer,
        }
    }

    pub fn uniform_buffer(&self) -> Arc<UniformBuffer<CameraUbo>> {
        self.buffer.clone()
    }

    pub fn update_buffer(&mut self) {
        self.buffer.access_data(|data| {
            data.camera = (cgmath::Matrix4::from_nonuniform_scale(self.zoom, self.zoom, 1.0)
                * cgmath::Matrix4::from_angle_z(Deg(self.rotation))
                * cgmath::Matrix4::from_translation(Vector3::new(
                    -self.position[0],
                    -self.position[1],
                    0.0,
                )))
            .into();
        });
    }
}
