// File system utilities
pub fn get_data_path() -> Option<std::path::PathBuf> {
    let config_path = dirs::data_dir()?.join("Crile");

    if !config_path.is_dir() {
        std::fs::create_dir_all(&config_path)
            .inspect_err(|err| log::error!("Failed to create {config_path:?}: {err}"))
            .ok()?;
    }

    Some(config_path)
}

pub fn write_file(path: &std::path::Path, str: &str) -> bool {
    log::trace!("Saving to {path:?}");
    std::fs::write(path, str)
        .inspect_err(|err| log::error!("Failed to save {path:?}: {err}"))
        .is_ok()
}

pub fn read_file(path: &std::path::Path) -> Option<String> {
    log::trace!("Loading from {path:?}");
    std::fs::read_to_string(path)
        .inspect_err(|err| log::error!("Failed to load {path:?}: {err}"))
        .ok()
}
