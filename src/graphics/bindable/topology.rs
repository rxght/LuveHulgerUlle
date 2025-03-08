use std::sync::Arc;

use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;

use super::Bindable;

pub struct Topology {
    topology: PrimitiveTopology,
}

impl Topology {
    pub fn new(topology: PrimitiveTopology) -> Arc<Self> {
        Arc::new(Self { topology })
    }
}

impl Bindable for Topology {
    fn bind_to_pipeline(&self, builder: &mut crate::graphics::pipeline::PipelineBuilder) {
        builder.input_assembly_state.topology = self.topology
    }
}
