pub fn index_mut_twice<T>(array: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
    assert!(a != b);
    assert!(a < array.len());
    assert!(b < array.len());
    let ptr = array.as_mut_ptr();
    unsafe { (&mut *ptr.add(a), &mut *ptr.add(b)) }
}

/// A hasher that directly forwards the value and does not hash
#[derive(Default)]
pub struct NoHashHasher {
    hash: u64,
}

impl std::hash::Hasher for NoHashHasher {
    fn write(&mut self, _: &[u8]) {
        panic!("tried to use NoHashHasher with an unsupported type");
    }

    fn write_u64(&mut self, i: u64) {
        self.hash = i;
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

/// Hash map that does not hash the key
/// Useful for types that already hashed like TypeId
pub type NoHashHashMap<K, V> =
    hashbrown::HashMap<K, V, std::hash::BuildHasherDefault<NoHashHasher>>;

// File system utilities
pub fn get_data_path() -> Option<std::path::PathBuf> {
    let mut config_path = dirs::data_dir()?;
    config_path.push("Crile");
    if !config_path.is_dir() {
        std::fs::create_dir_all(&config_path)
            .inspect_err(|err| log::error!("Failed to create {config_path:?}: {err}"))
            .ok()?;
    }

    Some(config_path)
}

pub fn write_file(path: &std::path::Path, str: &str) -> Option<()> {
    log::info!("Saving to {path:?}");
    std::fs::write(path, str)
        .inspect_err(|err| log::error!("Failed to save {path:?}: {err}"))
        .ok()
}

pub fn read_file(path: &std::path::Path) -> Option<String> {
    log::info!("Reading from {path:?}");
    std::fs::read_to_string(path)
        .inspect_err(|err| log::error!("Failed to load {path:?}: {err}"))
        .ok()
}
