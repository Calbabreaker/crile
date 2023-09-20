use std::{rc::Rc, sync::atomic::AtomicU64};

/// Wraps the object T in an reference counted smart pointer with a unique id
/// Allows keeping unique objects and useful for hashing and comparing T
pub struct RefId<T> {
    pub object: Rc<T>,
    id: u64,
}

impl<T> RefId<T> {
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
}

impl<T> Clone for RefId<T> {
    fn clone(&self) -> Self {
        Self {
            object: self.object.clone(),
            id: self.id,
        }
    }
}

impl<T> From<T> for RefId<T> {
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

pub struct RefHolder<T> {
    refs: Vec<RefId<T>>,
}

impl<T> RefHolder<T> {
    pub fn new() -> RefHolder<T> {
        Self {
            refs: Vec::with_capacity(1024),
        }
    }

    pub fn hold(&mut self, ref_id: RefId<T>) -> &'static T {
        let object = unsafe { std::mem::transmute(ref_id.as_ref()) };
        self.refs.push(ref_id);
        object
    }

    /// # Safety
    /// There must not be any references to T still in use or else T might get dropped and cause
    /// dangling pointers
    pub unsafe fn free(&mut self) {
        self.refs.clear()
    }
}

impl<T> Default for RefHolder<T> {
    fn default() -> Self {
        Self::new()
    }
}
