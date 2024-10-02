use std::{any::TypeId, ptr::NonNull};

use crate::NoHashHashMap;

use super::TypeInfo;

pub struct Archetype {
    pub(crate) component_arrays: Box<[ComponentArray]>,
    /// Maps a component type id to its index inside self.component
    index_map: NoHashHashMap<TypeId, usize>,
    type_infos: Box<[TypeInfo]>,
    pub(crate) entity_indexs: Box<[usize]>,
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
            .map(|info| ComponentArray::new(info.clone()))
            .collect();

        Self {
            type_infos,
            index_map,
            entity_indexs: Box::new([]),
            count: 0,
            component_arrays,
        }
    }

    /// Moves a component into a component array in this archetype using raw pointers
    /// # Safety
    /// - Component real type must match the type id
    /// - The component must not be dropped and must not be used elsewhere after moving. Use [Self::clone_component] otherwise
    pub unsafe fn move_component(
        &mut self,
        component_index: usize,
        component_ptr: *const u8,
        type_id: TypeId,
    ) {
        let (ptr, type_info) = self.get_component_dst(component_index, type_id);
        std::ptr::copy_nonoverlapping(component_ptr, ptr, type_info.layout.size());
    }

    /// Clones a component into a component array in this archetype using raw pointers
    /// # Safety
    /// - Component real type must match the type id
    pub unsafe fn clone_component(
        &mut self,
        component_index: usize,
        component_ptr: *const u8,
        type_id: TypeId,
    ) {
        let (ptr, type_info) = self.get_component_dst(component_index, type_id);
        (type_info.clone_to)(component_ptr, ptr);
    }

    // # Safety
    // - Component reference must follow Rust borrow rules
    pub(crate) unsafe fn borrow_component<T: 'static>(
        &self,
        component_index: usize,
    ) -> Option<&mut T> {
        assert!(component_index < self.count);

        let array = self.get_array(&TypeId::of::<T>())?;
        Some(&mut *array.get_ptr().cast::<T>().add(component_index))
    }

    pub(crate) fn has_component<T: 'static>(&self) -> bool {
        self.index_map.contains_key(&TypeId::of::<T>())
    }

    /// Returns the entity index inside this archetype
    pub(crate) fn new_entity(&mut self, entity_index: usize) -> usize {
        if self.count >= self.entity_indexs.len() {
            // Grow by double or at least 32
            self.grow(self.entity_indexs.len().max(32))
        }

        let index = self.count;
        self.entity_indexs[index] = entity_index;
        self.count += 1;
        index
    }

    pub(crate) fn remove_entity(&mut self, component_index: usize, should_drop: bool) -> usize {
        assert!(component_index < self.count);

        // Moves the last item to index and decrement length by 1
        let last_index = self.count - 1;
        for array in self.component_arrays.iter_mut() {
            if should_drop {
                unsafe {
                    array.drop_component(component_index);
                }
            }

            array.move_over(last_index, component_index);
        }

        self.entity_indexs[component_index] = self.entity_indexs[last_index];

        self.count -= 1;
        self.entity_indexs[component_index]
    }

    fn grow(&mut self, amount: usize) {
        if amount == 0 {
            return;
        }

        let old_capacity = self.entity_indexs.len();
        let new_capacity = old_capacity + amount;

        let mut new_entites = vec![0; new_capacity].into_boxed_slice();
        new_entites[0..self.count].copy_from_slice(&self.entity_indexs[0..self.count]);
        self.entity_indexs = new_entites;

        for array in self.component_arrays.iter_mut() {
            array.grow(new_capacity);
        }
    }

    pub(crate) fn get_array(&self, id: &TypeId) -> Option<&ComponentArray> {
        let index = self.index_map.get(id)?;
        Some(unsafe { self.component_arrays.get_unchecked(*index) })
    }

    fn get_component_dst(&self, component_index: usize, type_id: TypeId) -> (*mut u8, &TypeInfo) {
        assert!(component_index < self.count);

        let array = self
            .get_array(&type_id)
            .expect("Component is not in the archetype");
        let info = &array.type_info;
        (array.get_component_ptr(component_index), info)
    }

    pub fn type_infos(&self) -> &[TypeInfo] {
        &self.type_infos
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        if self.entity_indexs.len() == 0 {
            return;
        }

        for array in self.component_arrays.iter_mut() {
            array.clean(self.count);
        }
    }
}

impl Clone for Archetype {
    fn clone(&self) -> Self {
        let component_arrays = self
            .component_arrays
            .iter()
            .map(|array| array.clone(self.count))
            .collect();

        Self {
            count: self.count,
            index_map: self.index_map.clone(),
            entity_indexs: self.entity_indexs.clone(),
            component_arrays,
            type_infos: self.type_infos.clone(),
        }
    }
}

pub(crate) struct ComponentArray {
    /// Pointer to the allocated array
    /// We need to store the array as a raw ptr to allow for multiple types (can't use generics)
    /// This also means we need to manage the memory manually
    ptr: Option<NonNull<u8>>,
    type_info: TypeInfo,
    allocated_capacity: usize,
}

impl ComponentArray {
    fn new(type_info: TypeInfo) -> Self {
        Self {
            ptr: None,
            allocated_capacity: 0,
            type_info,
        }
    }

    fn grow(&mut self, new_capacity: usize) {
        assert!(new_capacity > self.allocated_capacity);

        let new_size = self.type_info.layout.size() * new_capacity;
        assert!(
            new_size > 0,
            "Zero sized components are not supported: {}",
            self.type_info
        );

        // We need to allocate new space manually since we don't have access the component generic here
        let old_ptr = self.ptr.map_or(std::ptr::null_mut(), |ptr| ptr.as_ptr());
        let new_ptr = unsafe { std::alloc::realloc(old_ptr, self.get_array_layout(), new_size) };

        if let Some(new_ptr) = NonNull::new(new_ptr) {
            self.allocated_capacity = new_capacity;
            self.ptr = Some(new_ptr);
        } else {
            panic!(
                "Failed to realloc/alloc component array for {}",
                self.type_info
            );
        }
    }

    /// Moves a component by copying over the data from index_a to index_b
    /// Used for swap_remove
    fn move_over(&mut self, index_a: usize, index_b: usize) {
        assert!(index_a <= self.allocated_capacity);
        assert!(index_b <= self.allocated_capacity);

        if index_a != index_b {
            let size = self.type_info.layout.size();
            unsafe {
                let ptr_a = self.get_ptr().add(index_a * size);
                let ptr_b = self.get_ptr().add(index_b * size);
                std::ptr::copy_nonoverlapping(ptr_a, ptr_b, size);
            }
        }
    }

    fn clean(&mut self, count: usize) {
        let ptr = match self.ptr {
            None => return,
            Some(ptr) => ptr.as_ptr(),
        };

        for i in 0..count {
            unsafe {
                self.drop_component(i);
            }
        }

        unsafe {
            std::alloc::dealloc(ptr, self.get_array_layout());
        }
    }

    fn clone(&self, count: usize) -> Self {
        assert!(count <= self.allocated_capacity);

        let mut component_array = Self::new(self.type_info.clone());
        component_array.grow(count);

        for i in 0..count {
            // Call clone on each component
            let src_ptr = self.get_component_ptr(i);
            let dst_ptr = component_array.get_component_ptr(i);
            unsafe {
                (self.type_info.clone_to)(src_ptr, dst_ptr);
            }
        }

        component_array
    }

    pub(crate) unsafe fn drop_component(&mut self, index: usize) {
        unsafe {
            (self.type_info.drop)(self.get_component_ptr(index));
        }
    }

    pub fn get_ptr(&self) -> *mut u8 {
        self.ptr.expect("Component ptr was null").as_ptr()
    }

    fn get_array_layout(&self) -> std::alloc::Layout {
        let size = self.type_info.layout.size();
        let align = self.type_info.layout.align();
        std::alloc::Layout::from_size_align(self.allocated_capacity * size, align).unwrap()
    }

    pub fn get_type_id(&self) -> TypeId {
        self.type_info.id
    }

    // Gets a raw ptr to a component inside this array
    pub fn get_component_ptr(&self, index: usize) -> *mut u8 {
        assert!(index <= self.allocated_capacity);
        let offset = self.type_info.layout.size() * index;
        unsafe { self.get_ptr().add(offset) }
    }
}
