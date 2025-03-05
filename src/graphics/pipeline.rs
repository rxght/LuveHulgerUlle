use egui_winit_vulkano::egui::ahash;
use smallvec::smallvec;
use std::{collections::HashSet, sync::Arc};
use vulkano::{
    descriptor_set::layout::DescriptorSetLayout,
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{
                AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
                ColorComponents,
            },
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            discard_rectangle::DiscardRectangleState,
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            subpass::PipelineSubpassType,
            tessellation::TessellationState,
            vertex_input::{VertexBufferDescription, VertexDefinition},
            viewport::{Scissor, Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::{PipelineLayoutCreateInfo, PushConstantRange},
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::Subpass,
    shader::ShaderModule,
};

use super::Graphics;

pub struct PipelineBuilder {
    pub subpass: Subpass,
    pub vertex_buffer_description: Option<VertexBufferDescription>,
    pub input_assembly_state: InputAssemblyState,
    pub vertex_shader: Option<Arc<ShaderModule>>,
    pub fragment_shader: Option<Arc<ShaderModule>>,
    pub viewport_state: ViewportState,
    pub color_blend_state: ColorBlendState,
    pub rasterization_state: RasterizationState,
    pub depth_stencil_state: DepthStencilState,
    pub discard_rectangle_state: DiscardRectangleState,
    pub multisample_state: MultisampleState,
    pub tessellation_state: Option<TessellationState>,
    pub dynamic_state: HashSet<DynamicState, ahash::RandomState>,

    descriptor_set_layouts: Vec<Option<Arc<DescriptorSetLayout>>>,
    pub push_constant_ranges: Vec<PushConstantRange>,
}

impl PipelineBuilder {
    pub fn new(gfx: &Graphics) -> Self {
        Self {
            subpass: Subpass::from(gfx.get_main_render_pass(), 0).unwrap(),
            vertex_buffer_description: None,
            input_assembly_state: InputAssemblyState::default(),
            vertex_shader: None,
            fragment_shader: None,
            viewport_state: ViewportState {
                viewports: smallvec![Viewport {
                    offset: [0.0, 0.0],
                    extent: [1.0, 1.0],
                    depth_range: 0.0..=1.0
                }],
                scissors: smallvec![Scissor::default()],
                ..Default::default()
            },
            color_blend_state: ColorBlendState {
                attachments: vec![ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend {
                        src_color_blend_factor: BlendFactor::SrcAlpha,
                        dst_color_blend_factor: BlendFactor::OneMinusSrcAlpha,
                        color_blend_op: BlendOp::Add,
                        ..Default::default()
                    }),
                    color_write_mask: ColorComponents::all(),
                    color_write_enable: true,
                }],
                ..Default::default()
            },
            rasterization_state: RasterizationState {
                cull_mode: CullMode::Back,
                front_face: FrontFace::Clockwise,
                depth_bias: None,
                ..Default::default()
            },
            depth_stencil_state: DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::LessOrEqual,
                }),
                ..Default::default()
            },
            discard_rectangle_state: DiscardRectangleState::default(),
            multisample_state: MultisampleState::default(),
            tessellation_state: None,

            descriptor_set_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
            dynamic_state: HashSet::from_iter([DynamicState::Viewport]),
        }
    }

    pub fn add_descriptor_set_layout(&mut self, set_num: u32, layout: Arc<DescriptorSetLayout>) {
        let set_num = set_num as usize;

        if self.descriptor_set_layouts.len() <= set_num {
            self.descriptor_set_layouts.resize(set_num + 1, None);
        }
        self.descriptor_set_layouts[set_num] = Some(layout);
    }

    pub fn build(self, device: Arc<Device>) -> (Arc<GraphicsPipeline>, Arc<PipelineLayout>) {
        let vs = self
            .vertex_shader
            .as_ref()
            .expect("No vertex shader supplied.")
            .entry_point("main")
            .unwrap();

        let fs = self
            .fragment_shader
            .as_ref()
            .expect("No fragment shader supplied.")
            .entry_point("main")
            .unwrap();

        let set_layouts = self
            .descriptor_set_layouts
            .into_iter()
            .enumerate()
            .map(|(set_num, opt)| match opt {
                Some(v) => v,
                None => panic!("Descriptor set with set_num: {set_num} is missing!"),
            })
            .collect();

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineLayoutCreateInfo {
                set_layouts: set_layouts,
                push_constant_ranges: self.push_constant_ranges,
                ..Default::default()
            },
        )
        .unwrap();

        let vertex_input_state = Some(
            self.vertex_buffer_description
                .unwrap()
                .per_vertex()
                .definition(&vs.info().input_interface)
                .unwrap(),
        );

        let stages = [vs, fs]
            .into_iter()
            .map(|ep| PipelineShaderStageCreateInfo::new(ep))
            .collect();

        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages,
                vertex_input_state,
                input_assembly_state: Some(self.input_assembly_state),
                tessellation_state: self.tessellation_state,
                viewport_state: Some(self.viewport_state),
                rasterization_state: Some(self.rasterization_state),
                multisample_state: Some(self.multisample_state),
                depth_stencil_state: Some(self.depth_stencil_state),
                color_blend_state: Some(self.color_blend_state),
                dynamic_state: self.dynamic_state,
                subpass: Some(PipelineSubpassType::BeginRenderPass(self.subpass)),
                base_pipeline: None,
                discard_rectangle_state: None,
                ..GraphicsPipelineCreateInfo::layout(layout.clone())
            },
        )
        .unwrap();

        (pipeline, layout)
    }
}
