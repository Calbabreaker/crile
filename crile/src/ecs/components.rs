use crate::{Camera, Color};

#[derive(Debug)]
pub struct TransformComponent {
    pub translation: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            translation: glam::Vec3::ZERO,
            rotation: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
        }
    }
}

impl TransformComponent {
    pub fn get_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            glam::Quat::from_scaled_axis(self.rotation),
            self.translation,
        )
    }
}

#[derive(Default)]
pub struct SpriteRendererComponent {
    pub color: Color,
}

#[derive(Default)]
pub struct CameraComponent {
    pub camera: Camera,
}
