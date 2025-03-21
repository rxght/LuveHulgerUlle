#![allow(unused_imports)]
use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder,
        PrimaryAutoCommandBuffer,
    },
    pipeline::PipelineLayout,
    shader::ShaderModule,
};

use super::{pipeline::PipelineBuilder, Graphics};

mod buffer;
mod push_constant;
mod shader;
mod texture;
mod topology;
mod uniform;
mod viewport;

pub use buffer::*;
pub use push_constant::*;
pub use shader::*;
pub use texture::*;
pub use topology::*;
pub use uniform::*;
pub use viewport::*;

pub type CommandBufferBuilder = AutoCommandBufferBuilder<
    PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>,
    Arc<StandardCommandBufferAllocator>,
>;

pub trait Bindable {
    fn bind_to_pipeline(&self, builder: &mut PipelineBuilder);
    fn bind(
        &self,
        _gfx: &Graphics,
        _builder: &mut CommandBufferBuilder,
        _pipeline_layout: Arc<PipelineLayout>,
    ) {
    }
}
