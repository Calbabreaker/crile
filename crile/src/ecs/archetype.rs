use std::any::TypeId;

use crate::{EntityId, FixedOrderedMap, TypeInfo};

pub struct Archetype {
    component_arrays: Box<[ComponentArray]>,
    /// Maps a component type id to its index inside self.component
    index_map: FixedOrderedMap<TypeId, usize>,
    entities: Box<[EntityId]>,
    count: usize,
}

impl Archetype {
    pub(crate) fn new(type_infos: &[TypeInfo]) -> Self {
        for w in type_infos.windows(2) {
            match w[0].cmp(&w[1]) {
                std::cmp::Ordering::Less => (),
                std::cmp::Ordering::Equal => panic!("created a entity with duplicate components"),
                std::cmp::Ordering::Greater => panic!("type infos not sorted"),
            }
        }

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

    /// # Safety
    /// - Index must be less than count
    /// - Component real pointer type must match id
    pub(crate) unsafe fn put_component(
        &mut self,
        index: usize,
        component_ptr: *const u8,
        id: TypeId,
    ) {
        debug_assert!(index < self.count);

        let array = self
            .get_array(&id)
            .expect("component is not in the archetype");
        let size = array.type_info.layout.size();
        std::ptr::copy_nonoverlapping(component_ptr, array.ptr.add(index * size), size);
    }

    pub(crate) fn borrow_component<'a, T: 'static>(&self, index: usize) -> Option<&'a mut T> {
        assert!(index < self.count);

        let array = self.get_array(&TypeId::of::<T>())?;
        Some(unsafe {
            &mut *array.ptr.cast::<T>().add(index) //
        })
    }

    /// Returns the entity index inside this archetype
    pub(crate) fn new_entity(&mut self, entity: EntityId) -> usize {
        if self.count >= self.entities.len() {
            // Grow by double or at least 32
            self.grow(self.entities.len().max(32))
        }

        let index = self.count;
        self.entities[index] = entity;
        self.count += 1;
        index
    }

    pub(crate) fn remove_entity(&mut self, index: usize) -> EntityId {
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

    pub(crate) fn get_array(&self, id: &TypeId) -> Option<&ComponentArray> {
        let index = self.index_map.get(id)?;
        Some(unsafe { self.component_arrays.get_unchecked(*index) })
    }

    pub(crate) fn get_all_arrays(&self) -> &[ComponentArray] {
        &self.component_arrays
    }

    pub(crate) fn get_type_infos(&self) -> impl Iterator<Item = TypeInfo> + '_ {
        self.component_arrays.iter().map(|array| array.type_info)
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

pub(crate) struct ComponentArray {
    /// Pointer to the allocated array
    /// We need to store the array as a raw ptr to allow for multiple types (can't use generics)
    /// This also means we need to manage the memory manually
    pub ptr: *mut u8,
    pub type_info: TypeInfo,
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
