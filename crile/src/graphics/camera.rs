use crate::{Matrix4, Vector2};

pub struct Camera {
    aspect_ratio: f32,
    near: f32,
    far: f32,
    ortho_size: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            aspect_ratio: 0.0,
            near: -1.0,
            far: 1.0,
            ortho_size: 5.0,
        }
    }
}

impl Camera {
    pub fn new(viewport_size: Vector2) -> Self {
        Self {
            aspect_ratio: viewport_size.x / viewport_size.y,
            ..Default::default()
        }
    }

    pub fn resize(&mut self, viewport_size: Vector2) {
        self.aspect_ratio = viewport_size.x / viewport_size.y;
    }

    pub fn get_projection(&self) -> Matrix4 {
        let left = -self.ortho_size * self.aspect_ratio;
        let right = self.ortho_size * self.aspect_ratio;
        let bottom = -self.ortho_size;
        let top = self.ortho_size;
        Matrix4::orthographic_lh(left, right, bottom, top, self.near, self.far)
    }
}
