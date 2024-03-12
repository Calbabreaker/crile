pub mod hierarchy;
pub mod inspector;
pub mod viewport;

#[derive(PartialEq, Eq, Debug)]
pub enum Selection {
    Entity(crile::EntityId),
    None,
}

pub struct EditorState {
    pub scene: crile::Scene,
    pub viewport_texture_id: Option<egui::TextureId>,
    pub viewport_size: glam::UVec2,
    pub selection: Selection,
    pub viewport_texture: Option<crile::RefId<crile::Texture>>,
    pub depth_texture: Option<crile::Texture>,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut scene = crile::Scene::default();

        scene.spawn(
            "Camera",
            (
                crile::TransformComponent::default(),
                crile::CameraComponent::default(),
            ),
            None,
        );

        scene.spawn(
            "Sprite",
            (
                crile::TransformComponent::default(),
                crile::SpriteRendererComponent {
                    color: crile::Color::from_rgb(99, 123, 255),
                    ..Default::default()
                },
            ),
            None,
        );

        Self {
            scene,
            selection: Selection::None,
            viewport_texture_id: None,
            viewport_size: glam::UVec2::ZERO,
            viewport_texture: None,
            depth_texture: None,
        }
    }
}
