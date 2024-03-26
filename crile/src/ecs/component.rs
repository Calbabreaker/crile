use std::any::TypeId;

use super::Archetype;

/// Stores information about a component type to be used inside component arrays
#[derive(Clone, Copy, Debug)]
pub struct TypeInfo {
    pub id: TypeId,
    pub layout: std::alloc::Layout,
    pub drop: unsafe fn(*mut u8),
    #[cfg(debug_assertions)]
    pub typename: &'static str,
}

impl TypeInfo {
    pub fn of<T: 'static>() -> Self {
        Self {
            drop: |ptr| unsafe { ptr.cast::<T>().drop_in_place() },
            id: TypeId::of::<T>(),
            layout: std::alloc::Layout::new::<T>(),
            #[cfg(debug_assertions)]
            typename: std::any::type_name::<T>(),
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

impl std::hash::Hash for TypeInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Represents a tuple of components of any type
/// It is automatically implemented for every tuple type (maximum 8 elements in a tuple)
pub trait ComponentTuple {
    /// Fixed sized array of type T with the length being the number of components inside this tuple
    type FixedArray<T>;

    /// Gets the array of sorted type infos for this tuple
    // TODO: remove heap allocation
    fn type_infos() -> Box<[TypeInfo]>;

    /// Moves every component from the tuple into the archetype
    fn take_all(self, index: usize, archetype: &mut Archetype);

    /// Gets an array of component arrays from the archetype that matches this component tuple
    /// We use a static sized tuple to prevent unnecessary heap allocation
    fn get_array_ptrs(archetype: &Archetype) -> Option<Self::FixedArray<*mut u8>>;

    /// The tuple but every component as a ref
    type RefTuple<'a>;
    /// The tuple but every component as a mut ref
    type MutTuple<'a>;

    /// Gets the component tuple as a reference to each component from the component array pointer
    /// obtained from [Self::get_array_ptrs]
    ///
    /// # Safety
    /// - Index must not be greater than the component array size
    /// - Since it returns mutable references to each component, it assumes borrow rules have been it met
    unsafe fn array_ptr_array_get<'a>(
        array_ptrs: &Self::FixedArray<*mut u8>,
        index: usize,
    ) -> Self::MutTuple<'a>;

    // Converts a tuple of mutable component reference to non mutable ones
    fn mut_to_ref(mut_tuple: Self::MutTuple<'_>) -> Self::RefTuple<'_>;
}

/// Counts the number of identifiers as input
/// Useful for counting macro repetition
/// https://danielkeep.github.io/tlborm/book/blk-counting.html
macro_rules! count_idents {
    ($($idents:ident),*) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            enum Idents { $($idents,)* __CountIdentsLast }
            Idents::__CountIdentsLast as usize
        }
    };
}

/// Macro to automatically impl ComponentTuple for the specified tuple type
macro_rules! tuple_impl {
    ($($type: ident),*) => {
        impl<$($type: 'static),*> ComponentTuple for ($($type,)*) {
            type FixedArray<T> = [T; count_idents!($($type),*)];

            fn type_infos() -> Box<[TypeInfo]> {
                let mut infos = [$(TypeInfo::of::<$type>()),*];
                infos.sort_unstable();
                Box::new(infos)
            }

            #[allow(non_snake_case, unused_variables, unused_mut)]
            fn take_all(self, index: usize, archetype: &mut Archetype) {
                let ($($type,)*) = self;
                $(
                    unsafe {
                        archetype.put_component(index, &$type as *const $type as *const u8, TypeId::of::<$type>());
                        std::mem::forget($type);
                    }
                )*
            }

            #[allow(unused_variables)]
            fn get_array_ptrs(archetype: &Archetype) -> Option<Self::FixedArray<*mut u8>> {
                Some([
                    $(
                        archetype.get_array(&TypeId::of::<$type>())?.ptr
                    ),*
                ])
            }

            type RefTuple<'a> = ($(&'a $type,)*);
            type MutTuple<'a> = ($(&'a mut $type,)*);

            #[allow(non_snake_case, unused_variables, clippy::unused_unit)]
            unsafe fn array_ptr_array_get<'a>(
                ptr_array: &Self::FixedArray<*mut u8>,
                index: usize,
            ) -> Self::MutTuple<'a> {
                let [$($type,)*] = ptr_array;
                (
                    $(
                        &mut *$type.cast::<$type>().add(index),
                    )*
                )
            }

            #[allow(non_snake_case, clippy::unused_unit)]
            fn mut_to_ref(mut_tuple: Self::MutTuple<'_>) -> Self::RefTuple<'_> {
                let ($($type,)*) = mut_tuple;
                ( $($type,)* )
            }
        }
    };
}

macro_rules! recursive_impl {
    ($head: tt) => {
        tuple_impl!();
        tuple_impl!($head);
    };
    ($head: tt, $($tail: tt),*) => {
        tuple_impl!($head, $($tail),*);
        recursive_impl!($($tail),*);
    };
}

// Expands to tuple_impl!(T1), tuple_impl!(T1, T2), tuple_impl!(T1, T2, T3), etc.
recursive_impl!(T1, T2, T3, T4, T5);
