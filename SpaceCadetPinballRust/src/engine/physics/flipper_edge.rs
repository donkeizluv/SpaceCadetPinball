use crate::engine::math::Vec2;

use super::edge::EdgeSegment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipperSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct FlipperEdge {
    side: FlipperSide,
    active: bool,
}

impl FlipperEdge {
    pub const fn new(side: FlipperSide) -> Self {
        Self {
            side,
            active: false,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn segment(&self) -> EdgeSegment {
        match (self.side, self.active) {
            (FlipperSide::Left, false) => {
                EdgeSegment::new(Vec2::new(190.0, 360.0), Vec2::new(280.0, 386.0))
            }
            (FlipperSide::Left, true) => {
                EdgeSegment::new(Vec2::new(190.0, 360.0), Vec2::new(292.0, 334.0))
            }
            (FlipperSide::Right, false) => {
                EdgeSegment::new(Vec2::new(410.0, 386.0), Vec2::new(500.0, 360.0))
            }
            (FlipperSide::Right, true) => {
                EdgeSegment::new(Vec2::new(398.0, 334.0), Vec2::new(500.0, 360.0))
            }
        }
    }
}
