use std::cell::Cell;
use std::sync::{Arc, Weak};
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout};

use super::bindable::Bindable;
use super::pipeline::PipelineBuilder;

pub trait Drawable {
    fn get_bindables(&self) -> &Vec<Arc<dyn Bindable>>;
    fn get_shared_bindables(&self) -> &Vec<Arc<dyn Bindable>>;
    fn get_pipeline(&self) -> Arc<GraphicsPipeline>;
    fn get_index_count(&self) -> u32;
    fn get_pipeline_layout(&self) -> Arc<PipelineLayout>;
}

pub struct DrawableSharedPart {
    pub bindables: Vec<Arc<dyn Bindable>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub layout: Arc<PipelineLayout>,
}

pub struct GenericDrawable {
    bindables: Vec<Arc<dyn Bindable>>,
    shared_part: Arc<DrawableSharedPart>,
    index_count: Cell<u32>,
}

pub struct DrawableEntry {
    entry: Arc<GenericDrawable>,
    pub registered_uid: Cell<Option<u32>>,
}

impl DrawableEntry {
    pub fn get_weak(&self) -> Weak<GenericDrawable> {
        Arc::downgrade(&self.entry)
    }
    pub fn get_arc(&self) -> Arc<GenericDrawable> {
        self.entry.clone()
    }
}

impl GenericDrawable {
    #[track_caller]
    pub fn new<Fn1, Fn2>(
        gfx: &super::Graphics,
        init_bindables: Fn1,
        init_shared_bindables: Fn2,
        index_count: u32,
    ) -> DrawableEntry
    where
        Fn1: FnOnce() -> Vec<Arc<dyn Bindable>>,
        Fn2: FnOnce() -> Vec<Arc<dyn Bindable>>,
    {
        let caller_location = std::panic::Location::caller();

        let shared_data = match gfx.get_shared_data_map().get(caller_location) {
            Some(weak) => match weak.upgrade() {
                Some(arc) => Some(arc),
                None => None,
            },
            None => None,
        };

        match shared_data {
            Some(data) => DrawableEntry {
                entry: Arc::new(Self {
                    bindables: init_bindables(),
                    shared_part: data,
                    index_count: Cell::new(index_count),
                }),
                registered_uid: Cell::new(None),
            },
            None => {
                let bindables = init_bindables();
                let shared_bindables = init_shared_bindables();

                let mut pipeline_builder = PipelineBuilder::new(gfx);

                for bindable in &bindables {
                    bindable.bind_to_pipeline(&mut pipeline_builder);
                }
                for bindable in &shared_bindables {
                    bindable.bind_to_pipeline(&mut pipeline_builder);
                }

                let (pipeline, layout) = pipeline_builder.build(gfx.get_device());

                let shared_part = Arc::new(DrawableSharedPart {
                    bindables: shared_bindables,
                    pipeline: pipeline,
                    layout: layout,
                });

                gfx.cache_drawable_shared_part(caller_location, shared_part.clone());

                DrawableEntry {
                    entry: Arc::new(Self {
                        bindables: bindables,
                        shared_part: shared_part,
                        index_count: Cell::new(index_count),
                    }),
                    registered_uid: Cell::new(None),
                }
            }
        }
    }
}

impl Drawable for GenericDrawable {
    fn get_bindables(&self) -> &Vec<Arc<dyn Bindable>> {
        &self.bindables
    }
    fn get_shared_bindables(&self) -> &Vec<Arc<dyn Bindable>> {
        &self.shared_part.bindables
    }
    fn get_pipeline(&self) -> Arc<GraphicsPipeline> {
        self.shared_part.pipeline.clone()
    }
    fn get_index_count(&self) -> u32 {
        self.index_count.get()
    }
    fn get_pipeline_layout(&self) -> Arc<PipelineLayout> {
        self.shared_part.layout.clone()
    }
}
