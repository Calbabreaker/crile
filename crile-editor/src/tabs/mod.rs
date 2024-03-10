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
    pub viewport_size: glam::Vec2,
    pub selection: Selection,
}

impl Default for EditorState {
    fn default() -> Self {
        let mut scene = crile::Scene::default();

        scene.spawn(
            (
                crile::MetaDataComponent {
                    name: "Camera".to_string(),
                    ..Default::default()
                },
                crile::TransformComponent::default(),
                crile::CameraComponent::default(),
            ),
            None,
        );

        scene.spawn(
            (
                crile::MetaDataComponent {
                    name: "Sprite".to_string(),
                    ..Default::default()
                },
                crile::TransformComponent::default(),
                crile::SpriteRendererComponent {
                    color: crile::Color::from_rgb(99, 123, 255),
                },
            ),
            None,
        );

        Self {
            scene,
            selection: Selection::None,
            viewport_texture_id: None,
            viewport_size: glam::Vec2::ZERO,
        }
    }
}
