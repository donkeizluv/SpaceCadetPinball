use super::ball::Ball;
use super::collision::{CollisionContact, collide_ball_with_edge};
use super::edge::EdgeSegment;
use super::flipper_edge::{FlipperEdge, FlipperSide};
use crate::engine::math::Vec2;

pub struct EdgeManager {
    walls: Vec<EdgeSegment>,
    left_flipper: FlipperEdge,
    right_flipper: FlipperEdge,
    restitution: f32,
}

impl EdgeManager {
    pub fn for_table_bounds(width: f32, height: f32) -> Self {
        Self {
            walls: vec![
                EdgeSegment::new(Vec2::new(8.0, 8.0), Vec2::new(width - 8.0, 8.0)),
                EdgeSegment::new(Vec2::new(8.0, 8.0), Vec2::new(8.0, height - 18.0)),
                EdgeSegment::new(
                    Vec2::new(width - 8.0, 8.0),
                    Vec2::new(width - 8.0, height - 18.0),
                ),
            ],
            left_flipper: FlipperEdge::new(FlipperSide::Left),
            right_flipper: FlipperEdge::new(FlipperSide::Right),
            restitution: 0.82,
        }
    }

    pub fn set_flipper_state(&mut self, left_active: bool, right_active: bool) {
        self.left_flipper.set_active(left_active);
        self.right_flipper.set_active(right_active);
    }

    pub fn resolve_ball(&self, ball: &mut Ball) -> Option<CollisionContact> {
        for edge in self
            .walls
            .iter()
            .copied()
            .chain([self.left_flipper.segment(), self.right_flipper.segment()])
        {
            if let Some(contact) = collide_ball_with_edge(ball, edge, self.restitution) {
                return Some(contact);
            }
        }

        None
    }
}
