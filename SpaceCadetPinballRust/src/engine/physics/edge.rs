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
}
