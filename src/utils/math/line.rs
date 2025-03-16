use std::ops::{Add, Sub};

use super::{Matrix, Vec2};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Line {
    p: Vec2,
    d: Vec2,
}

impl Line {
    pub fn from_position_direction(position: Vec2, direction: Vec2) -> Self {
        Self {
            p: position,
            d: direction,
        }
    }

    pub fn from_start_end(start: Vec2, end: Vec2) -> Self {
        Self {
            p: start,
            d: end - start,
        }
    }

    pub fn intersection(&self, other: &Line) -> f32 {
        (other.p - self.p).dot(other.d) / self.d.dot(other.d)
    }

    pub fn intersection_point(&self, other: &Line) -> Vec2 {
        let t = self.intersection(other);
        self.p + t * self.d
    }

    pub fn intersects(&self, other: &Line) -> bool {
        let t = self.intersection(other);
        return t >= 0.0 && t <= 1.0;
    }

    pub fn direction(&self) -> Vec2 {
        self.d
    }

    pub fn start_point(&self) -> Vec2 {
        self.p
    }

    pub fn apply_matrix(self, matrix: &Matrix) -> Self {
        Self {
            p: self.p.apply_matrix(matrix),
            d: self.d.apply_matrix(matrix),
        }
    }
}

impl Add<Vec2> for Line {
    type Output = Line;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            p: self.p + rhs,
            ..self
        }
    }
}

impl Add<Line> for Vec2 {
    type Output = Line;

    fn add(self, rhs: Line) -> Self::Output {
        rhs + self
    }
}

impl Sub<Vec2> for Line {
    type Output = Line;

    fn sub(self, rhs: Vec2) -> Self::Output {
        self + -rhs
    }
}
