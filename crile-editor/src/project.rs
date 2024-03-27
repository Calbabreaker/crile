use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Project {
    pub name: String,
    pub main_scene: Option<PathBuf>,
    #[serde(skip)]
    pub directory: PathBuf,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Default project".into(),
            directory: std::env::current_dir().expect("Cannot get current directory"),
            main_scene: None,
        }
    }
}

impl Project {
    pub fn save(&self) -> Option<()> {
        let mut path = self.directory.clone();
        path.push("project.crile");

        let data = toml::to_string(self).ok()?;
        crile::write_file(&path, &data)
    }

    pub fn load(mut path: PathBuf) -> Option<Self> {
        let source = crile::read_file(&path)?;
        let mut project: Self = toml::from_str(&source)
            .inspect_err(|err| log::error!("Failed to load {path:?}: {err}"))
            .ok()?;

        path.pop();
        project.directory = path;
        if let Some(main_scene) = &project.main_scene {
            if main_scene.is_absolute() {
                project.main_scene = project.make_relative(main_scene);
                project.save();
            }
        }

        Some(project)
    }

    pub fn make_absolute(&self, relative_path: &Path) -> PathBuf {
        let mut path = self.directory.clone();
        path.push(relative_path);
        path
    }

    pub fn make_relative(&self, path: &Path) -> Option<PathBuf> {
        pathdiff::diff_paths(path, &self.directory).and_then(|path| {
            if path.starts_with("../") {
                log::error!("Tried to get file outside of project directory");
                None
            } else {
                Some(path)
            }
        })
    }

    pub fn pick_file_relative(&self, filter_name: &str, extensions: &[&str]) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_directory(&self.directory)
            .add_filter(filter_name, extensions)
            .pick_file()
            .and_then(|path| pathdiff::diff_paths(path, &self.directory))
    }

    pub fn pick_save_relative(&self, file_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_directory(&self.directory)
            .set_file_name(file_name)
            .save_file()
            .and_then(|path| pathdiff::diff_paths(path, &self.directory))
    }
}
