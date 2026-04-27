use crate::engine::geom::{Circle, RectI};
use crate::engine::math::{Vec2, clamp};

const GRAVITY: f32 = 900.0;
const FLIPPER_KICK_X: f32 = 240.0;
const FLIPPER_KICK_Y: f32 = -180.0;

#[derive(Debug, Clone)]
pub struct Ball {
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    launched: bool,
}

impl Ball {
    pub fn ready_in_launch_lane() -> Self {
        Self::ready_at(Vec2::new(560.0, 382.0))
    }

    pub fn ready_at(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            radius: 6.0,
            launched: false,
        }
    }

    pub fn launch(&mut self, charge: f32) {
        let charge = clamp(charge, 0.15, 1.0);
        self.velocity = Vec2::new(0.0, -(380.0 + 420.0 * charge));
        self.launched = true;
    }

    pub fn is_launched(&self) -> bool {
        self.launched
    }

    pub fn apply_nudge(&mut self, impulse: Vec2) {
        self.velocity.x += impulse.x * 220.0;
        self.velocity.y -= impulse.y * 180.0;
    }

    pub fn apply_flipper_impulse(&mut self, left_active: bool, right_active: bool) {
        if self.position.y < 300.0 {
            return;
        }

        if left_active && self.position.x < 310.0 {
            self.velocity.x += FLIPPER_KICK_X;
            self.velocity.y += FLIPPER_KICK_Y;
        }

        if right_active && self.position.x > 290.0 {
            self.velocity.x -= FLIPPER_KICK_X;
            self.velocity.y += FLIPPER_KICK_Y;
        }
    }

    pub fn step(&mut self, dt: f32) {
        if !self.launched {
            self.velocity = Vec2::ZERO;
            return;
        }

        self.velocity.y += GRAVITY * dt;
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;
        self.velocity.x *= 0.992;
    }

    pub fn is_drained(&self, drain_y: f32) -> bool {
        self.position.y - self.radius > drain_y
    }

    pub fn bounds(&self) -> RectI {
        RectI::new(
            (self.position.x - self.radius).round() as i32,
            (self.position.y - self.radius).round() as i32,
            (self.radius * 2.0).round() as u32,
            (self.radius * 2.0).round() as u32,
        )
    }

    pub fn shape(&self) -> Circle {
        Circle {
            center: self.position,
            radius: self.radius,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::math::Vec2;

    use super::Ball;

    #[test]
    fn ready_ball_does_not_fall_before_launch() {
        let mut ball = Ball::ready_at(Vec2::new(100.0, 200.0));

        ball.step(1.0);

        assert_eq!(ball.position, Vec2::new(100.0, 200.0));
        assert_eq!(ball.velocity, Vec2::ZERO);
        assert!(!ball.is_launched());
    }
}
