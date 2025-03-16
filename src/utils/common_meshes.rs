pub mod rectangle {
    use crate::utils::{math::Rect, vertex_types::VertexPos2, Mesh, MeshTopology};

    pub fn outline(rect: Rect) -> Mesh<VertexPos2> {
        let vertices = points(rect);
        let indices = vec![0, 1, 2, 3, 0];
        Mesh::new(vertices, indices, MeshTopology::LineStrip)
    }

    pub fn filled(rect: Rect) -> Mesh<VertexPos2> {
        let vertices = points(rect);
        let indices = vec![0, 1, 2, 0, 2, 3];
        Mesh::new(vertices, indices, MeshTopology::TriangleList)
    }

    fn points(rect: Rect) -> Vec<VertexPos2> {
        let [min_x, min_y] = rect.min;
        let [max_x, max_y] = rect.max;

        vec![
            VertexPos2 {
                pos: [min_x, min_y],
            },
            VertexPos2 {
                pos: [min_x, max_y],
            },
            VertexPos2 {
                pos: [max_x, max_y],
            },
            VertexPos2 {
                pos: [max_x, min_y],
            },
        ]
    }
}

pub mod ellipse {
    use crate::utils::{math::Rect, vertex_types::VertexPos2, Mesh};
    use std::{
        f32::consts::PI,
        sync::atomic::{AtomicU32, Ordering},
    };
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
        let [min_x, min_y] = rect.min;
        let [max_x, max_y] = rect.max;

        let center_x = 0.5 * (min_x + max_x);
        let center_y = 0.5 * (min_y + max_y);

        let width = max_x - min_x;
        let height = max_y - min_y;

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
    use crate::utils::{math::Rect, vertex_types::VertexPos2, Mesh};
    use std::{f32::consts::PI, sync::atomic::AtomicU32};
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
        let [min_x, min_y] = rect.min;
        let [max_x, max_y] = rect.max;

        let mut vertices = Vec::new();

        let detail = DETAIL.load(std::sync::atomic::Ordering::Relaxed);

        let rx = min_x + radius;
        let ry = max_y - radius;
        for i in 0..=detail {
            let theta = (i as f32 / detail as f32) * 0.5 * PI;
            vertices.push(VertexPos2 {
                pos: [rx - radius * theta.cos(), ry + radius * theta.sin()],
            });
        }

        let rx = max_x - radius;
        let ry = max_y - radius;
        for i in 0..=detail {
            let theta = (i as f32 / detail as f32) * 0.5 * PI;
            vertices.push(VertexPos2 {
                pos: [rx + radius * theta.sin(), ry + radius * theta.cos()],
            });
        }

        let rx = max_x - radius;
        let ry = min_y + radius;
        for i in 0..=detail {
            let theta = (i as f32 / detail as f32) * 0.5 * PI;
            vertices.push(VertexPos2 {
                pos: [rx + radius * theta.cos(), ry - radius * theta.sin()],
            });
        }

        let rx = min_x + radius;
        let ry = min_y + radius;
        for i in 0..=detail {
            let theta = (i as f32 / detail as f32) * 0.5 * PI;
            vertices.push(VertexPos2 {
                pos: [rx - radius * theta.sin(), ry - radius * theta.cos()],
            });
        }

        vertices
    }
}
