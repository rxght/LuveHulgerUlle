
pub mod rectangle {
    use crate::utils::{vertex_types::VertexPos2, Mesh, MeshTopology, Rect};

    pub fn outline(rect: Rect) -> Mesh<VertexPos2> {
        let vertices = points(rect);
        let indices = vec![ 0, 1, 2, 3, 0 ];
        Mesh::new(vertices, indices, MeshTopology::LineStrip)
    }

    pub fn filled(rect: Rect) -> Mesh<VertexPos2> {
        let vertices = points(rect);
        let indices = vec![ 0, 1, 2, 0, 2, 3 ];
        Mesh::new(vertices, indices, MeshTopology::TriangleList)
    }

    fn points(rect: Rect) -> Vec<VertexPos2> {
        let Rect { position, dimensions } = rect;
        let [x, y] = position;
        let [width, height] = dimensions;

        vec![
            VertexPos2{ pos: [x, y] },
            VertexPos2{ pos: [x, y + height] },
            VertexPos2{ pos: [x + width, y + height] },
            VertexPos2{ pos: [x + width, y] },
        ]
    }
}

pub mod ellipse {
    use std::{f32::consts::PI, sync::atomic::{AtomicU32, Ordering}};
    use crate::utils::{vertex_types::VertexPos2, Mesh, Rect};
    static DETAIL: AtomicU32 = AtomicU32::new(8);

    pub fn outline(rect: Rect) -> Mesh<VertexPos2> {
        let vertices = points(rect);
        let mut indices = Vec::with_capacity(vertices.len() + 1);
        indices.extend(0..vertices.len() as u32);
        indices.push(0);
        
        Mesh {
            vertices,
            indices,
            topology: crate::utils::MeshTopology::LineStrip,
        }
    }

    pub fn filled(rect: Rect) -> Mesh<VertexPos2> {
        let vertices = points(rect);
        let mut indices = Vec::with_capacity(3 * vertices.len() - 6);
        for i in 2..vertices.len() as u32 {
            indices.extend([0, i, i - 1]);
        }
        
        Mesh {
            vertices,
            indices,
            topology: crate::utils::MeshTopology::TriangleList,
        }
    }

    fn points(rect: Rect) -> Vec<VertexPos2> {
        let Rect { position, dimensions } = rect;
        let [x, y] = position;
        let [width, height] = dimensions;

        let center_x = x + 0.5 * width;
        let center_y = y + 0.5 * height;

        let detail = DETAIL.load(Ordering::Relaxed);
        let mut vertices = Vec::with_capacity(4 * detail as usize);
        for i in 0..(detail * 4) {
            let theta = i as f32 / detail as f32 * 0.5 * PI;

            let x = center_x + theta.cos() * 0.5 * width;
            let y = center_y + theta.sin() * 0.5 * height;
            vertices.push(VertexPos2 { pos: [x, y] });
        }

        vertices
    }
}

pub mod rounded_rectangle {
    use std::{f32::consts::PI, sync::atomic::AtomicU32};
    use crate::utils::{vertex_types::VertexPos2, Mesh, Rect};
    static DETAIL: AtomicU32 = AtomicU32::new(5);

    pub fn outline(rect: Rect, radius: f32) -> Mesh<VertexPos2> {
        let vertices = points(rect, radius);
        let mut indices = Vec::with_capacity(vertices.len() + 1);
        indices.extend(0..vertices.len() as u32);
        indices.push(0);
        
        Mesh {
            vertices,
            indices,
            topology: crate::utils::MeshTopology::LineStrip,
        }
    }

    pub fn filled(rect: Rect, radius: f32) -> Mesh<VertexPos2> {
        let vertices = points(rect, radius);
        let mut indices = Vec::with_capacity(3 * vertices.len() - 6);
        for i in 2..vertices.len() as u32 {
            indices.extend([0, i - 1, i]);
        }
        
        Mesh {
            vertices,
            indices,
            topology: crate::utils::MeshTopology::TriangleList,
        }
    }
    
    fn points(rect: Rect, radius: f32) -> Vec<VertexPos2> {
        let Rect { position, dimensions } = rect;
        let [x, y] = position;
        let [width, height] = dimensions;

        let top = y + height;
        let bottom = y;
        let left = x;
        let right = x + width;

        let mut vertices = Vec::new();

        let detail = DETAIL.load(std::sync::atomic::Ordering::Relaxed);
        
        let rx = left + radius;
        let ry = top - radius;
        for i in 0..=detail {
            let theta = (i as f32 / detail as f32) * 0.5 * PI;
            vertices.push(VertexPos2{ pos: [ rx - radius * theta.cos(), ry + radius * theta.sin() ] });
        }

        let rx = right - radius;
        let ry = top - radius;
        for i in 0..=detail {
            let theta = (i as f32 / detail as f32) * 0.5 * PI;
            vertices.push(VertexPos2{ pos: [ rx + radius * theta.sin(), ry + radius * theta.cos() ] });
        }

        let rx = right - radius;
        let ry = bottom + radius;
        for i in 0..=detail {
            let theta = (i as f32 / detail as f32) * 0.5 * PI;
            vertices.push(VertexPos2{ pos: [ rx + radius * theta.cos(), ry - radius * theta.sin() ] });
        }
        
        let rx = left + radius;
        let ry = bottom + radius;
        for i in 0..=detail {
            let theta = (i as f32 / detail as f32) * 0.5 * PI;
            vertices.push(VertexPos2{ pos: [ rx - radius * theta.sin(), ry - radius * theta.cos() ] });
        }

        vertices
    }
}
