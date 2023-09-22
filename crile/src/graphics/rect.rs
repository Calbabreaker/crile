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

    pub fn constrain(&mut self, size: glam::Vec2) {
        self.w = self.w.clamp(0., size.x);
        self.h = self.h.clamp(0., size.y);
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
