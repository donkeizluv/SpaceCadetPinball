use super::ball::Ball;
use super::collision::{CollisionContact, collide_ball_with_circle, collide_ball_with_edge};
use super::edge::{EdgeCircle, EdgeSegment};
use super::flipper_edge::{FlipperEdge, FlipperSide};
use crate::engine::math::Vec2;

#[derive(Debug, Clone, Copy)]
struct RegisteredEdge {
    collider: RegisteredCollider,
    owner_token: Option<u32>,
    role: RegisteredEdgeRole,
}

#[derive(Debug, Clone, Copy)]
enum RegisteredCollider {
    Segment(EdgeSegment),
    Circle(EdgeCircle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegisteredEdgeRole {
    Solid,
    Trigger,
}

pub struct EdgeManager {
    bounds: Vec<EdgeSegment>,
    walls: Vec<RegisteredEdge>,
    left_flipper: FlipperEdge,
    right_flipper: FlipperEdge,
    restitution: f32,
}

impl EdgeManager {
    pub fn for_table_bounds(width: f32, height: f32) -> Self {
        Self {
            bounds: vec![
                EdgeSegment::new(Vec2::new(8.0, 8.0), Vec2::new(width - 8.0, 8.0)),
                EdgeSegment::new(Vec2::new(8.0, 8.0), Vec2::new(8.0, height - 18.0)),
                EdgeSegment::new(
                    Vec2::new(width - 8.0, 8.0),
                    Vec2::new(width - 8.0, height - 18.0),
                ),
            ],
            walls: Vec::new(),
            left_flipper: FlipperEdge::new(FlipperSide::Left),
            right_flipper: FlipperEdge::new(FlipperSide::Right),
            restitution: 0.82,
        }
    }

    pub fn add_wall(&mut self, wall: EdgeSegment) {
        self.add_owned_wall(wall, None);
    }

    pub fn add_owned_wall(&mut self, wall: EdgeSegment, owner_token: Option<u32>) {
        self.walls.push(RegisteredEdge {
            collider: RegisteredCollider::Segment(wall),
            owner_token,
            role: RegisteredEdgeRole::Solid,
        });
    }

    pub fn add_owned_circle(&mut self, circle: EdgeCircle, owner_token: Option<u32>) {
        self.walls.push(RegisteredEdge {
            collider: RegisteredCollider::Circle(circle),
            owner_token,
            role: RegisteredEdgeRole::Solid,
        });
    }

    pub fn add_owned_trigger(&mut self, wall: EdgeSegment, owner_token: Option<u32>) {
        self.walls.push(RegisteredEdge {
            collider: RegisteredCollider::Segment(wall),
            owner_token,
            role: RegisteredEdgeRole::Trigger,
        });
    }

    pub fn wall_count(&self) -> usize {
        self.bounds.len() + self.walls.len()
    }

    pub fn set_flipper_state(&mut self, left_active: bool, right_active: bool) {
        self.left_flipper.set_active(left_active);
        self.right_flipper.set_active(right_active);
    }

    pub fn resolve_ball(&self, ball: &mut Ball) -> Option<CollisionContact> {
        self.resolve_ball_with_filter(ball, |_| true)
    }

    pub fn resolve_ball_with_filter(
        &self,
        ball: &mut Ball,
        mut owner_is_active: impl FnMut(Option<u32>) -> bool,
    ) -> Option<CollisionContact> {
        for edge in self
            .bounds
            .iter()
            .copied()
            .map(|segment| (RegisteredCollider::Segment(segment), None))
            .chain(
                self.walls
                    .iter()
                    .filter(|wall| wall.role == RegisteredEdgeRole::Solid)
                    .map(|wall| (wall.collider, wall.owner_token)),
            )
            .chain(
                [self.left_flipper.segment(), self.right_flipper.segment()]
                    .into_iter()
                    .map(|segment| (RegisteredCollider::Segment(segment), None)),
            )
        {
            if !owner_is_active(edge.1) {
                continue;
            }

            let contact = match edge.0 {
                RegisteredCollider::Segment(segment) => {
                    collide_ball_with_edge(ball, segment, self.restitution)
                }
                RegisteredCollider::Circle(circle) => {
                    collide_ball_with_circle(ball, circle, self.restitution)
                }
            };
            if let Some(contact) = contact {
                return Some(contact);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::math::Vec2;
    use crate::engine::physics::{Ball, EdgeCircle, EdgeSegment};

    use super::EdgeManager;

    #[test]
    fn owned_walls_can_be_filtered_by_component_activity() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_wall(
            EdgeSegment::new(Vec2::new(100.0, 200.0), Vec2::new(200.0, 200.0)),
            Some(7),
        );

        let mut active_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        active_ball.velocity = Vec2::new(0.0, 20.0);
        let active_contact =
            edge_manager.resolve_ball_with_filter(&mut active_ball, |owner_token| {
                owner_token != Some(7) || true
        });
        assert!(active_contact.is_some());

        let mut inactive_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        inactive_ball.velocity = Vec2::new(0.0, 20.0);
        let inactive_contact = edge_manager
            .resolve_ball_with_filter(&mut inactive_ball, |owner_token| owner_token != Some(7));
        assert!(inactive_contact.is_none());
    }

    #[test]
    fn trigger_edges_do_not_collide_like_solid_walls() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_trigger(
            EdgeSegment::new(Vec2::new(100.0, 200.0), Vec2::new(200.0, 200.0)),
            Some(8),
        );

        let mut ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        ball.velocity = Vec2::new(0.0, 20.0);
        let contact = edge_manager.resolve_ball_with_filter(&mut ball, |_| true);
        assert!(contact.is_none());
    }

    #[test]
    fn owned_circles_collide_like_solid_walls() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_circle(EdgeCircle::new(Vec2::new(150.0, 200.0), 4.0), Some(9));

        let mut ball = Ball::ready_at(Vec2::new(150.0, 191.0));
        ball.velocity = Vec2::new(0.0, 20.0);
        let contact = edge_manager.resolve_ball_with_filter(&mut ball, |_| true);
        assert!(contact.is_some());
    }
}
