use std::sync::{atomic::AtomicU64, Arc};

/// Wraps the object T in an Arc with a unique id
/// Allows keeping unique objects and useful for hashing and comparing T
pub struct ArcId<T> {
    // Can't use memory location of Arc as the id
    pub object: Arc<T>,
    id: u64,
}

impl<T> ArcId<T> {
    pub fn new(object: T) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self {
            object: Arc::new(object),
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl<T> Clone for ArcId<T> {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
            id: self.id,
        }
    }
}

impl<T> From<T> for ArcId<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> std::hash::Hash for ArcId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> PartialEq for ArcId<T> {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id
    }
}

impl<T> Eq for ArcId<T> {}

impl<T> std::ops::Deref for ArcId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<T> AsRef<T> for ArcId<T> {
    fn as_ref(&self) -> &T {
        self.object.as_ref()
    }
}
