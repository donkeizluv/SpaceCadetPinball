use crate::engine::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeSegment {
    pub start: Vec2,
    pub end: Vec2,
}

impl EdgeSegment {
    pub const fn new(start: Vec2, end: Vec2) -> Self {
        Self { start, end }
    }

    pub fn direction(self) -> Vec2 {
        Vec2::new(self.end.x - self.start.x, self.end.y - self.start.y)
    }

    pub fn offset(self, amount: f32) -> Self {
        let direction = self.direction();
        let length = direction.length_squared().sqrt();
        if length <= f32::EPSILON {
            return self;
        }

        let perpendicular = Vec2::new(-direction.y / length, direction.x / length);
        let delta = Vec2::new(perpendicular.x * amount, perpendicular.y * amount);
        Self {
            start: Vec2::new(self.start.x + delta.x, self.start.y + delta.y),
            end: Vec2::new(self.end.x + delta.x, self.end.y + delta.y),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeCircle {
    pub center: Vec2,
    pub radius: f32,
}

impl EdgeCircle {
    pub const fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
}
