use crate::engine::math::Vec2;
use crate::engine::physics::{Ball, EdgeCircle, EdgeSegment};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionContact {
    pub point: Vec2,
    pub normal: Vec2,
    pub distance: f32,
}

impl CollisionContact {
    pub const fn new(point: Vec2, normal: Vec2, distance: f32) -> Self {
        Self {
            point,
            normal,
            distance,
        }
    }
}

pub fn collide_ball_with_edge(
    ball: &mut Ball,
    edge: EdgeSegment,
    restitution: f32,
) -> Option<CollisionContact> {
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
    if incoming_speed >= 0.0 {
        return None;
    }

    let distance = distance_sq.sqrt();
    let penetration = (ball.radius - distance).max(0.0);
    ball.position.x += normal.x * penetration;
    ball.position.y += normal.y * penetration;
    ball.velocity.x -= (1.0 + restitution) * incoming_speed * normal.x;
    ball.velocity.y -= (1.0 + restitution) * incoming_speed * normal.y;

    Some(CollisionContact::new(closest, normal, penetration))
}

pub fn collide_ball_with_circle(
    ball: &mut Ball,
    circle: EdgeCircle,
    restitution: f32,
) -> Option<CollisionContact> {
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
    if incoming_speed >= 0.0 {
        return None;
    }

    let distance = distance_sq.sqrt();
    let penetration = (collision_radius - distance).max(0.0);
    ball.position.x += normal.x * penetration;
    ball.position.y += normal.y * penetration;
    ball.velocity.x -= (1.0 + restitution) * incoming_speed * normal.x;
    ball.velocity.y -= (1.0 + restitution) * incoming_speed * normal.y;

    let contact_point = Vec2::new(
        circle.center.x + normal.x * circle.radius,
        circle.center.y + normal.y * circle.radius,
    );
    Some(CollisionContact::new(contact_point, normal, penetration))
}
