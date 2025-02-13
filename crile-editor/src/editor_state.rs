use std::path::PathBuf;

use crate::{
    editor_camera::EditorCamera2D, preferences::Preferences, project::Project,
    sections::viewport::SceneViewport,
};

#[derive(PartialEq, Eq, Debug)]
pub enum Selection {
    Entity(usize),
    None,
}

#[derive(PartialEq)]
pub enum PopupKind {
    Preferences(Preferences),
    Stats,
    None,
}

pub struct RuntimeData {
    pub backup_scene: crile::Scene,
    pub scene_runner: crile::SceneRunner,
    pub game_window_id: crile::WindowId,
}

pub enum SceneState {
    Edting,
    Running(RuntimeData),
}

pub struct EditorState {
    pub active_scene: crile::Scene,
    pub scene_state: SceneState,
    pub editor_scene_path: Option<PathBuf>,

    pub selection: Selection,
    pub editor_camera: EditorCamera2D,
    pub editor_view: SceneViewport,

    pub popup_open: PopupKind,
    pub project: Project,
    pub preferences: Preferences,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            active_scene: crile::Scene::with_root(),
            scene_state: SceneState::Edting,
            editor_scene_path: None,

            selection: Selection::None,
            editor_camera: EditorCamera2D::default(),
            project: Project::default(),
            editor_view: SceneViewport::default(),

            popup_open: PopupKind::None,
            preferences: Preferences::load().unwrap_or_default(),
        }
    }
}

impl EditorState {
    pub fn play_scene(&mut self, engine: &mut crile::Engine, event_loop: &crile::ActiveEventLoop) {
        log::trace!("Playing scene...");

        let game_window_id = engine.create_window(
            event_loop,
            crile::WindowConfig {
                title: "Game",
                ..Default::default()
            },
        );

        self.scene_state = SceneState::Running(RuntimeData {
            backup_scene: self.active_scene.clone(),
            scene_runner: unsafe {
                crile::SceneRunner::new(crile::ScriptingEngine::new(
                    &mut self.active_scene,
                    engine,
                    game_window_id,
                ))
            },
            game_window_id,
        });

        if let SceneState::Running(runtime_data) = &mut self.scene_state {
            if let Err(err) = runtime_data.scene_runner.start() {
                log::error!("{err}");
                self.stop_scene(engine);
            }
        }
    }

    pub fn stop_scene(&mut self, engine: &mut crile::Engine) {
        if let SceneState::Running(runtime_data) = &mut self.scene_state {
            log::trace!("Stopping scene...");
            runtime_data.scene_runner.stop();
            engine.delete_window(runtime_data.game_window_id);
            self.active_scene = runtime_data.backup_scene.clone();
            self.scene_state = SceneState::Edting;
        }
    }

    pub fn save_scene(&mut self, scene_path: Option<PathBuf>) {
        if matches!(self.scene_state, SceneState::Running(_)) {
            return;
        }

        if let Ok(data) = crile::SceneSerializer::serialize(&self.active_scene)
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
        if matches!(self.scene_state, SceneState::Running(_)) {
            return;
        }

        if let Some(path) =
            scene_path.or_else(|| self.project.pick_file_relative("Scene", &["scene"]))
        {
            if let Some(source) = crile::read_file(&self.project.make_absolute(&path)) {
                if let Ok(scene) = crile::SceneSerializer::deserialize(source)
                    .inspect_err(|err| log::error!("Failed to load scene: {err} "))
                {
                    self.active_scene = scene;
                    self.editor_scene_path = Some(path);
                }
            }
        }
    }

    pub fn open_project(&mut self, project_file_path: Option<PathBuf>) {
        if matches!(self.scene_state, SceneState::Running(_)) {
            return;
        }

        if let Some(path) = project_file_path.or_else(|| {
            rfd::FileDialog::new()
                .add_filter("Crile project (.crile)", &["crile"])
                .set_file_name("project.crile")
                .set_directory(std::env::current_dir().unwrap_or_default())
                .pick_file()
        }) {
            if let Some(project) = Project::load(path.clone()) {
                self.project = project;

                if let Some(main_scene) = &self.project.main_scene {
                    self.load_scene(Some(main_scene.clone()));
                }

                // Set the last opened project if it isn't the same
                if self.preferences.last_opened_project.as_ref() != Some(&path) {
                    self.preferences.last_opened_project = Some(path);
                    self.preferences.save();
                }
            }
        }
    }
}
