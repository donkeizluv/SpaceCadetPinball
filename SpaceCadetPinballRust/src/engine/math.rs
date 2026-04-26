#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn clamp_length(self, max_length: f32) -> Self {
        let max_length_sq = max_length * max_length;
        let length_sq = self.length_squared();
        if length_sq <= max_length_sq || length_sq == 0.0 {
            return self;
        }

        let scale = max_length / length_sq.sqrt();
        Self::new(self.x * scale, self.y * scale)
    }
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * clamp(t, 0.0, 1.0)
}
