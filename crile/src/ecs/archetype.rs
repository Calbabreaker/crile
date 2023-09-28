use std::any::{Any, TypeId};

use super::world::EntityID;
use crate::OrderedMap;

pub struct Archetype {
    /// We need to store the component arrays as a raw ptr to allow for multiple types
    /// This also means we need to manage the memory manually
    component_arrays: Box<[*mut u8]>,
    /// Maps a component id to its index inside self.component
    index_map: OrderedMap<TypeId, usize>,
    entities: Box<[EntityID]>,
    length: usize,
}

impl Archetype {
    pub fn new(component_ids: &[TypeId]) -> Self {
        let index_map = OrderedMap::new(
            component_ids
                .iter()
                .enumerate()
                .map(|(i, id)| (*id, i))
                .collect::<Box<_>>(),
        );

        Self {
            index_map,
            entities: Box::new([]),
            length: 0,
            component_arrays: Box::new([]),
        }
    }

    pub fn set_component<T: 'static>(&mut self, component: T, entity_index: usize) {
        assert!(entity_index < self.length);
        if let Some(index) = self.index_map.get(&component.type_id()) {
            unsafe {
                let component_array_ptr = self.component_arrays.get_unchecked(*index);
                *component_array_ptr.add(entity_index).cast() = component;
            }
        }
    }

    pub fn new_entity(&mut self, entity: EntityID) {
        self.entities[self.length] = entity;
        self.length += 1;
    }

    fn grow(&mut self, amount: usize) {
        let old_cap = self.entities.len();
        let new_cap = old_cap + amount;

        let mut new_entites = vec![0; new_cap].into_boxed_slice();
        new_entites.copy_from_slice(&self.entities[0..self.length]);
        self.entities = new_entites;
    }
}

unsafe fn alloc_new(
    layout: std::alloc::Layout,
    src: *mut u8,
    old_cap: usize,
    new_cap: usize,
    old_len: usize,
) -> *mut u8 {
    // We need to allocate new space manually since we don't have access the generic component type here
    if new_cap == 0 {
        return src;
    }

    let data = std::alloc::alloc(
        std::alloc::Layout::from_size_align(layout.size() * new_cap, layout.align()).unwrap(),
    );

    std::ptr::copy_nonoverlapping(src, data.cast(), old_len);
    if old_cap > 0 {
        std::alloc::dealloc(
            src.cast(),
            std::alloc::Layout::from_size_align(layout.size() * old_cap, layout.align()).unwrap(),
        );
    }
    data.cast()
}

pub struct ComponentTypeInfo {
    id: TypeId,
    layout: std::alloc::Layout,
}

trait ComponentBundle {
    fn get_type_info() -> Box<[ComponentTypeInfo]>;
}

impl<T1> ComponentBundle for (T1) {
    fn get_type_info() -> Box<[ComponentTypeInfo]> {
        (ComponentTypeInfo {
            id: TypeId::of::<T1>(),
        })
    }
}
