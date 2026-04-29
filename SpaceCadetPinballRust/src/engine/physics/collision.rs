use crate::engine::math::Vec2;
use crate::engine::physics::{Ball, EdgeCircle, EdgeSegment};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionResponseParams {
    pub elasticity: f32,
    pub smoothness: f32,
    pub threshold: f32,
    pub boost: f32,
}

impl Default for CollisionResponseParams {
    fn default() -> Self {
        Self {
            elasticity: 0.82,
            smoothness: 0.95,
            threshold: f32::MAX,
            boost: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CollisionEdgeRole {
    #[default]
    Solid,
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionContact {
    pub point: Vec2,
    pub normal: Vec2,
    pub distance: f32,
    pub impact_speed: f32,
    pub threshold_exceeded: bool,
    pub owner_token: Option<u32>,
    pub edge_role: CollisionEdgeRole,
}

impl CollisionContact {
    pub const fn new(point: Vec2, normal: Vec2, distance: f32) -> Self {
        Self {
            point,
            normal,
            distance,
            impact_speed: 0.0,
            threshold_exceeded: false,
            owner_token: None,
            edge_role: CollisionEdgeRole::Solid,
        }
    }

    pub const fn with_owner(mut self, owner_token: Option<u32>, edge_role: CollisionEdgeRole) -> Self {
        self.owner_token = owner_token;
        self.edge_role = edge_role;
        self
    }
}

fn edge_contact(ball: &Ball, edge: EdgeSegment, require_incoming: bool) -> Option<CollisionContact> {
    let edge_delta = edge.direction();
    let edge_length_sq = edge_delta.length_squared();
    if edge_length_sq == 0.0 {
        return None;
    }

    let to_center = Vec2::new(
        ball.position.x - edge.start.x,
        ball.position.y - edge.start.y,
    );
    let projection = ((to_center.x * edge_delta.x) + (to_center.y * edge_delta.y)) / edge_length_sq;
    let t = projection.clamp(0.0, 1.0);
    let closest = Vec2::new(
        edge.start.x + edge_delta.x * t,
        edge.start.y + edge_delta.y * t,
    );
    let separation = Vec2::new(ball.position.x - closest.x, ball.position.y - closest.y);
    let distance_sq = separation.length_squared();
    if distance_sq > ball.radius * ball.radius {
        return None;
    }

    let normal = if distance_sq > 0.0001 {
        let distance = distance_sq.sqrt();
        Vec2::new(separation.x / distance, separation.y / distance)
    } else {
        let fallback = Vec2::new(-edge_delta.y, edge_delta.x);
        let fallback_length = fallback.length_squared().sqrt();
        Vec2::new(fallback.x / fallback_length, fallback.y / fallback_length)
    };

    let incoming_speed = ball.velocity.x * normal.x + ball.velocity.y * normal.y;
    if require_incoming && incoming_speed >= 0.0 {
        return None;
    }

    let distance = distance_sq.sqrt();
    Some(CollisionContact::new(
        closest,
        normal,
        (ball.radius - distance).max(0.0),
    ))
}

fn circle_contact(ball: &Ball, circle: EdgeCircle, require_incoming: bool) -> Option<CollisionContact> {
    let separation = Vec2::new(
        ball.position.x - circle.center.x,
        ball.position.y - circle.center.y,
    );
    let distance_sq = separation.length_squared();
    let collision_radius = ball.radius + circle.radius;
    if distance_sq > collision_radius * collision_radius {
        return None;
    }

    let normal = if distance_sq > 0.0001 {
        let distance = distance_sq.sqrt();
        Vec2::new(separation.x / distance, separation.y / distance)
    } else {
        Vec2::new(0.0, -1.0)
    };

    let incoming_speed = ball.velocity.x * normal.x + ball.velocity.y * normal.y;
    if require_incoming && incoming_speed >= 0.0 {
        return None;
    }

    let distance = distance_sq.sqrt();
    let contact_point = Vec2::new(
        circle.center.x + normal.x * circle.radius,
        circle.center.y + normal.y * circle.radius,
    );
    Some(CollisionContact::new(
        contact_point,
        normal,
        (collision_radius - distance).max(0.0),
    ))
}

pub fn detect_ball_with_edge(ball: &Ball, edge: EdgeSegment) -> Option<CollisionContact> {
    edge_contact(ball, edge, true)
}

pub fn collide_ball_with_edge(
    ball: &mut Ball,
    edge: EdgeSegment,
    response: CollisionResponseParams,
) -> Option<CollisionContact> {
    let contact = edge_contact(ball, edge, true)?;
    Some(apply_collision_response(ball, contact, response))
}

pub fn detect_ball_with_circle(ball: &Ball, circle: EdgeCircle) -> Option<CollisionContact> {
    circle_contact(ball, circle, true)
}

pub fn collide_ball_with_circle(
    ball: &mut Ball,
    circle: EdgeCircle,
    response: CollisionResponseParams,
) -> Option<CollisionContact> {
    let contact = circle_contact(ball, circle, true)?;
    Some(apply_collision_response(ball, contact, response))
}

fn apply_collision_response(
    ball: &mut Ball,
    mut contact: CollisionContact,
    response: CollisionResponseParams,
) -> CollisionContact {
    let incoming_speed = ball.velocity.x * contact.normal.x + ball.velocity.y * contact.normal.y;
    let normal_velocity = Vec2::new(contact.normal.x * incoming_speed, contact.normal.y * incoming_speed);
    let tangent_velocity = Vec2::new(
        ball.velocity.x - normal_velocity.x,
        ball.velocity.y - normal_velocity.y,
    );
    let impact_speed = -incoming_speed;

    ball.position.x += contact.normal.x * (contact.distance + 0.0005);
    ball.position.y += contact.normal.y * (contact.distance + 0.0005);

    let reflected_normal = Vec2::new(
        -normal_velocity.x * response.elasticity,
        -normal_velocity.y * response.elasticity,
    );
    let smoothed_tangent = Vec2::new(
        tangent_velocity.x * response.smoothness,
        tangent_velocity.y * response.smoothness,
    );
    ball.velocity = Vec2::new(
        reflected_normal.x + smoothed_tangent.x,
        reflected_normal.y + smoothed_tangent.y,
    );

    if impact_speed >= response.threshold {
        ball.velocity.x += contact.normal.x * response.boost;
        ball.velocity.y += contact.normal.y * response.boost;
    }

    contact.impact_speed = impact_speed;
    contact.threshold_exceeded = impact_speed >= response.threshold;
    contact
}

#[cfg(test)]
mod tests {
    use crate::engine::math::Vec2;
    use crate::engine::physics::{Ball, EdgeSegment};

    use super::{CollisionResponseParams, collide_ball_with_edge};

    #[test]
    fn collision_response_marks_threshold_exceeded() {
        let edge = EdgeSegment::new(Vec2::new(100.0, 200.0), Vec2::new(200.0, 200.0));
        let mut soft_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        soft_ball.velocity = Vec2::new(0.0, 20.0);
        let mut hard_ball = Ball::ready_at(Vec2::new(150.0, 194.0));
        hard_ball.velocity = Vec2::new(0.0, 20.0);

        let soft = collide_ball_with_edge(
            &mut soft_ball,
            edge,
            CollisionResponseParams {
                threshold: 25.0,
                ..CollisionResponseParams::default()
            },
        )
        .expect("soft collision should resolve");
        let hard = collide_ball_with_edge(
            &mut hard_ball,
            edge,
            CollisionResponseParams {
                threshold: 5.0,
                ..CollisionResponseParams::default()
            },
        )
        .expect("hard collision should resolve");

        assert!(!soft.threshold_exceeded);
        assert!(hard.threshold_exceeded);
        assert!(hard.impact_speed > soft.impact_speed - 0.001);
    }
}
