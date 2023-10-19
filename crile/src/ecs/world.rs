use std::any::TypeId;

use crate::{Archetype, ComponentTuple, QueryIter, QueryIterMut};

pub type EntityID = usize;

/// This ECS works by storing a bunch of archetypes where each archetype stores all the components
/// of entities that have the same components.
/// Then when querying, it will go to each archetype and will only get components in that archetype
/// if that archetype contains the components in the query.
/// This means that it is more optimized for querying components rather than adding/removing components.
#[derive(Default)]
pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    /// Maps to the archetype index inside self.archetypes
    type_ids_index_map: hashbrown::HashMap<Box<[TypeId]>, usize>,
    /// Maps to the archetype index inside self.archetypes
    bundle_id_index_map: hashbrown::HashMap<TypeId, usize>,

    entity_count: EntityID,
}

impl World {
    pub fn spawn<T: ComponentTuple>(&mut self, components: T) -> EntityID {
        let id = self.entity_count;
        self.entity_count += 1;

        let archetype = self.get_archetype::<T>();
        let index = archetype.new_entity(id);
        archetype.put_bundle(index, components);

        id
    }

    pub fn query<T: ComponentTuple>(&self) -> QueryIter<T> {
        QueryIter::new(self)
    }

    pub fn query_mut<T: ComponentTuple>(&mut self) -> QueryIterMut<T> {
        QueryIterMut::new(self)
    }

    fn get_archetype<T: ComponentTuple>(&mut self) -> &mut Archetype {
        // First use the index the map with the bundle id since that's faster to compute with
        let index = *self
            .bundle_id_index_map
            .entry(T::bundle_id())
            .or_insert_with(|| {
                // Then try with the (sorted) type ids
                *self
                    .type_ids_index_map
                    .entry_ref(T::type_ids())
                    .or_insert_with(|| {
                        let archetype = Archetype::new(T::type_infos());
                        self.archetypes.push(archetype);
                        self.archetypes.len() - 1
                    })
            });

        &mut self.archetypes[index]
    }
}
