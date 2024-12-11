use std::sync::Arc;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineLayout};

use super::bindable::Bindable;
use super::pipeline::PipelineBuilder;

pub struct DrawableSharedPart {
    pub bindables: Vec<Arc<dyn Bindable>>,
    pub pipeline: Arc<GraphicsPipeline>,
}

pub struct Drawable {
    bindables: Vec<Arc<dyn Bindable>>,
    shared_part: Arc<DrawableSharedPart>,
    index_count: u32,
}

impl Drawable {
    #[track_caller]
    pub fn new<Fn1>(
        gfx: &super::Graphics,
        bindables: Vec<Arc<dyn Bindable>>,
        init_shared_bindables: Fn1,
        index_count: u32,
    ) -> Arc<Self>
    where
        Fn1: FnOnce() -> Vec<Arc<dyn Bindable>>,
    {
        let caller_location = std::panic::Location::caller();

        match gfx.get_shared_data(caller_location) {
            Some(data) => Arc::new(Self {
                bindables: bindables,
                shared_part: data,
                index_count: index_count,
            }),
            None => {
                let bindables = bindables;
                let shared_bindables = init_shared_bindables();

                let mut pipeline_builder = PipelineBuilder::new(gfx);

                for bindable in &bindables {
                    bindable.bind_to_pipeline(&mut pipeline_builder);
                }
                for bindable in &shared_bindables {
                    bindable.bind_to_pipeline(&mut pipeline_builder);
                }

                let (pipeline, _) = pipeline_builder.build(gfx.get_device());

                let shared_part = Arc::new(DrawableSharedPart {
                    bindables: shared_bindables,
                    pipeline: pipeline,
                });

                gfx.cache_drawable_shared_part(caller_location, shared_part.clone());

                Arc::new(Self {
                    bindables: bindables,
                    shared_part: shared_part,
                    index_count: index_count,
                })
            }
        }
    }

    pub fn get_bindables(&self) -> &[Arc<dyn Bindable>] {
        &self.bindables
    }

    pub fn get_shared_bindables(&self) -> &[Arc<dyn Bindable>] {
        &self.shared_part.bindables
    }

    pub fn get_index_count(&self) -> u32 {
        self.index_count
    }

    pub fn get_pipeline(&self) -> Arc<GraphicsPipeline> {
        self.shared_part.pipeline.clone()
    }

    pub fn get_pipeline_layout(&self) -> Arc<PipelineLayout> {
        self.shared_part.pipeline.layout().clone()
    }
}
