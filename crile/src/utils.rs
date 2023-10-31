use std::{any::Any, rc::Rc, sync::atomic::AtomicU64};

/// Wraps the object T in an reference counted smart pointer with a unique id
/// Allows keeping unique objects and useful for hashing and comparing T
#[derive(Debug)]
pub struct RefId<T: ?Sized> {
    pub object: Rc<T>,
    id: u64,
}

impl<T: 'static> RefId<T> {
    pub fn new(object: T) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self {
            object: Rc::new(object),
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn as_any(self) -> RefId<dyn Any> {
        RefId {
            object: self.object,
            id: self.id,
        }
    }
}

impl<T> Clone for RefId<T> {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
            id: self.id,
        }
    }
}

impl<T: 'static> From<T> for RefId<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> std::hash::Hash for RefId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> PartialEq for RefId<T> {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id
    }
}

impl<T> Eq for RefId<T> {}

impl<T> std::ops::Deref for RefId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<T> AsRef<T> for RefId<T> {
    fn as_ref(&self) -> &T {
        &self.object
    }
}

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
