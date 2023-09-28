use std::sync::atomic::{AtomicI32, AtomicU32};

pub struct TransformComponent {
    translation: glam::Vec3,
    rotation: glam::Vec3,
    scale: glam::Vec3,
}

impl TransformComponent {
    pub fn matrix(&self) -> glam::Mat4 {
        f32::INFINITY;
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            glam::Quat::from_scaled_axis(self.rotation),
            self.translation,
        )
    }
}
