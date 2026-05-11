use super::ball::Ball;
use super::collision::{
    CollisionContact, CollisionEdgeRole, CollisionResponseParams, collide_ball_with_circle,
    collide_ball_with_edge, detect_ball_with_circle, detect_ball_with_edge,
};
use super::edge::{EdgeCircle, EdgeSegment};
use super::flipper_edge::{FlipperEdge, FlipperSide};
use crate::engine::math::{Vec2, clamp};

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

const BOUND_EDGE_ID_BASE: u32 = 1;
const WALL_EDGE_ID_BASE: u32 = 1_000;
const FLIPPER_EDGE_ID_BASE: u32 = 2_000;

#[derive(Debug, Clone, Copy)]
struct GridConfig {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    advance_x: f32,
    advance_y: f32,
    max_box_x: usize,
    max_box_y: usize,
}

impl GridConfig {
    fn new(min_x: f32, min_y: f32, width: f32, height: f32) -> Self {
        let max_box_x = 10;
        let max_box_y = 15;
        Self {
            min_x,
            min_y,
            max_x: min_x + width,
            max_y: min_y + height,
            advance_x: width / max_box_x as f32,
            advance_y: height / max_box_y as f32,
            max_box_x,
            max_box_y,
        }
    }

    fn box_x(&self, x: f32) -> usize {
        let normalized = ((x - self.min_x) / self.advance_x).floor();
        clamp(normalized, 0.0, self.max_box_x.saturating_sub(1) as f32) as usize
    }

    fn box_y(&self, y: f32) -> usize {
        let normalized = ((y - self.min_y) / self.advance_y).floor();
        clamp(normalized, 0.0, self.max_box_y.saturating_sub(1) as f32) as usize
    }

    fn box_index(&self, x: usize, y: usize) -> usize {
        x + y * self.max_box_x
    }

    fn total_box_count(&self) -> usize {
        self.max_box_x * self.max_box_y
    }
}

pub struct EdgeManager {
    bounds: Vec<EdgeSegment>,
    walls: Vec<RegisteredEdge>,
    grid: Vec<Vec<usize>>,
    grid_config: GridConfig,
    left_flipper: FlipperEdge,
    right_flipper: FlipperEdge,
    restitution: f32,
}

impl EdgeManager {
    pub fn for_table_bounds(width: f32, height: f32) -> Self {
        let grid_config = GridConfig::new(0.0, 0.0, width, height);
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
            grid: vec![Vec::new(); grid_config.total_box_count()],
            grid_config,
            left_flipper: FlipperEdge::new(FlipperSide::Left),
            right_flipper: FlipperEdge::new(FlipperSide::Right),
            restitution: 0.82,
        }
    }

    pub fn for_world_bounds(min_x: f32, min_y: f32, width: f32, height: f32) -> Self {
        let grid_config = GridConfig::new(min_x, min_y, width, height);
        Self {
            bounds: Vec::new(),
            walls: Vec::new(),
            grid: vec![Vec::new(); grid_config.total_box_count()],
            grid_config,
            left_flipper: FlipperEdge::new(FlipperSide::Left),
            right_flipper: FlipperEdge::new(FlipperSide::Right),
            restitution: 0.82,
        }
    }

    pub fn add_wall(&mut self, wall: EdgeSegment) {
        self.add_owned_wall(wall, None);
    }

    pub fn add_owned_wall(&mut self, wall: EdgeSegment, owner_token: Option<u32>) {
        let wall_index = self.walls.len();
        self.walls.push(RegisteredEdge {
            collider: RegisteredCollider::Segment(wall),
            owner_token,
            role: RegisteredEdgeRole::Solid,
        });
        self.register_wall_in_grid(wall_index);
    }

    pub fn add_owned_circle(&mut self, circle: EdgeCircle, owner_token: Option<u32>) {
        let wall_index = self.walls.len();
        self.walls.push(RegisteredEdge {
            collider: RegisteredCollider::Circle(circle),
            owner_token,
            role: RegisteredEdgeRole::Solid,
        });
        self.register_wall_in_grid(wall_index);
    }

    pub fn add_owned_trigger(&mut self, wall: EdgeSegment, owner_token: Option<u32>) {
        let wall_index = self.walls.len();
        self.walls.push(RegisteredEdge {
            collider: RegisteredCollider::Segment(wall),
            owner_token,
            role: RegisteredEdgeRole::Trigger,
        });
        self.register_wall_in_grid(wall_index);
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

    pub fn prepare_collision_pass(&self, ball: &mut Ball) {
        ball.begin_collision_pass();
    }

    pub fn resolve_ball_with_filter(
        &self,
        ball: &mut Ball,
        owner_is_active: impl FnMut(Option<u32>) -> bool,
    ) -> Option<CollisionContact> {
        self.resolve_ball_with_context(ball, owner_is_active, |_| {
            CollisionResponseParams {
                elasticity: self.restitution,
                ..CollisionResponseParams::default()
            }
        })
    }

    pub fn resolve_ball_with_context(
        &self,
        ball: &mut Ball,
        mut owner_is_active: impl FnMut(Option<u32>) -> bool,
        mut owner_response: impl FnMut(Option<u32>) -> CollisionResponseParams,
    ) -> Option<CollisionContact> {
        let candidate_walls = self.candidate_walls_for_ball(ball);
        for edge in self
            .bounds
            .iter()
            .copied()
            .enumerate()
            .map(|(index, segment)| {
                (
                    RegisteredCollider::Segment(segment),
                    None,
                    BOUND_EDGE_ID_BASE + index as u32,
                )
            })
            .chain(candidate_walls.into_iter().filter_map(|wall_index| {
                let wall = self.walls[wall_index];
                (wall.role == RegisteredEdgeRole::Solid).then_some((
                    wall.collider,
                    wall.owner_token,
                    WALL_EDGE_ID_BASE + wall_index as u32,
                ))
            }))
            .chain(
                [self.left_flipper.segment(), self.right_flipper.segment()]
                    .into_iter()
                    .enumerate()
                    .map(|(index, segment)| {
                        (
                            RegisteredCollider::Segment(segment),
                            None,
                            FLIPPER_EDGE_ID_BASE + index as u32,
                        )
                    }),
            )
        {
            if ball.already_hit(edge.2) {
                continue;
            }

            if !owner_is_active(edge.1) {
                continue;
            }

            let response = owner_response(edge.1);
            let contact = match edge.0 {
                RegisteredCollider::Segment(segment) => {
                    collide_ball_with_edge(ball, segment, response)
                }
                RegisteredCollider::Circle(circle) => {
                    collide_ball_with_circle(ball, circle, response)
                }
            };
            if let Some(contact) = contact {
                ball.remember_collision(edge.2);
                return Some(contact.with_owner(edge.1, CollisionEdgeRole::Solid));
            }
        }

        None
    }

    pub fn trigger_contacts_with_filter(
        &self,
        ball: &mut Ball,
        mut owner_is_active: impl FnMut(Option<u32>) -> bool,
    ) -> Vec<CollisionContact> {
        let mut contacts = Vec::new();
        for wall_index in self.candidate_walls_for_ball(ball) {
            let wall = self.walls[wall_index];
            if wall.role != RegisteredEdgeRole::Trigger {
                continue;
            }

            let collision_id = WALL_EDGE_ID_BASE + wall_index as u32;
            if ball.already_hit(collision_id) || !owner_is_active(wall.owner_token) {
                continue;
            }

            let contact = match wall.collider {
                RegisteredCollider::Segment(segment) => detect_ball_with_edge(ball, segment),
                RegisteredCollider::Circle(circle) => detect_ball_with_circle(ball, circle),
            };
            if let Some(contact) = contact {
                ball.remember_collision(collision_id);
                contacts.push(contact.with_owner(wall.owner_token, CollisionEdgeRole::Trigger));
            }
        }
        contacts
    }

    fn register_wall_in_grid(&mut self, wall_index: usize) {
        let wall = self.walls[wall_index];
        let (min, max) = match wall.collider {
            RegisteredCollider::Segment(segment) => (
                Vec2::new(segment.start.x.min(segment.end.x), segment.start.y.min(segment.end.y)),
                Vec2::new(segment.start.x.max(segment.end.x), segment.start.y.max(segment.end.y)),
            ),
            RegisteredCollider::Circle(circle) => (
                Vec2::new(circle.center.x - circle.radius, circle.center.y - circle.radius),
                Vec2::new(circle.center.x + circle.radius, circle.center.y + circle.radius),
            ),
        };

        let min_x = self.grid_config.box_x(min.x.clamp(self.grid_config.min_x, self.grid_config.max_x));
        let max_x = self.grid_config.box_x(max.x.clamp(self.grid_config.min_x, self.grid_config.max_x));
        let min_y = self.grid_config.box_y(min.y.clamp(self.grid_config.min_y, self.grid_config.max_y));
        let max_y = self.grid_config.box_y(max.y.clamp(self.grid_config.min_y, self.grid_config.max_y));

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.grid[self.grid_config.box_index(x, y)].push(wall_index);
            }
        }
    }

    fn candidate_walls_for_ball(&self, ball: &Ball) -> Vec<usize> {
        let min = Vec2::new(ball.position.x - ball.radius, ball.position.y - ball.radius);
        let max = Vec2::new(ball.position.x + ball.radius, ball.position.y + ball.radius);
        let min_x = self.grid_config.box_x(min.x.clamp(self.grid_config.min_x, self.grid_config.max_x));
        let max_x = self.grid_config.box_x(max.x.clamp(self.grid_config.min_x, self.grid_config.max_x));
        let min_y = self.grid_config.box_y(min.y.clamp(self.grid_config.min_y, self.grid_config.max_y));
        let max_y = self.grid_config.box_y(max.y.clamp(self.grid_config.min_y, self.grid_config.max_y));

        let mut candidates = Vec::new();
        let mut seen = vec![false; self.walls.len()];
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for &wall_index in &self.grid[self.grid_config.box_index(x, y)] {
                    if !seen[wall_index] {
                        seen[wall_index] = true;
                        candidates.push(wall_index);
                    }
                }
            }
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::math::Vec2;
    use crate::engine::physics::{
        Ball, CollisionEdgeRole, CollisionResponseParams, EdgeCircle, EdgeSegment,
    };

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
        edge_manager.prepare_collision_pass(&mut active_ball);
        let active_contact =
            edge_manager.resolve_ball_with_filter(&mut active_ball, |owner_token| {
                owner_token != Some(7) || true
        });
        assert!(active_contact.is_some());

        let mut inactive_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        inactive_ball.velocity = Vec2::new(0.0, 20.0);
        edge_manager.prepare_collision_pass(&mut inactive_ball);
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
        edge_manager.prepare_collision_pass(&mut ball);
        let contact = edge_manager.resolve_ball_with_filter(&mut ball, |_| true);
        assert!(contact.is_none());
    }

    #[test]
    fn world_bounds_grid_supports_negative_coordinate_collision_space() {
        let mut edge_manager = EdgeManager::for_world_bounds(-20.0, -30.0, 80.0, 90.0);
        edge_manager.add_owned_wall(
            EdgeSegment::new(Vec2::new(-5.0, 10.0), Vec2::new(15.0, 10.0)),
            Some(4),
        );

        let mut ball = Ball::ready_at(Vec2::new(5.0, 4.0));
        ball.velocity = Vec2::new(0.0, 20.0);
        edge_manager.prepare_collision_pass(&mut ball);

        let contact = edge_manager.resolve_ball_with_filter(&mut ball, |_| true);
        assert!(contact.is_some());
        assert_eq!(contact.and_then(|contact| contact.owner_token), Some(4));
    }

    #[test]
    fn owned_circles_collide_like_solid_walls() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_circle(EdgeCircle::new(Vec2::new(150.0, 200.0), 4.0), Some(9));

        let mut ball = Ball::ready_at(Vec2::new(150.0, 191.0));
        ball.velocity = Vec2::new(0.0, 20.0);
        edge_manager.prepare_collision_pass(&mut ball);
        let contact = edge_manager.resolve_ball_with_filter(&mut ball, |_| true);
        assert!(contact.is_some());
    }

    #[test]
    fn broad_phase_candidates_come_from_overlapping_grid_boxes() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_wall(
            EdgeSegment::new(Vec2::new(40.0, 40.0), Vec2::new(80.0, 40.0)),
            Some(1),
        );
        edge_manager.add_owned_wall(
            EdgeSegment::new(Vec2::new(520.0, 360.0), Vec2::new(560.0, 360.0)),
            Some(2),
        );
        edge_manager.add_owned_circle(EdgeCircle::new(Vec2::new(540.0, 340.0), 8.0), Some(3));

        let near_origin = Ball::ready_at(Vec2::new(60.0, 44.0));
        let near_far_corner = Ball::ready_at(Vec2::new(540.0, 350.0));

        let origin_candidates = edge_manager.candidate_walls_for_ball(&near_origin);
        let far_candidates = edge_manager.candidate_walls_for_ball(&near_far_corner);

        assert_eq!(origin_candidates, vec![0]);
        assert_eq!(far_candidates, vec![1, 2]);
    }

    #[test]
    fn recent_collision_memory_skips_immediate_repeat_until_it_expires() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_wall(
            EdgeSegment::new(Vec2::new(100.0, 200.0), Vec2::new(200.0, 200.0)),
            Some(7),
        );

        let mut ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        ball.velocity = Vec2::new(0.0, 20.0);

        edge_manager.prepare_collision_pass(&mut ball);
        let first = edge_manager.resolve_ball_with_filter(&mut ball, |_| true);
        assert!(first.is_some());

        ball.position = Vec2::new(150.0, 194.0);
        ball.velocity = Vec2::new(0.0, 20.0);
        edge_manager.prepare_collision_pass(&mut ball);
        let second = edge_manager.resolve_ball_with_filter(&mut ball, |_| true);
        assert!(second.is_none());

        ball.position = Vec2::new(150.0, 194.0);
        ball.velocity = Vec2::new(0.0, 20.0);
        edge_manager.prepare_collision_pass(&mut ball);
        let third = edge_manager.resolve_ball_with_filter(&mut ball, |_| true);
        assert!(third.is_some());
    }

    #[test]
    fn trigger_contacts_report_owner_without_physical_collision() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_trigger(
            EdgeSegment::new(Vec2::new(100.0, 200.0), Vec2::new(200.0, 200.0)),
            Some(11),
        );

        let mut ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        ball.velocity = Vec2::new(0.0, 20.0);

        edge_manager.prepare_collision_pass(&mut ball);
        let contacts = edge_manager.trigger_contacts_with_filter(&mut ball, |_| true);
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].owner_token, Some(11));
        assert_eq!(contacts[0].edge_role, CollisionEdgeRole::Trigger);
    }

    #[test]
    fn owned_edges_can_override_restitution_from_context() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_wall(
            EdgeSegment::new(Vec2::new(100.0, 200.0), Vec2::new(200.0, 200.0)),
            Some(12),
        );

        let mut soft_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        soft_ball.velocity = Vec2::new(0.0, 20.0);
        edge_manager.prepare_collision_pass(&mut soft_ball);
        let _ = edge_manager.resolve_ball_with_context(&mut soft_ball, |_| true, |owner| {
            if owner == Some(12) {
                CollisionResponseParams {
                    elasticity: 0.1,
                    ..CollisionResponseParams::default()
                }
            } else {
                CollisionResponseParams {
                    elasticity: 0.82,
                    ..CollisionResponseParams::default()
                }
            }
        });

        let mut bouncy_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        bouncy_ball.velocity = Vec2::new(0.0, 20.0);
        edge_manager.prepare_collision_pass(&mut bouncy_ball);
        let _ = edge_manager.resolve_ball_with_context(&mut bouncy_ball, |_| true, |owner| {
            if owner == Some(12) {
                CollisionResponseParams {
                    elasticity: 1.0,
                    ..CollisionResponseParams::default()
                }
            } else {
                CollisionResponseParams {
                    elasticity: 0.82,
                    ..CollisionResponseParams::default()
                }
            }
        });

        assert!(soft_ball.velocity.y < 0.0);
        assert!(bouncy_ball.velocity.y < soft_ball.velocity.y);
    }

    #[test]
    fn owned_edges_can_apply_threshold_boost_and_tangent_smoothing() {
        let mut edge_manager = EdgeManager::for_table_bounds(600.0, 416.0);
        edge_manager.add_owned_wall(
            EdgeSegment::new(Vec2::new(100.0, 200.0), Vec2::new(200.0, 200.0)),
            Some(13),
        );

        let mut baseline_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        baseline_ball.velocity = Vec2::new(10.0, 20.0);
        edge_manager.prepare_collision_pass(&mut baseline_ball);
        let _ = edge_manager.resolve_ball_with_context(&mut baseline_ball, |_| true, |_| {
            CollisionResponseParams {
                elasticity: 0.6,
                smoothness: 1.0,
                threshold: f32::MAX,
                boost: 0.0,
            }
        });

        let mut boosted_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        boosted_ball.velocity = Vec2::new(10.0, 20.0);
        edge_manager.prepare_collision_pass(&mut boosted_ball);
        let _ = edge_manager.resolve_ball_with_context(&mut boosted_ball, |_| true, |_| {
            CollisionResponseParams {
                elasticity: 0.6,
                smoothness: 0.5,
                threshold: 5.0,
                boost: 8.0,
            }
        });

        assert!(boosted_ball.velocity.x.abs() < baseline_ball.velocity.x.abs());
        assert!(boosted_ball.velocity.y < baseline_ball.velocity.y);
    }
}
