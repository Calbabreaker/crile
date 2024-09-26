use std::{alloc::Layout, any::TypeId, mem::ManuallyDrop};

use crate::Component;

/// Stores information about a component type to be used inside component arrays
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub id: TypeId,
    pub layout: Layout,
    pub drop: unsafe fn(*mut u8),
    /// Calls the type's clone function on T at src cloning to dst
    pub clone_to: unsafe fn(*const u8, *mut u8),
    pub typename: &'static str,
}

impl TypeInfo {
    pub fn of<T: Component>() -> Self {
        Self {
            clone_to: |src, dst| unsafe {
                let mut cloned = ManuallyDrop::new((*src.cast::<T>()).clone());
                let cloned_ptr = &mut *cloned as *mut T as *mut u8;
                std::ptr::copy_nonoverlapping(cloned_ptr, dst, Layout::new::<T>().size());
            },
            drop: |ptr| unsafe {
                ptr.cast::<T>().drop_in_place();
            },
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            typename: std::any::type_name::<T>(),
        }
    }
}

impl std::fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeInfo({})", self.typename)
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TypeInfo {}

impl PartialOrd for TypeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for TypeInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl std::hash::Hash for TypeInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub fn last_type_name<T: 'static>() -> &'static str {
    let name = std::any::type_name::<T>();
    name.split("::").last().unwrap_or(name)
}
