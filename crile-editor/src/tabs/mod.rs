mod hierarchy;
mod inspector;
mod viewport;

#[derive(Debug)]
pub enum Tab {
    Hierarchy,
    Viewport,
    Inspector,
}

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

impl egui_dock::TabViewer for EditorState {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{tab:?}").into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Hierarchy => hierarchy::show(self, ui),
            Tab::Viewport => viewport::show(self, ui),
            Tab::Inspector => inspector::show(self, ui),
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        let mut scene = crile::Scene::default();
        scene.world.spawn((
            crile::MetaDataComponent {
                name: "Camera".to_string(),
            },
            crile::TransformComponent::default(),
            crile::CameraComponent::default(),
        ));

        scene.world.spawn((
            crile::MetaDataComponent {
                name: "Sprite".to_string(),
            },
            crile::TransformComponent::default(),
            crile::SpriteRendererComponent {
                color: crile::Color::from_rgb(255, 0, 0),
            },
        ));

        Self {
            scene,
            selection: Selection::None,
            viewport_texture_id: None,
            viewport_size: glam::Vec2::ZERO,
        }
    }
}
