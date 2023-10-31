use super::{Archetype, ComponentTuple, EntityId, World};

pub struct QueryIter<'a, T: ComponentTuple> {
    world: &'a World,
    next_archetype_index: usize,
    current_iter: ArchetypeIter<T>,
}

impl<'a, T: ComponentTuple> QueryIter<'a, T> {
    pub(crate) fn new(world: &'a World) -> Self {
        Self {
            next_archetype_index: 0,
            current_iter: ArchetypeIter::empty(),
            world,
        }
    }

    fn next_archetype(&mut self) -> Option<()> {
        let archetype = self
            .world
            .archetype_set
            .archetypes
            .get(self.next_archetype_index)?;
        self.current_iter = ArchetypeIter::new(archetype);
        self.next_archetype_index += 1;
        Some(())
    }

    unsafe fn next_mut(&mut self) -> Option<(EntityId, T::MutTuple<'a>)> {
        match self.current_iter.next() {
            Some(tuple) => Some(tuple),
            None => {
                self.next_archetype()?;
                self.next_mut()
            }
        }
    }
}

impl<'a, T: ComponentTuple> Iterator for QueryIter<'a, T> {
    type Item = (EntityId, T::RefTuple<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.next_mut() }.map(|(id, c)| (id, T::mut_to_ref(c)))
    }
}

/// This is the same as QueryIter (it uses it internally) but requires mutably borrowing World in order to ensure borrow rules
pub struct QueryIterMut<'a, T: ComponentTuple> {
    query: QueryIter<'a, T>,
}

impl<'a, T: ComponentTuple> QueryIterMut<'a, T> {
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self {
            query: QueryIter::new(world),
        }
    }
}

impl<'a, T: ComponentTuple> Iterator for QueryIterMut<'a, T> {
    type Item = (EntityId, T::MutTuple<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.query.next_mut() }
    }
}

struct ArchetypeIter<T: ComponentTuple> {
    index: usize,
    count: usize,
    entities: *const EntityId,
    array_ptr_tuple: Option<T::FixedArray<*mut u8>>,
}

impl<T: ComponentTuple> ArchetypeIter<T> {
    fn new(archetype: &Archetype) -> Self {
        match T::get_array_ptrs(archetype) {
            Some(array_ptr_tuple) => Self {
                index: 0,
                count: archetype.count(),
                entities: archetype.entities.as_ptr(),
                array_ptr_tuple: Some(array_ptr_tuple),
            },
            None => Self::empty(),
        }
    }

    fn empty() -> Self {
        Self {
            index: 0,
            count: 0,
            entities: std::ptr::null(),
            array_ptr_tuple: None,
        }
    }

    unsafe fn next<'a>(&mut self) -> Option<(EntityId, T::MutTuple<'a>)> {
        if self.index < self.count {
            let component_tuple = T::array_ptr_array_get(
                self.array_ptr_tuple.as_ref().unwrap_unchecked(),
                self.index,
            );
            let id = *self.entities.add(self.index);
            self.index += 1;
            Some((id, component_tuple))
        } else {
            None
        }
    }
}
