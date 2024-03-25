use crate::{Color, EntityId, RefId, Texture};

#[derive(Debug, Default)]
pub struct MetaDataComponent {
    pub name: String,
    pub children: Vec<EntityId>,
    pub parent: EntityId,
}

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

#[derive(Debug, Default)]
pub struct SpriteComponent {
    pub color: Color,
    pub texture: Option<RefId<Texture>>,
    pub texture_path: Option<std::path::PathBuf>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ProjectionKind {
    Perspective,
    #[default]
    Orthographic,
}

#[derive(Debug)]
pub struct CameraComponent {
    aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub ortho_size: f32,
    /// Vertical field-of-view of the camera
    pub fov: f32,
    pub projection_kind: ProjectionKind,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            aspect_ratio: 0.0,
            near: -1.0,
            far: 1.0,
            ortho_size: 5.0,
            fov: 45.,
            projection_kind: ProjectionKind::default(),
        }
    }
}

impl CameraComponent {
    pub fn new(viewport_size: glam::Vec2) -> Self {
        Self {
            aspect_ratio: viewport_size.x / viewport_size.y,
            ..Default::default()
        }
    }

    pub fn set_viewport(&mut self, viewport_size: glam::Vec2) {
        self.aspect_ratio = viewport_size.x / viewport_size.y;
    }

    pub fn projection(&self) -> glam::Mat4 {
        match self.projection_kind {
            ProjectionKind::Perspective => glam::Mat4::perspective_rh(
                self.fov.to_radians(),
                self.aspect_ratio,
                self.near.max(0.001),
                self.far,
            ),
            ProjectionKind::Orthographic => {
                let left = -self.ortho_size * self.aspect_ratio;
                let right = self.ortho_size * self.aspect_ratio;
                let bottom = -self.ortho_size;
                let top = self.ortho_size;
                glam::Mat4::orthographic_rh(left, right, bottom, top, self.near, self.far)
            }
        }
    }
}

/// Calls a macro with all the components crile has
#[macro_export]
macro_rules! with_components {
    ($macro: ident) => {{
        use ::crile::*;
        $macro!([TransformComponent, CameraComponent, SpriteComponent])
    }};
}
