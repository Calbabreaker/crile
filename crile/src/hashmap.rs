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

    fn write_u32(&mut self, i: u32) {
        self.hash = i as u64;
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

/// Hash map that does not hash the key
/// Useful for types that already hashed like TypeId or ids
pub type NoHashHashMap<K, V> =
    hashbrown::HashMap<K, V, std::hash::BuildHasherDefault<NoHashHasher>>;
