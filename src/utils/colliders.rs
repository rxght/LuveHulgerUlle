use std::collections::{HashMap, HashSet};

use ordered_float::{FloatIsNan, NotNan};

use crate::utils::math::{Line, Matrix};

use super::math::{Rect, Vec2};

const MAX_COLLISIONS_PER_FRAME: usize = 10;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
struct ValidRect {
    min: [NotNan<f32>; 2],
    max: [NotNan<f32>; 2],
}

impl TryFrom<Rect> for ValidRect {
    type Error = FloatIsNan;

    fn try_from(value: Rect) -> Result<Self, Self::Error> {
        let [min_x, min_y] = value.min;
        let [max_x, max_y] = value.max;
        Ok(Self {
            min: [min_x.try_into()?, min_y.try_into()?],
            max: [max_x.try_into()?, max_y.try_into()?],
        })
    }
}

impl Into<Rect> for ValidRect {
    fn into(self) -> Rect {
        Rect {
            min: [self.min[0].into_inner(), self.min[1].into_inner()],
            max: [self.max[0].into_inner(), self.max[1].into_inner()],
        }
    }
}

impl ValidRect {
    pub fn new(min: [f32; 2], max: [f32; 2]) -> Result<Self, FloatIsNan> {
        Rect::new(min, max).try_into()
    }

    pub fn intersects(&self, other: &ValidRect) -> bool {
        !(self.max[0] <= other.min[0]
            || self.min[0] >= other.max[0]
            || self.max[1] <= other.min[1]
            || self.min[1] >= other.max[1])
    }

    pub fn add_offset(self, offset: Vec2) -> Self {
        let offset_x = NotNan::new(offset[0]).unwrap();
        let offset_y = NotNan::new(offset[1]).unwrap();
        Self {
            min: [self.min[0] + offset_x, self.min[1] + offset_y],
            max: [self.max[0] + offset_x, self.max[1] + offset_y],
        }
    }

    pub fn min(&self) -> [f32; 2] {
        [self.min[0].into_inner(), self.min[1].into_inner()]
    }

    pub fn max(&self) -> [f32; 2] {
        [self.max[0].into_inner(), self.max[1].into_inner()]
    }
}

#[derive(Debug)]
pub struct HitboxManager {
    chunks: HashMap<[i32; 2], HitboxChunk>,
    chunk_dimensions: [f32; 2],
}

#[derive(Default, Debug)]
struct HitboxChunk {
    bounding_box: Option<ValidRect>,
    elements: HashSet<ValidRect>,
}

impl HitboxChunk {
    pub fn insert(&mut self, hitbox: ValidRect) {
        self.expand_bounding_box(&hitbox);
        self.elements.insert(hitbox);
    }

    pub fn remove(&mut self, hitbox: ValidRect) {
        if self.elements.remove(&hitbox) {
            self.recalculate_bounding_box();
        }
    }

    fn recalculate_bounding_box(&mut self) {
        if self.elements.is_empty() {
            self.bounding_box = None;
            return;
        }

        let mut bb_min_x = f32::INFINITY;
        let mut bb_max_x = f32::NEG_INFINITY;
        let mut bb_min_y = f32::INFINITY;
        let mut bb_max_y = f32::NEG_INFINITY;

        for hitbox in self.elements.iter() {
            let [min_x, min_y] = hitbox.min;
            let [max_x, max_y] = hitbox.max;

            bb_min_x = f32::min(bb_min_x, *min_x);
            bb_max_x = f32::max(bb_max_x, *max_x);
            bb_min_y = f32::min(bb_min_y, *min_y);
            bb_max_y = f32::max(bb_max_y, *max_y);
        }

        self.bounding_box = Some(ValidRect {
            min: [bb_min_x.try_into().unwrap(), bb_min_y.try_into().unwrap()],
            max: [bb_max_x.try_into().unwrap(), bb_max_y.try_into().unwrap()],
        })
    }

    fn expand_bounding_box(&mut self, hitbox: &ValidRect) {
        if self.elements.is_empty() {
            self.bounding_box = Some(hitbox.clone());
        }

        match &mut self.bounding_box {
            Some(bb) => {
                bb.min[0] = bb.min[0].min(hitbox.min[0]);
                bb.max[0] = bb.max[0].max(hitbox.max[0]);
                bb.min[1] = bb.min[1].min(hitbox.min[1]);
                bb.max[1] = bb.max[1].max(hitbox.max[1]);
            }
            None => {
                self.bounding_box = Some(hitbox.clone());
                return;
            }
        }
    }
}

impl HitboxManager {
    pub fn new(hitboxes: impl IntoIterator<Item = Rect>, chunk_dimensions: [f32; 2]) -> Self {
        let mut obj = Self {
            chunks: HashMap::new(),
            chunk_dimensions,
        };
        obj.extend(hitboxes);
        return obj;
    }

    pub fn insert(&mut self, hitbox: Rect) -> Result<(), FloatIsNan> {
        let valid_rect = hitbox.try_into()?;
        let chunk_coords = self.get_chunk_coordinates(&valid_rect);
        let chunk = self.chunks.entry(chunk_coords).or_default();
        chunk.insert(valid_rect);
        Ok(())
    }

    pub fn remove(&mut self, hitbox: Rect) -> Result<(), FloatIsNan> {
        let valid_rect = hitbox.try_into()?;
        let chunk_coords = self.get_chunk_coordinates(&valid_rect);
        if let Some(chunk) = self.chunks.get_mut(&chunk_coords) {
            chunk.remove(valid_rect);
        }
        Ok(())
    }

    pub fn hit_test(&self, _rect: Rect) -> bool {
        todo!()
    }

    pub fn moving_hit_test(&self, rect: Rect, movement: Vec2) -> Vec2 {
        let v_rect = rect.try_into().unwrap();
        let chunks = self.nearby_chunks(&v_rect);

        let mut remaining_movement: Vec2 = movement;
        let mut applied_movement: Vec2 = Vec2::new(0.0, 0.0);

        let rect = Rect::new(*(Vec2::from(rect.min) + applied_movement), *(Vec2::from(rect.max) + applied_movement));

        let [min_x, min_y] = rect.min;
        let [max_x, max_y] = rect.max;

        let top_left = Vec2::new(min_x, max_y);
        let top_right = Vec2::new(max_x, max_y);
        let bottom_left = Vec2::new(min_x, min_y);
        let bottom_right = Vec2::new(max_x, min_y);

        let source_lines = [
            Line::from_start_end(top_left, top_right),
            Line::from_start_end(top_right, bottom_right),
            Line::from_start_end(bottom_right, bottom_left),
            Line::from_start_end(bottom_left, top_left),
        ];

        let v_target_rect = v_rect.add_offset(remaining_movement);

        let colliders = chunks
            .into_iter()
            .filter(|f| {
                f.bounding_box
                    .as_ref()
                    .is_some_and(|f| f.intersects(&v_target_rect))
            })
            .flat_map(|f| f.elements.iter())
            .collect::<Vec<_>>();

        let mut min_t: f32 = 1.0;
        let mut collision_direction = Vec2::new(0.0, 0.0);

        for elem in colliders {
            if !elem.intersects(&v_target_rect) {
                continue;
            }

            let [min_x, min_y] = elem.min();
            let [max_x, max_y] = elem.max();

            let top_left = Vec2::new(min_x, max_y);
            let top_right = Vec2::new(max_x, max_y);
            let bottom_left = Vec2::new(min_x, min_y);
            let bottom_right = Vec2::new(max_x, min_y);

            let collision_lines = [
                Line::from_start_end(top_left, top_right),
                Line::from_start_end(top_right, bottom_right),
                Line::from_start_end(bottom_right, bottom_left),
                Line::from_start_end(bottom_left, top_left),
            ];

            for collision_line in collision_lines {
                if collision_line.direction().cross(remaining_movement) >= 0.0 {
                    continue;
                }

                for source_line in &source_lines {
                    if source_line.direction().cross(remaining_movement) <= 0.0 {
                        continue;
                    }
                    let collision_t = Self::line_collision(source_line, remaining_movement, &collision_line);
                    //println!("let l1 = Line::from_position_direction({:?}.into(), {:?}.into())", *source_line.start_point(), *source_line.direction());
                    //println!("let l2 = Line::from_position_direction({:?}.into(), {:?}.into())", *collision_line.start_point(), *collision_line.direction());
                    //println!("let movement = {:?}.into()", *remaining_movement);
                    //println!("  => {collision_t}");
                    //assert!(-1.0 <= collision_t && collision_t <= 1.0);
                    if collision_t.abs() < min_t.abs() {
                        collision_direction = source_line.direction().normalized();
                        min_t = collision_t;
                    }
                    if collision_t.abs() - min_t.abs() < f32::EPSILON {
                        let source_d = source_line.direction().normalized();
                        let collider_d = collision_line.direction().normalized();
                        if source_d.dot(collider_d).abs() > 0.9 {
                            collision_direction = source_d;
                        }
                    }
                }
            }
        }
        applied_movement += min_t * remaining_movement;
        remaining_movement = (1.0 - min_t) * remaining_movement.dot(collision_direction) * collision_direction;
        return applied_movement + remaining_movement;
    }

    fn nearby_chunks(&self, hitbox: &ValidRect) -> Vec<&HitboxChunk> {
        let [chunk_x, chunk_y] = self.get_chunk_coordinates(hitbox);

        let mut nearby_chunks: Vec<&HitboxChunk> = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                let key = [chunk_x + x as i32 - 1, chunk_y + y as i32 - 1];
                if let Some(a) = self.chunks.get(&key) {
                    nearby_chunks.push(a);
                }
            }
        }
        return nearby_chunks;
    }

    #[inline]
    fn get_chunk_coordinates(&self, hitbox: &ValidRect) -> [i32; 2] {
        [
            (hitbox.min[0].into_inner() / self.chunk_dimensions[0]).floor() as i32,
            (hitbox.min[1].into_inner() / self.chunk_dimensions[1]).floor() as i32,
        ]
    }

    fn line_collision(moving_line: &Line, movement: Vec2, collider: &Line) -> f32 {
        let lp = moving_line.start_point();
        let ld = moving_line.direction();

        let m = Matrix{
            rows: [[ld[0], movement[0]].into(), [ld[1], movement[1]].into()],
        }.inverse();

        let transform = match m {
            Some(v) => v,
            None => return 1.0,
        };
        
        let repositioned_collider = Line::from_position_direction(collider.start_point() - lp, collider.direction());
        let mut remapped_collider = repositioned_collider.apply_matrix(&transform);

        // normalize line direction such that higher t always gives a higher distance
        if remapped_collider.direction()[1] < 0.0 {
            let p = remapped_collider.start_point();
            let d = remapped_collider.direction();
            remapped_collider = Line::from_position_direction(p + d, -d);
        }

        let [offset_k, distance_k] = *remapped_collider.direction();
        let [offset_m, distance_m] = *remapped_collider.start_point();

        // distance = distance_k * t + distance_m
        // offset = offset_k * t + offset_m

        // test t = 0.0
        let t = 0.0;
        let offset = offset_k * t + offset_m;
        if offset >= 0.0 {
            if offset <= 1.0 {
                return f32::min(distance_k * t + distance_m, 1.0);
            }
            // offset > 1.0
            if offset_k >= 0.0 {
                return 1.0;
            }
            // solve for t when offset = 1.0, and then check if it's <= 1.0
            let t = (1.0 - offset_m) / offset_k;
            if t <= 1.0 {
                return f32::min(distance_k * t + distance_m, 1.0);
            }
            return 1.0;
        }
        // we know: offset < 0.0
        if offset_k <= 0.0 {
            return 1.0;
        }
        // solve for t when offset = 0.0, and then check if it's <= 1.0
        let t = -offset_m / offset_k;
        if t <= 1.0 {
            return f32::min(distance_k * t + distance_m, 1.0);
        }
        return 1.0;
    }
}


impl FromIterator<Rect> for HitboxManager {
    fn from_iter<T: IntoIterator<Item = Rect>>(iter: T) -> Self {
        let mut obj = Self {
            chunks: HashMap::new(),
            chunk_dimensions: [128.0; 2],
        };
        obj.extend(iter);
        return obj;
    }
}

impl Extend<Rect> for HitboxManager {
    fn extend<T: IntoIterator<Item = Rect>>(&mut self, iter: T) {
        for rect in iter {
            self.insert(rect).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{colliders::HitboxManager, math::Line};

    macro_rules! approx_assert_eq {
        ($left:expr, $right:expr $(,)?) => {
            match (&$left, &$right) {
                (left_val, right_val) => {
                    if (left_val - right_val).abs() > 0.001 {
                        assert_eq!(left_val, right_val);
                    }
                }
            }
        };
    }
    
    #[test]
    fn test_line_collision() {
        let l1 = Line::from_start_end([0.0, 0.0].into(), [0.0, 10.0].into());
        let l2 = Line::from_start_end([10.0, 0.0].into(), [10.0, 10.0].into());
    
        approx_assert_eq!(HitboxManager::line_collision(&l1, [10.0, 0.0].into(), &l2), 1.0);
        approx_assert_eq!(HitboxManager::line_collision(&l1, [20.0, 0.0].into(), &l2), 0.5);
        
        let l1 = Line::from_start_end([-2.0, 0.0].into(), [0.0, 10.0].into());
        let l2 = Line::from_start_end([12.0, 0.0].into(), [10.0, 10.0].into());
    
        approx_assert_eq!(HitboxManager::line_collision(&l1, [10.0, 0.0].into(), &l2), 1.0);
        approx_assert_eq!(HitboxManager::line_collision(&l1, [20.0, 0.0].into(), &l2), 0.5);
        
        let l1 = Line::from_start_end([0.0, 0.0].into(), [0.0, 10.0].into());
        let l2 = Line::from_start_end([5.0, 10.0].into(), [10.0, 10.0].into());
    
        approx_assert_eq!(HitboxManager::line_collision(&l1, [5.0, 5.0].into(), &l2), 1.0);
        approx_assert_eq!(HitboxManager::line_collision(&l1, [10.0, 10.0].into(), &l2), 0.5);
    
        let l1 = Line::from_position_direction([128.76125, 38.796474].into(), [0.0, 16.0].into());
        let l2 = Line::from_position_direction([159.0, 33.0].into(), [-14.0, 0.0].into());
        let movement = [0.509916, 0.0];
    
        let result = HitboxManager::line_collision(&l1, movement.into(), &l2);
        dbg!(result);
        assert!(-1.0 <= result && result <= 1.0);
    }
}