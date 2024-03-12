#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_pos_size(pos: glam::Vec2, size: glam::Vec2) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            w: size.x,
            h: size.y,
        }
    }

    pub fn constrain(&mut self, size: glam::Vec2) {
        self.w = f32::min(self.w, size.x - self.x);
        self.h = f32::min(self.h, size.y - self.y);
        self.x = f32::min(self.x, size.x);
        self.y = f32::min(self.y, size.y);
    }

    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::orthographic_lh(
            self.left(),
            self.right(),
            self.bottom(),
            self.top(),
            0.0,
            1.0,
        )
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

impl std::ops::Mul<f32> for Rect {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.w * rhs, self.h * rhs)
    }
}
