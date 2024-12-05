use std::sync::Arc;

use super::*;

/// Used to configure pipeline settings and
pub struct GodBindable<BindClosure, BindToPipelineClosure>
where
    BindClosure: Fn(&mut CommandBufferBuilder, Arc<PipelineLayout>),
    BindToPipelineClosure: Fn(&mut PipelineBuilder),
{
    bind_closure: BindClosure,
    bind_to_pipeline_closure: BindToPipelineClosure,
}

impl<B, BP> Bindable for GodBindable<B, BP>
where
    B: Fn(&mut CommandBufferBuilder, Arc<PipelineLayout>),
    BP: Fn(&mut PipelineBuilder),
{
    fn bind_to_pipeline(&self, builder: &mut PipelineBuilder) {
        (self.bind_to_pipeline_closure)(builder)
    }
    fn bind(
        &self,
        _gfx: &Graphics,
        builder: &mut CommandBufferBuilder,
        pipeline_layout: Arc<PipelineLayout>,
    ) {
        (self.bind_closure)(builder, pipeline_layout)
    }
}

impl<B, BP> GodBindable<B, BP>
where
    B: Fn(&mut CommandBufferBuilder, Arc<PipelineLayout>),
    BP: Fn(&mut PipelineBuilder),
{
    pub fn new(bind_closure: B, bind_to_pipeline_closure: BP) -> Arc<Self> {
        Arc::new(Self {
            bind_closure: bind_closure,
            bind_to_pipeline_closure: bind_to_pipeline_closure,
        })
    }
}
