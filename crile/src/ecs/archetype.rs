use std::{any::TypeId, ptr::NonNull};

use crate::{Component, NoHashHashMap};

use super::TypeInfo;

#[derive(Clone)]
pub struct Archetype {
    pub(crate) component_arrays: Box<[ComponentArray]>,
    /// Maps a component type id to its index inside [Self::component_arrays]
    index_map: NoHashHashMap<TypeId, usize>,
    type_infos: Box<[TypeInfo]>,
    pub(crate) entity_indexs: Vec<usize>,
}

impl Archetype {
    const START_CAP: usize = 32;

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
            .map(|info| ComponentArray::with_capacity(info.clone(), Self::START_CAP))
            .collect();

        Self {
            type_infos,
            index_map,
            entity_indexs: Vec::with_capacity(Self::START_CAP),
            component_arrays,
        }
    }

    /// Push a component into a component array in this archetype using raw pointers.
    /// [Self::new_entity] must be called before this.
    ///
    /// # Safety
    /// - Component real type must match the type id.
    /// - The component must not be dropped and must not be used elsewhere after calling this function. Use [Self::push_component_cloned] otherwise.
    pub unsafe fn push_component(&mut self, component_ptr: *const u8, type_id: TypeId) {
        let array = self.get_array_mut(&type_id).expect("id does not exist");
        let array_ptr = array.alloc_push_space();
        std::ptr::copy_nonoverlapping(component_ptr, array_ptr, array.type_info.layout.size());
    }

    /// Push a component into a component array in this archetype using raw pointers but cloned.
    /// [Self::new_entity] must be called before this.
    ///
    /// # Safety
    /// - Component real type must match the type id.
    pub unsafe fn push_component_cloned(&mut self, component_ptr: *const u8, type_id: TypeId) {
        let array = self.get_array_mut(&type_id).expect("id does not exist");
        let array_ptr = array.alloc_push_space();
        (array.type_info.clone_to)(component_ptr, array_ptr);
    }

    /// # Safety
    /// - Component reference must follow Rust borrow rules
    pub(crate) unsafe fn borrow_component<T: Component>(
        &self,
        component_index: usize,
    ) -> Option<&mut T> {
        let array = self.get_array(&TypeId::of::<T>())?;
        Some(&mut *array.get_component_ptr(component_index).cast::<T>())
    }

    pub(crate) fn has_component<T: 'static>(&self) -> bool {
        self.index_map.contains_key(&TypeId::of::<T>())
    }

    /// Returns the entity index inside this archetype
    pub(crate) fn new_entity(&mut self, entity_index: usize) -> usize {
        self.entity_indexs.push(entity_index);
        self.entity_indexs.len() - 1
    }

    /// Returns the entity index of the component that was swapped
    pub(crate) fn remove_entity(&mut self, component_index: usize, should_drop: bool) -> usize {
        assert!(component_index < self.count());

        // Moves the last item to index and decrement length by 1
        for array in self.component_arrays.iter_mut() {
            array.swap_remove(component_index, should_drop);
        }

        let moved_index = *self.entity_indexs.last().unwrap();
        self.entity_indexs.swap_remove(component_index);
        moved_index
    }

    pub(crate) fn get_array(&self, id: &TypeId) -> Option<&ComponentArray> {
        let index = self.index_map.get(id)?;
        self.component_arrays.get(*index)
    }

    pub(crate) fn get_array_mut(&mut self, id: &TypeId) -> Option<&mut ComponentArray> {
        let index = self.index_map.get(id)?;
        let array = self.component_arrays.get_mut(*index)?;
        assert_eq!(array.count + 1, self.entity_indexs.len()); // Check new_entity was called
        Some(array)
    }

    pub fn type_infos(&self) -> &[TypeInfo] {
        &self.type_infos
    }

    pub fn count(&self) -> usize {
        self.entity_indexs.len()
    }
}

pub(crate) struct ComponentArray {
    /// Pointer to the allocated array
    /// We need to store the array as a raw ptr to allow for multiple types (can't use generics)
    /// This also means we need to manage the memory manually
    ptr: NonNull<u8>,
    type_info: TypeInfo,
    capacity: usize,
    count: usize,
}

impl ComponentArray {
    fn new(type_info: TypeInfo) -> Self {
        Self {
            ptr: NonNull::dangling(),
            capacity: 0,
            type_info,
            count: 0,
        }
    }

    fn with_capacity(type_info: TypeInfo, capacity: usize) -> Self {
        let mut array = Self::new(type_info);
        array.grow(capacity);
        array
    }

    fn grow(&mut self, new_capacity: usize) {
        assert!(new_capacity > self.capacity);

        let new_layout = self.get_array_layout(new_capacity);
        assert!(
            new_layout.size() > 0,
            "Zero sized components are not supported: {}",
            self.type_info
        );
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation overflowed"
        );

        let new_ptr = if self.capacity == 0 {
            unsafe { std::alloc::alloc(new_layout) }
        } else {
            let old_ptr = self.ptr.as_ptr();
            let old_layout = self.get_array_layout(self.capacity);
            unsafe { std::alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        if let Some(new_ptr) = NonNull::new(new_ptr) {
            self.ptr = new_ptr;
            self.capacity = new_capacity;
        } else {
            std::alloc::handle_alloc_error(new_layout);
        }
    }

    fn swap_remove(&mut self, index: usize, drop: bool) {
        if drop {
            unsafe {
                self.drop_component(index);
            }
        }

        let last_index = self.count - 1;
        self.count -= 1;
        if index == last_index {
            return;
        }

        let size = self.type_info.layout.size();
        unsafe {
            let ptr_src = self.get_ptr().add(index * size);
            let ptr_dst = self.get_ptr().add(last_index * size);
            std::ptr::copy_nonoverlapping(ptr_src, ptr_dst, size);
        }
    }

    pub(crate) unsafe fn drop_component(&mut self, index: usize) {
        (self.type_info.drop)(self.get_component_ptr(index));
    }

    pub fn get_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    fn get_array_layout(&self, capacity: usize) -> std::alloc::Layout {
        let size = self.type_info.layout.size();
        let align = self.type_info.layout.align();
        std::alloc::Layout::from_size_align(capacity * size, align).unwrap()
    }

    pub fn get_type_id(&self) -> TypeId {
        self.type_info.id
    }

    // Gets a raw ptr to a component inside this array
    pub fn get_component_ptr(&self, index: usize) -> *mut u8 {
        assert!(index <= self.count);
        let offset = self.type_info.layout.size() * index;
        unsafe { self.get_ptr().add(offset) }
    }

    fn alloc_push_space(&mut self) -> *mut u8 {
        self.count += 1;
        if self.count > self.capacity {
            self.grow(self.capacity * 2);
        }

        self.get_component_ptr(self.count - 1)
    }
}

impl Clone for ComponentArray {
    fn clone(&self) -> Self {
        let mut component_array = Self::with_capacity(self.type_info.clone(), self.capacity);
        component_array.count = self.count;

        for i in 0..self.count {
            // Call clone on each component
            let src_ptr = self.get_component_ptr(i);
            let dst_ptr = component_array.get_component_ptr(i);
            unsafe {
                (self.type_info.clone_to)(src_ptr, dst_ptr);
            }
        }

        component_array
    }
}

impl Drop for ComponentArray {
    fn drop(&mut self) {
        if self.capacity == 0 {
            return;
        }

        for i in 0..self.count {
            unsafe {
                self.drop_component(i);
            }
        }

        unsafe {
            std::alloc::dealloc(self.get_ptr(), self.get_array_layout(self.capacity));
        }
    }
}
