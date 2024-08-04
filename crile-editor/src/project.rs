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
    pub fn save(&self) -> bool {
        let mut path = self.directory.clone();
        path.push("project.crile");

        let data = toml::to_string(self).unwrap();
        crile::write_file(&path, &data)
    }

    pub fn load(mut path: PathBuf) -> Option<Self> {
        let source = crile::read_file(&path)?;
        let mut project: Self = toml::from_str(&source)
            .inspect_err(|err| log::error!("Failed to load {path:?}: {err}"))
            .ok()?;

        // Get the actual directory itself
        path.pop();
        project.directory = path;

        Some(project)
    }

    pub fn make_absolute(&self, relative_path: &Path) -> PathBuf {
        let mut path = self.directory.clone();
        path.push(relative_path);
        path
    }

    pub fn make_relative(&self, path: &Path) -> Option<PathBuf> {
        path.strip_prefix(&self.directory)
            .inspect_err(|_| log::error!("{path:?} is outside of the project directory"))
            .map(|path| path.to_path_buf())
            .ok()
    }

    pub fn pick_file_relative(&self, filter_name: &str, extensions: &[&str]) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_directory(&self.directory)
            .add_filter(filter_name, extensions)
            .pick_file()
            .and_then(|path| self.make_relative(&path))
    }

    pub fn pick_save_relative(&self, file_name: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_directory(&self.directory)
            .set_file_name(file_name)
            .save_file()
            .and_then(|path| self.make_relative(&path))
    }
}
