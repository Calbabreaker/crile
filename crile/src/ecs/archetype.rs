use std::{any::TypeId, sync::OnceLock};

use super::world::EntityID;
use crate::FixedOrderedMap;

pub struct Archetype {
    component_arrays: Box<[ComponentArray]>,
    /// Maps a component type id to its index inside self.component
    index_map: FixedOrderedMap<TypeId, usize>,
    entities: Box<[EntityID]>,
    count: usize,
}

impl Archetype {
    pub(crate) fn new(type_infos: &[TypeInfo]) -> Self {
        let index_map = FixedOrderedMap::new(
            type_infos
                .iter()
                .enumerate()
                .map(|(i, info)| (info.id, i))
                .collect(),
        );

        let component_arrays = type_infos
            .iter()
            .map(|info| ComponentArray {
                ptr: std::ptr::null_mut(),
                type_info: *info,
            })
            .collect();

        Self {
            index_map,
            entities: Box::new([]),
            count: 0,
            component_arrays,
        }
    }

    pub(crate) fn put_components<T: ComponentTuple>(&mut self, index: usize, components: T) {
        assert!(index < self.count);

        components.take_all(|ptr, id| {
            let component_array_index = self
                .index_map
                .get(&id)
                .expect("component was not in the archetype");

            unsafe {
                let array = self.component_arrays.get_unchecked(*component_array_index);
                let size = array.type_info.layout.size();
                std::ptr::copy_nonoverlapping(ptr, array.ptr.add(index * size), size);
            }
        });
    }

    pub(crate) fn new_entity(&mut self, entity: EntityID) -> usize {
        if self.count >= self.entities.len() {
            // Grow by double or at least 32
            self.grow(self.entities.len().max(32))
        }

        let index = self.count;
        self.entities[index] = entity;
        self.count += 1;
        index
    }

    pub(crate) fn remove(&mut self, index: usize) -> EntityID {
        assert!(index < self.count);

        // Moves the last item to index and decrement length by 1
        let last_index = self.count - 1;
        if index != last_index {
            for array in self.component_arrays.iter() {
                unsafe {
                    let size = array.type_info.layout.size();
                    let to_remove = array.ptr.add(index * size);
                    let last = array.ptr.add(last_index * size);
                    std::ptr::copy_nonoverlapping(last, to_remove, size);
                }
            }

            self.entities[index] = self.entities[last_index];
        }

        self.count -= 1;
        self.entities[index]
    }

    fn grow(&mut self, amount: usize) {
        let old_cap = self.entities.len();
        let new_cap = old_cap + amount;

        if new_cap == old_cap {
            return;
        }

        let mut new_entites = vec![0; new_cap].into_boxed_slice();
        new_entites[0..self.count].copy_from_slice(&self.entities[0..self.count]);
        self.entities = new_entites;

        for array in self.component_arrays.iter_mut() {
            unsafe {
                array.grow(old_cap, new_cap, self.count);
            }
        }
    }

    fn get_array_ptr<T: 'static>(&self) -> Option<*mut T> {
        let index = self.index_map.get(&TypeId::of::<T>())?;
        Some(unsafe { self.component_arrays.get_unchecked(*index).ptr.cast() })
    }

    pub fn get_count(&self) -> usize {
        self.count
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        if self.entities.len() == 0 {
            return;
        }

        for array in self.component_arrays.iter() {
            unsafe {
                std::alloc::dealloc(
                    array.ptr,
                    std::alloc::Layout::from_size_align_unchecked(
                        array.type_info.layout.size() * self.entities.len(),
                        array.type_info.layout.align(),
                    ),
                );
            }
        }
    }
}

struct ComponentArray {
    /// Pointer to the allocated array
    /// We need to store the array as a raw ptr to allow for multiple types (can't use generics)
    /// This also means we need to manage the memory manually
    ptr: *mut u8,
    type_info: TypeInfo,
}

impl ComponentArray {
    unsafe fn grow(&mut self, old_cap: usize, new_cap: usize, count: usize) {
        // We need to allocate new space manually since we don't have access the generic component type here
        let data = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
            self.type_info.layout.size() * new_cap,
            self.type_info.layout.align(),
        ));

        std::ptr::copy_nonoverlapping(self.ptr, data.cast(), count);
        if old_cap > 0 {
            std::alloc::dealloc(
                self.ptr.cast(),
                std::alloc::Layout::from_size_align_unchecked(
                    self.type_info.layout.size() * old_cap,
                    self.type_info.layout.align(),
                ),
            );
        }

        self.ptr = data.cast();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TypeInfo {
    id: TypeId,
    layout: std::alloc::Layout,
}

impl TypeInfo {
    fn of<T: 'static>() -> Self {
        Self {
            id: TypeId::of::<T>(),
            layout: std::alloc::Layout::new::<T>(),
        }
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

/// Represents a tuple of components of any type
/// It is automatically implemented for every tuple
pub trait ComponentTuple {
    fn type_infos() -> &'static [TypeInfo];
    fn type_ids() -> &'static [TypeId];
    fn id() -> TypeId;

    /// Moves every component from the tuple to whatever put_func does and consumes self
    fn take_all(self, put_func: impl Fn(*mut u8, TypeId));

    type ArrayPtrTuple;

    /// Gets a tuple of component arrays from the archetype that matches this component tuple
    /// We use a static sized tuple to prevent unnecessary heap allocation
    fn get_array_ptr_tuple(archetype: &Archetype) -> Option<Self::ArrayPtrTuple>;

    /// Gets the component tuple (self) as a reference to each component from component array pointers
    /// obtained from [Self::get_array_ptr_tuple]
    ///
    /// # Safety
    /// - Index must not be greater than the array size
    /// - Since it returns mutable references to each component, it assumes borrow rules have been it met
    unsafe fn array_ptr_tuple_get<'a>(
        ptr_tuple: &Self::ArrayPtrTuple,
        index: usize,
    ) -> Self::MutTuple<'a>;

    /// The tuple but every component as a ref
    type RefTuple<'a>;
    /// The tuple but every component as a mut ref
    type MutTuple<'a>;

    // Converts a tuple of mutable component reference to non mutable ones
    fn mut_to_ref(mut_tuple: Self::MutTuple<'_>) -> Self::RefTuple<'_>;
}

// TODO: make a macro that impls for (T1, T2, T3, etc.)
impl<T1: 'static> ComponentTuple for (T1,) {
    fn type_infos() -> &'static [TypeInfo] {
        static TYPE_INFOS: OnceLock<[TypeInfo; 1]> = OnceLock::new();
        TYPE_INFOS.get_or_init(|| {
            let mut infos = [TypeInfo::of::<T1>()];
            infos.sort_unstable();
            infos
        })
    }

    fn type_ids() -> &'static [TypeId] {
        static TYPE_IDS: OnceLock<[TypeId; 1]> = OnceLock::new();
        TYPE_IDS.get_or_init(|| {
            let mut infos = [TypeId::of::<T1>()];
            infos.sort_unstable();
            infos
        })
    }

    fn id() -> TypeId {
        TypeId::of::<(T1,)>()
    }

    fn take_all(mut self, put_func: impl Fn(*mut u8, TypeId)) {
        put_func(&mut self.0 as *mut T1 as *mut u8, TypeId::of::<T1>());
    }

    type ArrayPtrTuple = (*mut T1,);

    fn get_array_ptr_tuple(archetype: &Archetype) -> Option<Self::ArrayPtrTuple> {
        Some((archetype.get_array_ptr::<T1>()?,))
    }

    unsafe fn array_ptr_tuple_get<'a>(
        ptr_tuple: &Self::ArrayPtrTuple,
        index: usize,
    ) -> Self::MutTuple<'a> {
        (&mut *ptr_tuple.0.add(index),)
    }

    type RefTuple<'a> = (&'a T1,);
    type MutTuple<'a> = (&'a mut T1,);

    fn mut_to_ref(mut_tuple: Self::MutTuple<'_>) -> Self::RefTuple<'_> {
        (mut_tuple.0,)
    }
}
