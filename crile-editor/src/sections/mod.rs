use std::path::PathBuf;

use crate::{editor_camera::EditorCamera2D, project::Project, Preferences};

pub mod hierarchy;
pub mod inspector;
pub mod top_panel;
pub mod viewport;

#[derive(PartialEq, Eq, Debug)]
pub enum Selection {
    Entity(crile::EntityId),
    None,
}

#[derive(Eq, Hash, PartialEq)]
pub enum WindowKind {
    Preferences,
    None,
}

#[derive(Eq, PartialEq)]
pub enum SceneState {
    Editing,
    Running,
}

pub struct EditorState {
    pub scene: crile::Scene,
    pub scene_state: SceneState,
    pub editor_scene_path: Option<PathBuf>,
    pub backup_scene: Option<crile::Scene>,

    pub selection: Selection,
    pub depth_texture: Option<crile::Texture>,
    pub editor_camera: EditorCamera2D,
    pub project: Project,

    pub viewport_texture_id: Option<egui::TextureId>,
    pub viewport_size: glam::UVec2,
    pub viewport_texture: Option<crile::RefId<crile::Texture>>,

    pub window_open: WindowKind,
    pub preferences: Preferences,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            scene: Default::default(),
            scene_state: SceneState::Editing,
            editor_scene_path: None,
            backup_scene: None,

            selection: Selection::None,
            depth_texture: None,
            editor_camera: EditorCamera2D::default(),
            project: Project::default(),

            viewport_texture_id: None,
            viewport_size: glam::UVec2::ZERO,
            viewport_texture: None,

            window_open: WindowKind::None,
            preferences: Preferences::load().unwrap_or_default(),
        }
    }
}

impl EditorState {
    pub fn play_scene(&mut self) {
        log::info!("Playing scene...");
        self.backup_scene = Some(self.scene.clone());
        self.scene.start_runtime();
        self.scene_state = SceneState::Running;
    }

    pub fn stop_scene(&mut self) {
        if self.scene_state != SceneState::Running {
            return;
        }

        log::info!("Stopping scene...");
        self.scene = self.backup_scene.take().expect("Backup scene not found");
        self.scene_state = SceneState::Editing;
    }

    pub fn save_scene(&mut self, scene_path: Option<PathBuf>) {
        self.stop_scene();

        if let Ok(data) = crile::SceneSerializer::serialize(&self.scene)
            .inspect_err(|err| log::error!("Failed to save scene: {err}"))
        {
            if let Some(path) =
                scene_path.or_else(|| self.project.pick_save_relative("scene.scene"))
            {
                crile::write_file(&self.project.make_absolute(&path), &data);
                self.editor_scene_path = Some(path);

                // Save project when first save scene
                // TODO: option to choose specific main scene
                if self.project.main_scene.is_none() {
                    self.project.main_scene.clone_from(&self.editor_scene_path);
                    self.project.save();
                }
            }
        }
    }

    pub fn load_scene(&mut self, scene_path: Option<PathBuf>) {
        if let Some(path) =
            scene_path.or_else(|| self.project.pick_file_relative("Scene", &["scene"]))
        {
            if let Some(source) = crile::read_file(&self.project.make_absolute(&path)) {
                if let Ok(scene) = crile::SceneSerializer::deserialize(source)
                    .inspect_err(|err| log::error!("Failed to load scene: {err} "))
                {
                    self.stop_scene();
                    self.scene = scene;
                    self.editor_scene_path = Some(path);
                    self.scene.set_viewport(self.viewport_size);
                }
            }
        }
    }
}
