use std::path::PathBuf;

use crate::{
    editor_camera::EditorCamera2D, project::Project, sections::viewport::SceneViewport, Preferences,
};

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

pub struct EditorState {
    pub scene: crile::Scene,
    pub scene_runtime: Option<crile::SceneRuntime>,
    pub editor_scene_path: Option<PathBuf>,

    pub selection: Selection,
    pub editor_camera: EditorCamera2D,
    pub project: Project,
    pub editor_view: SceneViewport,

    pub window_open: WindowKind,
    pub preferences: Preferences,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            scene: Default::default(),
            scene_runtime: None,
            editor_scene_path: None,

            selection: Selection::None,
            editor_camera: EditorCamera2D::default(),
            project: Project::default(),
            editor_view: SceneViewport::default(),

            window_open: WindowKind::None,
            preferences: Preferences::load().unwrap_or_default(),
        }
    }
}

impl EditorState {
    pub fn play_scene(&mut self, engine: &crile::Engine) {
        log::trace!("Playing scene...");
        let mut scene_runtime = crile::SceneRuntime::new(self.scene.clone(), engine);

        if let Err(err) = scene_runtime.start() {
            log::error!("{err}");
            scene_runtime.stop();
        } else {
            self.scene_runtime = Some(scene_runtime);
        }
    }

    pub fn stop_scene(&mut self) {
        if let Some(mut scene_runtime) = self.scene_runtime.take() {
            log::trace!("Stopping scene...");
            scene_runtime.stop();
        }
    }

    pub fn save_scene(&mut self, scene_path: Option<PathBuf>) {
        if self.scene_runtime.is_some() {
            return;
        }

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
        if self.scene_runtime.is_some() {
            return;
        }

        if let Some(path) =
            scene_path.or_else(|| self.project.pick_file_relative("Scene", &["scene"]))
        {
            if let Some(source) = crile::read_file(&self.project.make_absolute(&path)) {
                if let Ok(scene) = crile::SceneSerializer::deserialize(source)
                    .inspect_err(|err| log::error!("Failed to load scene: {err} "))
                {
                    self.scene = scene;
                    self.editor_scene_path = Some(path);
                }
            }
        }
    }

    pub fn open_project(&mut self, project_file_path: Option<PathBuf>) {
        if self.scene_runtime.is_some() {
            return;
        }

        if let Some(path) = project_file_path.or_else(|| {
            rfd::FileDialog::new()
                .add_filter("Crile project", &["crile"])
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

    pub fn active_scene<'a>(&mut self) -> &'a mut crile::Scene {
        let scene = if let Some(scene_runtime) = self.scene_runtime.as_mut() {
            &mut scene_runtime.scene
        } else {
            &mut self.scene
        };
        unsafe { &mut *(scene as *mut crile::Scene) }
    }
}
