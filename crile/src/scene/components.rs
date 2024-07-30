use serde::{Deserialize, Serialize};

use crate::{Color, RefId, Script, Texture};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TransformComponent {
    #[serde(skip_serializing_if = "default")]
    pub translation: glam::Vec3,

    #[serde(skip_serializing_if = "default")]
    pub rotation: glam::Vec3,

    #[serde(skip_serializing_if = "default")]
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
    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            glam::Quat::from_scaled_axis(self.rotation),
            self.translation,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct SpriteComponent {
    #[serde(skip_serializing_if = "default")]
    pub color: Color,
    #[serde(skip)]
    pub texture: Option<RefId<Texture>>,
    pub texture_path: AssetPath,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ProjectionKind {
    Perspective,
    #[default]
    Orthographic,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CameraComponent {
    #[serde(skip)]
    pub viewport_size: glam::Vec2,

    #[serde(skip)]
    pub projection: glam::Mat4,

    #[serde(skip)]
    pub view_projection: glam::Mat4,

    #[serde(skip)]
    pub dirty: bool,

    #[serde(skip_serializing_if = "default")]
    pub near: f32,

    #[serde(skip_serializing_if = "default")]
    pub far: f32,

    #[serde(skip_serializing_if = "default")]
    pub orthographic_zoom: f32,

    /// Vertical field-of-view of the camera
    #[serde(skip_serializing_if = "default")]
    pub perspective_fov: f32,

    #[serde(skip_serializing_if = "default")]
    pub projection_kind: ProjectionKind,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            viewport_size: Default::default(),
            near: -1.0,
            far: 1.0,
            dirty: true,
            perspective_fov: 45.,
            orthographic_zoom: 1.,
            projection: glam::Mat4::IDENTITY,
            view_projection: glam::Mat4::IDENTITY,
            projection_kind: ProjectionKind::default(),
        }
    }
}

impl CameraComponent {
    pub fn update_projection(&mut self, transform: glam::Mat4) {
        if !self.dirty {
            return;
        }

        match self.projection_kind {
            ProjectionKind::Perspective => {
                self.projection = glam::Mat4::perspective_rh(
                    self.perspective_fov.to_radians(),
                    self.viewport_size.x / self.viewport_size.y,
                    self.near.max(0.001),
                    self.far,
                );
            }
            ProjectionKind::Orthographic => {
                let size_half = self.viewport_size / self.orthographic_zoom / 2.;
                self.projection = glam::Mat4::orthographic_rh(
                    -size_half.x,
                    size_half.x,
                    size_half.y,
                    -size_half.y,
                    self.near,
                    self.far,
                );
            }
        }

        self.view_projection = self.projection * transform.inverse();
        self.dirty = false;
    }

    pub fn screen_to_world(&self, screen: glam::Vec2) -> glam::Vec2 {
        // Make position go from -1 to +1
        let mut normalized = screen / self.viewport_size * 2.;
        normalized.x -= 1.;
        normalized.y = -normalized.y + 1.;

        let mut clip = glam::Vec4::new(normalized.x, normalized.y, -1., 1.);
        clip = self.view_projection.inverse().mul_vec4(clip);
        glam::Vec2::new(clip.x / clip.w, clip.y / clip.w)
    }
}

#[derive(Serialize, Default, Deserialize, Clone)]
pub struct ScriptComponent {
    #[serde(skip)]
    pub script: Option<RefId<Script>>,
    pub script_path: AssetPath,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct AssetPath {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<std::path::PathBuf>,

    #[serde(skip)]
    pub open_picker: bool,
}

/// Calls a macro with all the components crile has excluding MetaDataComponent
#[macro_export]
macro_rules! with_components {
    ($macro: ident) => {{
        use $crate::*;
        $macro!([
            TransformComponent,
            CameraComponent,
            SpriteComponent,
            ScriptComponent
        ])
    }};
}

fn default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
}
