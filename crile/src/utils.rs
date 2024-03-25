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
