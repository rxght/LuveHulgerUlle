use std::sync::Arc;
use vulkano::{
    descriptor_set::layout::DescriptorSetLayout,
    device::Device,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            depth_stencil::DepthStencilState,
            discard_rectangle::DiscardRectangleState,
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            render_pass::PipelineRenderPassType,
            tessellation::TessellationState,
            vertex_input::VertexBufferDescription,
            viewport::ViewportState,
        },
        layout::{PipelineLayoutCreateInfo, PushConstantRange},
        GraphicsPipeline, PipelineLayout, StateMode,
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
    pub tessellation_state: TessellationState,

    descriptor_set_layouts: Vec<Option<Arc<DescriptorSetLayout>>>,
    pub push_constant_ranges: Vec<PushConstantRange>,
}

impl PipelineBuilder {
    pub fn new(gfx: &Graphics) -> Self {
        Self {
            subpass: Subpass::from(gfx.get_main_render_pass(), 0).unwrap(),
            vertex_buffer_description: None,
            input_assembly_state: InputAssemblyState::new(),
            vertex_shader: None,
            fragment_shader: None,
            viewport_state: ViewportState::viewport_dynamic_scissor_irrelevant(),
            color_blend_state: ColorBlendState::default(),
            rasterization_state: RasterizationState {
                cull_mode: StateMode::Fixed(CullMode::Back),
                front_face: StateMode::Fixed(FrontFace::Clockwise),
                depth_bias: None,
                ..Default::default()
            },
            depth_stencil_state: DepthStencilState::disabled(),
            discard_rectangle_state: DiscardRectangleState::new(),
            multisample_state: MultisampleState::new(),
            tessellation_state: TessellationState::new(),

            descriptor_set_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
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
        let vertex_shader_entry = self
            .vertex_shader
            .as_ref()
            .expect("No vertex shader supplied.")
            .entry_point("main")
            .unwrap();

        let fragment_shader_entry = self
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

        (
            GraphicsPipeline::start()
                .render_pass(PipelineRenderPassType::BeginRenderPass(self.subpass))
                .vertex_input_state(self.vertex_buffer_description.unwrap())
                .input_assembly_state(self.input_assembly_state)
                .vertex_shader(vertex_shader_entry, ())
                .fragment_shader(fragment_shader_entry, ())
                .viewport_state(self.viewport_state)
                .color_blend_state(self.color_blend_state)
                .rasterization_state(self.rasterization_state)
                .depth_stencil_state(self.depth_stencil_state)
                .discard_rectangle_state(self.discard_rectangle_state)
                .multisample_state(self.multisample_state)
                .tessellation_state(self.tessellation_state)
                .with_pipeline_layout(device.clone(), layout.clone())
                .expect("Failed to create pipeline!"),
            layout,
        )
    }
}
