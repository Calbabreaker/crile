use std::any::TypeId;

use crate::NoHashHashMap;

use super::{EntityId, TypeInfo};

pub struct Archetype {
    pub(crate) component_arrays: Box<[ComponentArray]>,
    /// Maps a component type id to its index inside self.component
    index_map: NoHashHashMap<TypeId, usize>,
    type_infos: Box<[TypeInfo]>,
    pub(crate) entities: Box<[EntityId]>,
    count: usize,
}

impl Archetype {
    pub(crate) fn new(type_infos: Box<[TypeInfo]>) -> Self {
        for w in type_infos.windows(2) {
            match w[0].cmp(&w[1]) {
                std::cmp::Ordering::Less => (),
                std::cmp::Ordering::Equal => {
                    panic!("created an entity with duplicate components: {:?}", w[0])
                }
                std::cmp::Ordering::Greater => panic!("type infos not sorted"),
            }
        }

        let index_map = type_infos
            .iter()
            .enumerate()
            .map(|(i, info)| (info.id, i))
            .collect();

        let component_arrays = type_infos
            .iter()
            .map(|info| ComponentArray {
                ptr: std::ptr::null_mut(),
                type_info: info.clone(),
            })
            .collect();

        Self {
            type_infos,
            index_map,
            entities: Box::new([]),
            count: 0,
            component_arrays,
        }
    }

    /// # Safety
    /// - Component pointer's real type must match the type id
    pub unsafe fn put_component(&mut self, index: usize, component_ptr: *const u8, id: TypeId) {
        assert!(index < self.count);

        let array = self
            .get_array(&id)
            .expect("component is not in the archetype");
        let size = array.type_info.layout.size();
        std::ptr::copy_nonoverlapping(component_ptr, array.ptr.add(index * size), size);
    }

    // # Safety
    // - Component reference must follow Rust borrow rules
    pub(crate) unsafe fn borrow_component<T: 'static>(&self, index: usize) -> Option<&mut T> {
        assert!(index < self.count);

        let array = self.get_array(&TypeId::of::<T>())?;
        Some(&mut *array.ptr.cast::<T>().add(index))
    }

    pub(crate) fn has_component<T: 'static>(&self) -> bool {
        self.index_map.contains_key(&TypeId::of::<T>())
    }

    /// Returns the entity index inside this archetype
    pub(crate) fn new_entity(&mut self, id: EntityId) -> usize {
        if self.count >= self.entities.len() {
            // Grow by double or at least 32
            self.grow(self.entities.len().max(32))
        }

        let index = self.count;
        self.entities[index] = id;
        self.count += 1;
        index
    }

    pub(crate) fn remove_entity(&mut self, index: usize, should_drop: bool) -> EntityId {
        assert!(index < self.count);

        // Moves the last item to index and decrement length by 1
        let last_index = self.count - 1;
        for array in self.component_arrays.iter_mut() {
            unsafe {
                if should_drop {
                    array.drop_component(index);
                }

                array.swap_remove(index, last_index);
            }
        }

        self.entities[index] = self.entities[last_index];

        self.count -= 1;
        self.entities[index]
    }

    fn grow(&mut self, amount: usize) {
        let old_capacity = self.entities.len();
        let new_capacity = old_capacity + amount;

        if new_capacity == old_capacity {
            return;
        }

        let mut new_entites = vec![0; new_capacity].into_boxed_slice();
        new_entites[0..self.count].copy_from_slice(&self.entities[0..self.count]);
        self.entities = new_entites;

        for array in self.component_arrays.iter_mut() {
            unsafe {
                array.grow(old_capacity, new_capacity, self.count);
            }
        }
    }

    pub(crate) fn get_array(&self, id: &TypeId) -> Option<&ComponentArray> {
        let index = self.index_map.get(id)?;
        Some(unsafe { self.component_arrays.get_unchecked(*index) })
    }

    pub(crate) fn type_infos(&self) -> &[TypeInfo] {
        &self.type_infos
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        if self.entities.len() == 0 {
            return;
        }

        for array in self.component_arrays.iter_mut() {
            unsafe {
                array.clean(self.count, self.entities.len());
            }
        }
    }
}

impl Clone for Archetype {
    fn clone(&self) -> Self {
        let component_arrays = self
            .component_arrays
            .iter()
            .map(|array| unsafe { array.clone(self.count, self.entities.len()) })
            .collect();

        Self {
            count: self.count,
            index_map: self.index_map.clone(),
            entities: self.entities.clone(),
            component_arrays,
            type_infos: self.type_infos.clone(),
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
    // We don't store the capcity or count inside the ComponentArray to make sure every
    // ComponentArray is the same size
    unsafe fn grow(&mut self, old_capacity: usize, new_capacity: usize, count: usize) {
        let size = self.type_info.layout.size();

        // We need to allocate new space manually since we don't have access the component type here
        let new_ptr = std::alloc::realloc(
            self.ptr,
            std::alloc::Layout::from_size_align_unchecked(
                size * old_capacity,
                self.type_info.layout.align(),
            ),
            size * new_capacity,
        );

        if count > 0 {
            std::ptr::copy_nonoverlapping(self.ptr, new_ptr, count * size);
        }

        self.ptr = new_ptr;
    }

    // Removes a entity component by swapping with the last element for a O(1) operation
    unsafe fn swap_remove(&mut self, index: usize, last_index: usize) {
        if index != last_index {
            let size = self.type_info.layout.size();
            let to_remove = self.ptr.add(index * size);
            let last = self.ptr.add(last_index * size);

            std::ptr::copy_nonoverlapping(last, to_remove, size);
        }
    }

    unsafe fn clean(&mut self, count: usize, capacity: usize) {
        // Call the destructor on the components
        for i in 0..count {
            self.drop_component(i);
        }

        // Deallocate the component array data
        std::alloc::dealloc(
            self.ptr,
            std::alloc::Layout::from_size_align_unchecked(
                self.type_info.layout.size() * capacity,
                self.type_info.layout.align(),
            ),
        );
    }

    unsafe fn clone(&self, count: usize, capacity: usize) -> Self {
        let size = self.type_info.layout.size();
        let new_ptr = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
            size * capacity,
            self.type_info.layout.align(),
        ));

        for i in 0..count {
            // Call clone on each component
            let offset = size * i;
            (self.type_info.clone_to)(self.ptr.add(offset), new_ptr.add(offset));
        }

        Self {
            ptr: new_ptr,
            type_info: self.type_info.clone(),
        }
    }

    pub(crate) unsafe fn drop_component(&mut self, index: usize) {
        let offset = self.type_info.layout.size() * index;
        (self.type_info.drop)(self.ptr.add(offset));
    }
}
