use std::{any::Any, rc::Rc, sync::atomic::AtomicU64};

/// Wraps the object T in an reference counted smart pointer with a unique id
/// Allows keeping unique objects and useful for hashing and comparing T
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

pub struct RefIdHolder {
    refs: Vec<RefId<dyn Any>>,
}

impl RefIdHolder {
    pub fn new() -> RefIdHolder {
        Self {
            refs: Vec::with_capacity(1024),
        }
    }

    /// Holds the ref_id and returns a static reference to the inner value
    /// This ensures the value will live at least until self.free() is called
    pub fn hold<T>(&mut self, ref_id: RefId<T>) -> &'static T {
        let object = unsafe { std::mem::transmute(ref_id.as_ref()) };
        self.refs.push(ref_id.as_any());
        object
    }

    /// # Safety
    /// There must not be any references to any ref ids still in use or else it might get dropped and cause
    /// dangling pointers
    pub unsafe fn free(&mut self) {
        self.refs.clear()
    }
}

impl Default for RefIdHolder {
    fn default() -> Self {
        Self::new()
    }
}

/// Fixed sized map where element is ordered by K and is indexed by binary searching
pub struct FixedOrderedMap<K, V> {
    data: Box<[(K, V)]>,
}

impl<K: Ord + Copy, V> FixedOrderedMap<K, V> {
    pub fn new(mut data: Box<[(K, V)]>) -> Self {
        data.sort_unstable_by_key(|(id, _)| *id);
        Self { data }
    }

    pub fn get(&self, id: &K) -> Option<&V> {
        let index = self.data.binary_search_by_key(id, |(id, _)| *id).ok()?;
        Some(&self.data[index].1)
    }
}
