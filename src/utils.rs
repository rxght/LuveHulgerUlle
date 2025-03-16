use std::collections::HashSet;

use vulkano::pipeline::graphics::{input_assembly::PrimitiveTopology, vertex_input::Vertex};

pub mod colliders;
pub mod common_meshes;
pub mod math;
pub mod mesh_drawable;
pub mod vertex_types;

#[derive(Debug, Clone, Copy)]
pub enum MeshTopology {
    TriangleList,
    LineList,
    LineStrip,
    PointList,
}

impl Into<PrimitiveTopology> for MeshTopology {
    fn into(self) -> PrimitiveTopology {
        match self {
            MeshTopology::TriangleList => PrimitiveTopology::TriangleList,
            MeshTopology::LineList => PrimitiveTopology::LineList,
            MeshTopology::LineStrip => PrimitiveTopology::LineStrip,
            MeshTopology::PointList => PrimitiveTopology::PointList,
        }
    }
}

#[derive(Debug)]
pub struct Mesh<VertexT: Vertex>
where
    VertexT: Vertex,
{
    pub vertices: Vec<VertexT>,
    pub indices: Vec<u32>,
    pub topology: MeshTopology,
}

impl<VertexT: Vertex> Mesh<VertexT> {
    pub fn new(vertices: Vec<VertexT>, indices: Vec<u32>, topology: MeshTopology) -> Self {
        Self {
            vertices,
            indices,
            topology,
        }
    }

    pub fn into_wireframe(mut self) -> Self {
        match self.topology {
            MeshTopology::LineList => return self,
            MeshTopology::LineStrip => return self,
            MeshTopology::PointList => {
                unimplemented!("A point list can't be turned into a wireframe")
            }
            MeshTopology::TriangleList => {
                let vertices = self.vertices;
                let mut lines = HashSet::<[u32; 2]>::new();
                for triangle in self.indices.chunks_exact_mut(3) {
                    triangle.sort();
                    lines.insert([triangle[0], triangle[1]]);
                    lines.insert([triangle[0], triangle[2]]);
                    lines.insert([triangle[1], triangle[2]]);
                }

                let indices = lines.iter().flatten().cloned().collect();

                return Self {
                    vertices,
                    indices,
                    topology: MeshTopology::LineList,
                };
            }
        }
    }
}
