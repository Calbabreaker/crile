use std::any::TypeId;

use crate::{Archetype, ComponentTuple, QueryIter, QueryIterMut};

pub type EntityID = usize;

#[derive(Clone, Copy, Default, Debug)]
struct EntityLocation {
    archetype_index: usize,
    entity_index: usize,
    active: bool,
}

/// Contains all the data for an ECS instance.
/// This ECS works by storing a bunch of archetypes where each archetype stores all the components
/// of entities that have the same set of components.
/// Then when querying, it will go to each archetype and will only get components in that archetype
/// if that archetype contains the components in the query.
/// This means that it is more optimized for querying components rather than adding/removing components
/// and having few different archetypes.
#[derive(Default)]
pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    /// Maps to the archetype index inside self.archetypes
    type_ids_index_map: hashbrown::HashMap<Box<[TypeId]>, usize>,
    /// Maps to the archetype index inside self.archetypes
    tuple_id_index_map: hashbrown::HashMap<TypeId, usize>,

    free_entity_ids: Vec<EntityID>,
    entity_count: EntityID,
    entity_locations: Vec<EntityLocation>,
}

impl World {
    pub fn spawn<T: ComponentTuple>(&mut self, components: T) -> EntityID {
        let id = self.alloc_entity();
        let archetype_index = self.get_archetype_index::<T>();
        let archetype = &mut self.archetypes[archetype_index];
        let entity_index = archetype.new_entity(id);
        archetype.put_components(entity_index, components);

        self.entity_locations[id] = EntityLocation {
            archetype_index,
            entity_index,
            active: true,
        };

        id
    }

    pub fn despawn(&mut self, id: EntityID) {
        let location = self.entity_locations[id];
        let archetype = &mut self.archetypes[location.archetype_index];
        let moved_id = archetype.remove(location.entity_index);
        self.entity_locations[moved_id].entity_index = location.entity_index;

        self.free_entity_ids.push(id);
        self.entity_locations[id].active = false;
    }

    pub fn query<T: ComponentTuple>(&self) -> QueryIter<T> {
        QueryIter::new(self)
    }

    pub fn query_mut<T: ComponentTuple>(&mut self) -> QueryIterMut<T> {
        QueryIterMut::new(self)
    }

    fn get_archetype_index<T: ComponentTuple>(&mut self) -> usize {
        // First use the index the map with the comopnent tuple id since that's faster to compute with
        *self.tuple_id_index_map.entry(T::id()).or_insert_with(|| {
            // Then try with the (sorted) type ids
            *self
                .type_ids_index_map
                .entry_ref(T::type_ids())
                .or_insert_with(|| {
                    let archetype = Archetype::new(T::type_infos());
                    self.archetypes.push(archetype);
                    self.archetypes.len() - 1
                })
        })
    }

    fn alloc_entity(&mut self) -> EntityID {
        if let Some(id) = self.free_entity_ids.pop() {
            id
        } else {
            let id = self.entity_count;
            self.entity_count += 1;
            self.entity_locations.push(EntityLocation::default());
            id
        }
    }
}
