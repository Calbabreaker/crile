use serde::{Deserialize, Serialize};

use crate::{Color, EntityId, RefId, Script, Texture};

/// Component to contain meta data about the enitity
/// Not really a real component but used internally to keep track of metadata
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct MetaDataComponent {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<EntityId>,

    #[serde(skip_serializing_if = "default")]
    pub parent: EntityId,
}

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
    pub projection: ProjectionKind,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            viewport_size: Default::default(),
            near: -1.0,
            far: 1.0,
            perspective_fov: 45.,
            orthographic_zoom: 1.,
            projection: ProjectionKind::default(),
        }
    }
}

impl CameraComponent {
    pub fn projection(&self) -> glam::Mat4 {
        match self.projection {
            ProjectionKind::Perspective => glam::Mat4::perspective_rh(
                self.perspective_fov.to_radians(),
                self.viewport_size.x / self.viewport_size.y,
                self.near.max(0.001),
                self.far,
            ),
            ProjectionKind::Orthographic => {
                let size_half = self.viewport_size / self.orthographic_zoom / 2.;
                glam::Mat4::orthographic_rh(
                    -size_half.x,
                    size_half.x,
                    size_half.y,
                    -size_half.y,
                    self.near,
                    self.far,
                )
            }
        }
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
