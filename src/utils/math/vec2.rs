use std::ops::{Add, AddAssign, Deref, DerefMut, Div, Mul, Neg, Sub, SubAssign};

use super::Matrix;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Vec2 {
    inner: [f32; 2],
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { inner: [x, y] }
    }

    pub fn hadamard(self, other: Vec2) -> Self {
        Self {
            inner: [self[0] * other[0], self[1] * other[1]],
        }
    }

    pub fn dot(self, other: Vec2) -> f32 {
        self[0] * other[0] + self[1] * other[1]
    }

    pub fn cross(self, other: Vec2) -> f32 {
        self[0] * other[1] - self[1] * other[0]
    }

    pub fn apply_matrix(self, matrix: &Matrix) -> Self {
        Self {
            inner: [matrix.rows[0].dot(self), matrix.rows[1].dot(self)],
        }
    }

    pub fn abs_squared(&self) -> f32 {
        self[0] * self[0] + self[1] * self[1]
    }

    pub fn abs(&self) -> f32 {
        self.abs_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        self / self.abs()
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from(value: [f32; 2]) -> Self {
        Self { inner: value }
    }
}

impl Deref for Vec2 {
    type Target = [f32; 2];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Vec2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Add<Self> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            inner: [self[0] + rhs[0], self[1] + rhs[1]],
        }
    }
}

impl Sub<Self> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            inner: [self[0] - rhs[0], self[1] - rhs[1]],
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            inner: [self[0] * rhs, self[1] * rhs],
        }
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output {
            inner: [self[0] / rhs, self[1] / rhs],
        }
    }
}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        Self::Output {
            inner: [-self[0], -self[1]],
        }
    }
}

impl AddAssign<Self> for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign<Self> for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
