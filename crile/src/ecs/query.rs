use super::{Archetype, ComponentTuple, World};

pub struct QueryIter<'a, T: ComponentTuple> {
    world: &'a World,
    next_archetype_index: usize,
    current_archetype_iter: ArchetypeIter<T>,
}

impl<'a, T: ComponentTuple> QueryIter<'a, T> {
    pub(crate) fn new(world: &'a World) -> Self {
        Self {
            next_archetype_index: 0,
            current_archetype_iter: ArchetypeIter::empty(),
            world,
        }
    }

    fn next_archetype(&mut self) -> Option<()> {
        let archetype = self.world.archetypes.get(self.next_archetype_index)?;
        self.current_archetype_iter = ArchetypeIter::new(archetype);
        self.next_archetype_index += 1;
        Some(())
    }

    // Gets the next entity as part of this query
    unsafe fn next_mut(&mut self) -> Option<(usize, T::MutTuple<'a>)> {
        match self.current_archetype_iter.next() {
            Some(tuple) => Some(tuple),
            None => {
                // We went through all the entities in the archetype so get the next one
                self.next_archetype()?;
                self.next_mut()
            }
        }
    }
}

impl<'a, T: ComponentTuple> Iterator for QueryIter<'a, T> {
    type Item = (usize, T::RefTuple<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.next_mut() }.map(|(id, c)| (id, T::mut_to_ref(c)))
    }
}

/// This is the same as QueryIter (uses it internally) but force mutably borrowing World to
/// allow a mutable borrow to the components
pub struct QueryIterMut<'a, T: ComponentTuple> {
    query: QueryIter<'a, T>,
}

impl<'a, T: ComponentTuple> QueryIterMut<'a, T> {
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self {
            query: QueryIter::new(world),
        }
    }
}

impl<'a, T: ComponentTuple> Iterator for QueryIterMut<'a, T> {
    type Item = (usize, T::MutTuple<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.query.next_mut() }
    }
}

struct ArchetypeIter<T: ComponentTuple> {
    component_index: usize,
    count: usize,
    entity_indexs: *const usize,
    array_ptr_array: Option<T::FixedArray<*mut u8>>,
}

impl<T: ComponentTuple> ArchetypeIter<T> {
    fn new(archetype: &Archetype) -> Self {
        match T::get_array_ptrs(archetype) {
            Some(array_ptr_array) => Self {
                component_index: 0,
                count: archetype.count(),
                entity_indexs: archetype.entity_indexs.as_ptr(),
                array_ptr_array: Some(array_ptr_array),
            },
            None => Self::empty(),
        }
    }

    fn empty() -> Self {
        Self {
            component_index: 0,
            count: 0,
            entity_indexs: std::ptr::null(),
            array_ptr_array: None,
        }
    }

    unsafe fn next<'a>(&mut self) -> Option<(usize, T::MutTuple<'a>)> {
        if self.component_index < self.count {
            let component_tuple = T::array_ptr_array_get(
                self.array_ptr_array.as_ref().unwrap_unchecked(),
                self.component_index,
            );
            let index = *self.entity_indexs.add(self.component_index);
            self.component_index += 1;
            Some((index, component_tuple))
        } else {
            None
        }
    }
}
