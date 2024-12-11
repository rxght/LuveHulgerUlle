use smallvec::smallvec;
use vulkano::pipeline::DynamicState;

use crate::graphics::Graphics;

use super::Bindable;

struct Viewport {
    generator: dyn Fn(&Graphics) -> vulkano::pipeline::graphics::viewport::Viewport,
}

impl Bindable for Viewport {
    fn bind_to_pipeline(&self, builder: &mut crate::graphics::pipeline::PipelineBuilder) {
        builder.dynamic_state.insert(DynamicState::Viewport);
    }
    fn bind(
        &self,
        gfx: &crate::graphics::Graphics,
        builder: &mut super::CommandBufferBuilder,
        _pipeline_layout: std::sync::Arc<vulkano::pipeline::PipelineLayout>,
    ) {
        let vp = (&self.generator)(gfx);
        builder.set_viewport(0, smallvec![vp]).unwrap();
    }
}

struct Scissor {
    generator: dyn Fn(&Graphics) -> vulkano::pipeline::graphics::viewport::Scissor,
}

impl Bindable for Scissor {
    fn bind_to_pipeline(&self, builder: &mut crate::graphics::pipeline::PipelineBuilder) {
        builder.dynamic_state.insert(DynamicState::Scissor);
    }
    fn bind(
        &self,
        gfx: &crate::graphics::Graphics,
        builder: &mut super::CommandBufferBuilder,
        _pipeline_layout: std::sync::Arc<vulkano::pipeline::PipelineLayout>,
    ) {
        let sc = (&self.generator)(gfx);
        builder.set_scissor(0, smallvec![sc]).unwrap();
    }
}
