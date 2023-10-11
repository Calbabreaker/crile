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
    pub fn new(type_infos: &[TypeInfo]) -> Self {
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

    pub fn put_bundle<T: ComponentBundle>(&mut self, entity_index: usize, components: T) {
        assert!(entity_index < self.count);
        components.take(|ptr, info| {
            if let Some(index) = self.index_map.get(&info.id) {
                unsafe {
                    let array = self.component_arrays.get_unchecked(*index);
                    std::ptr::copy_nonoverlapping(
                        ptr,
                        array.ptr.add(entity_index),
                        info.layout.size(),
                    );
                }
            } else {
                panic!("components does not match the archetype components");
            }
        });
    }

    pub fn new_entity(&mut self, entity: EntityID) -> usize {
        if self.count >= self.entities.len() {
            // Grow by double or at least 32
            self.grow(self.entities.len().max(32))
        }

        let index = self.count;
        self.entities[self.count] = entity;
        self.count += 1;
        index
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
}

struct ComponentArray {
    /// We need to store the array as a raw ptr to allow for multiple types (can't use generics)
    /// This also means we need to manage the memory manually
    ptr: *mut u8,
    type_info: TypeInfo,
}

impl ComponentArray {
    unsafe fn grow(&mut self, old_cap: usize, new_cap: usize, count: usize) {
        // We need to allocate new space manually since we don't have access the generic component type here
        let data = std::alloc::alloc(
            std::alloc::Layout::from_size_align(
                self.type_info.layout.size() * new_cap,
                self.type_info.layout.align(),
            )
            .unwrap(),
        );

        std::ptr::copy_nonoverlapping(self.ptr, data.cast(), count);
        if old_cap > 0 {
            std::alloc::dealloc(
                self.ptr.cast(),
                std::alloc::Layout::from_size_align(
                    self.type_info.layout.size() * old_cap,
                    self.type_info.layout.align(),
                )
                .unwrap(),
            );
        }

        self.ptr = data.cast();
    }
}

impl Drop for ComponentArray {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.ptr, self.type_info.layout);
        }
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

pub trait ComponentBundle {
    fn type_infos() -> &'static [TypeInfo];
    fn type_ids() -> &'static [TypeId];
    fn bundle_id() -> TypeId;

    /// Moves every component in the bundle to whatever put_func does and consuming self
    fn take(self, put_func: impl Fn(*mut u8, TypeInfo));

    type PtrTuple;

    fn get_component_array_ptrs(archetype: &Archetype) -> Self::PtrTuple;
    unsafe fn ptrs_to_bundle(component_array_ptrs: Self::PtrTuple, index: usize) -> Self;
}

impl<T1: 'static> ComponentBundle for (T1,) {
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

    fn bundle_id() -> TypeId {
        TypeId::of::<(T1,)>()
    }

    fn take(mut self, put_func: impl Fn(*mut u8, TypeInfo)) {
        put_func(&mut self.0 as *mut T1 as *mut u8, TypeInfo::of::<T1>());
    }

    type PtrTuple = (*mut u8,);

    fn get_component_array_ptrs(archetype: &Archetype) -> Self::PtrTuple {
        (archetype.component_arrays[*archetype.index_map.get(&TypeId::of::<T1>()).unwrap()].ptr,)
    }

    unsafe fn ptrs_to_bundle(component_array_ptrs: Self::PtrTuple, index: usize) -> Self {
        (*component_array_ptrs.0.cast::<T1>(),)
    }
}
