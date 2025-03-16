use super::Vec2;

pub struct Matrix {
    pub rows: [Vec2; 2],
}

impl Matrix {
    pub fn new(rows: [[f32; 2]; 2]) -> Self {
        Self {
            rows: [rows[0].into(), rows[1].into()],
        }
    }

    pub fn inverse(self) -> Option<Self> {
        let a = self.rows[0][0];
        let b = self.rows[0][1];
        let c = self.rows[1][0];
        let d = self.rows[1][1];

        let det = a * d - b * c;

        if det.abs() < f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;

        Some(Matrix {
            rows: [
                Vec2::new(d * inv_det, -b * inv_det),
                Vec2::new(-c * inv_det, a * inv_det),
            ],
        })
    }

    pub fn determinant(&self) -> f32 {
        self.rows[0][0] * self.rows[1][1] - self.rows[0][1] * self.rows[1][0]
    }
}
