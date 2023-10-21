use crate::Archetype;
use crate::{ComponentTuple, World};

pub struct QueryIter<'w, T: ComponentTuple> {
    world: &'w World,
    next_archetype_index: usize,
    current_iter: ArchetypeIter<T>,
    _phantom: std::marker::PhantomData<T>,
}

impl<'w, T: ComponentTuple> QueryIter<'w, T> {
    pub(crate) fn new(world: &'w World) -> Self {
        Self {
            next_archetype_index: 0,
            current_iter: ArchetypeIter::empty(),
            world,
            _phantom: std::marker::PhantomData,
        }
    }

    fn next_archetype(&mut self) -> Option<()> {
        let archetype = self.world.archetypes.get(self.next_archetype_index)?;
        self.current_iter = ArchetypeIter::new(archetype);
        self.next_archetype_index += 1;
        Some(())
    }

    unsafe fn next_mut(&mut self) -> Option<T::MutTuple<'w>> {
        match self.current_iter.next() {
            Some(tuple) => Some(tuple),
            None => {
                self.next_archetype()?;
                self.next_mut()
            }
        }
    }
}

impl<'w, T: ComponentTuple> Iterator for QueryIter<'w, T> {
    type Item = T::RefTuple<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { Some(T::mut_to_ref(self.next_mut()?)) }
    }
}

/// This is the same as QueryIter (it uses it internally) but requires mutably borrowing World
/// in order to ensure borrow rules
pub struct QueryIterMut<'w, T: ComponentTuple> {
    query: QueryIter<'w, T>,
}

impl<'w, T: ComponentTuple> QueryIterMut<'w, T> {
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(crate) fn new(world: &'w mut World) -> Self {
        Self {
            query: QueryIter::new(world),
        }
    }
}

impl<'w, T: ComponentTuple> Iterator for QueryIterMut<'w, T> {
    type Item = T::MutTuple<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.query.next_mut() }
    }
}

struct ArchetypeIter<T: ComponentTuple> {
    index: usize,
    count: usize,
    array_ptr_tuple: Option<T::BytePtrArray>,
}

impl<T: ComponentTuple> ArchetypeIter<T> {
    fn new(archetype: &Archetype) -> Self {
        match T::get_array_ptrs(archetype) {
            Some(array_ptr_uple) => Self {
                index: 0,
                count: archetype.get_count(),
                array_ptr_tuple: Some(array_ptr_uple),
            },
            None => Self::empty(),
        }
    }

    fn empty() -> Self {
        Self {
            index: 0,
            count: 0,
            array_ptr_tuple: None,
        }
    }

    unsafe fn next<'a>(&mut self) -> Option<T::MutTuple<'a>> {
        if self.index < self.count {
            let component_tuple = T::array_ptr_array_get(
                self.array_ptr_tuple.as_ref().unwrap_unchecked(),
                self.index,
            );
            self.index += 1;
            Some(component_tuple)
        } else {
            None
        }
    }
}
