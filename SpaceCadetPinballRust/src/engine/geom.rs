use sdl2::rect::Rect;

use super::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RectI {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl RectI {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(self) -> i32 {
        self.x.saturating_add_unsigned(self.width)
    }

    pub fn bottom(self) -> i32 {
        self.y.saturating_add_unsigned(self.height)
    }

    pub fn center(self) -> Vec2 {
        Vec2::new(
            self.x as f32 + self.width as f32 * 0.5,
            self.y as f32 + self.height as f32 * 0.5,
        )
    }

    pub fn to_sdl_rect(self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    pub fn contains(self, point: Vec2) -> bool {
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        dx * dx + dy * dy <= self.radius * self.radius
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec2,
    pub direction: Vec2,
}

impl Ray {
    pub fn point_at(self, distance: f32) -> Vec2 {
        Vec2::new(
            self.origin.x + self.direction.x * distance,
            self.origin.y + self.direction.y * distance,
        )
    }
}
