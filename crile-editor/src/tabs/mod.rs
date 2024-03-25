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
                crile::SpriteComponent {
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

impl EditorState {
    pub fn save_scene(&mut self) {
        if let Ok(data) = crile::SceneSerializer::serialize(&self.scene)
            .inspect_err(|err| log::error!("Failed to save scene: {err}"))
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Scene", &["scene"])
                .set_file_name("test.scene")
                .set_directory(std::env::current_dir().unwrap_or("/".into()))
                .save_file()
            {
                std::fs::write(&path, data)
                    .inspect_err(|err| log::error!("Failed to save {path:?}: {err}"))
                    .ok();
            }
        }
    }

    pub fn load_scene(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Scene", &["scene"])
            .set_directory(std::env::current_dir().unwrap_or("/".into()))
            .pick_file()
        {
            if let Ok(source) = std::fs::read_to_string(&path)
                .inspect_err(|err| log::error!("Failed to load {path:?}: {err}"))
            {
                if let Ok(scene) = crile::SceneSerializer::deserialize(source)
                    .inspect_err(|err| log::error!("Failed to load scene: {err} "))
                {
                    self.scene = scene;
                }
            }
        }
    }
}
