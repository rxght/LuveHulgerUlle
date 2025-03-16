#[derive(PartialEq, Eq, Debug, Default, Clone, Copy, Hash)]
pub struct Rect<T = f32>
where
    T: num_traits::Num,
{
    pub min: [T; 2],
    pub max: [T; 2],
}

impl<T> Rect<T>
where
    T: num_traits::Num,
{
    pub fn new(min: [T; 2], max: [T; 2]) -> Self {
        Self { min, max }
    }
}

impl<T> Rect<T>
where
    T: num_traits::Num + Copy,
{
    pub fn width(&self) -> T {
        self.max[0] - self.min[0]
    }

    pub fn height(&self) -> T {
        self.max[1] - self.min[1]
    }
}

impl<T> Rect<T>
where
    T: num_traits::Num + Ord,
{
    pub fn intersects(&self, other: &Rect<T>) -> bool {
        !(self.max[0] <= other.min[0]
            || self.min[0] >= other.max[0]
            || self.max[1] <= other.min[1]
            || self.min[1] >= other.max[1])
    }
}
