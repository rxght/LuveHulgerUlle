use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(Vertex, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct VertexPos3Uv3 {
    #[format(R32G32B32_SFLOAT)]
    pub pos: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub uv: [f32; 3],
}

#[derive(Vertex, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct VertexPos3Uv2 {
    #[format(R32G32B32_SFLOAT)]
    pub pos: [f32; 3],
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

#[derive(Vertex, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct VertexPos3 {
    #[format(R32G32B32_SFLOAT)]
    pub pos: [f32; 3],
}

#[derive(Vertex, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct VertexPos2Uv3 {
    #[format(R32G32_SFLOAT)]
    pub pos: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    pub uv: [f32; 3],
}

#[derive(Vertex, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct VertexPos2Uv2 {
    #[format(R32G32_SFLOAT)]
    pub pos: [f32; 2],
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

#[derive(Vertex, bytemuck::Pod, Clone, Copy, bytemuck::Zeroable, Debug)]
#[repr(C)]
pub struct VertexPos2 {
    #[format(R32G32_SFLOAT)]
    pub pos: [f32; 2],
}
