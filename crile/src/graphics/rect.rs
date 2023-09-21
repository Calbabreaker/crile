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
        self.w = f32::min(size.x - self.x, self.w);
        self.h = f32::min(size.y - self.h, self.h);
    }
}
