use std::any::TypeId;

use super::archetype::{Archetype, ComponentBundle};

pub type EntityID = usize;

/// This ECS works by storing a bunch of archetypes where each archetype stores all the components
/// of entties that have the same components.
/// This means that it is more optimized for querying components rather than adding/removing components.
#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,
    /// Maps to the archetype index inside self.archetypes
    type_ids_index_map: hashbrown::HashMap<Box<[TypeId]>, usize>,
    bundle_id_index_map: hashbrown::HashMap<TypeId, usize>,

    entity_count: EntityID,
}

impl World {
    pub fn spawn<T: ComponentBundle>(&mut self, components: T) -> EntityID {
        let id = self.entity_count;
        self.entity_count += 1;

        let archetype = self.get_archetype::<T>();
        let index = archetype.new_entity(id);
        archetype.put_bundle(index, components);

        id
    }

    fn query<T: ComponentBundle>(&mut self) {
        QueryIter::<T>::new(self);
    }

    fn get_archetype<T: ComponentBundle>(&mut self) -> &mut Archetype {
        // First index the map with the bundle id since that's faster to compute with
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

struct QueryIter<'w, T: ComponentBundle> {
    world: &'w World,
    current_archetype_index: usize,
    current_entity_index: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<'w, T: ComponentBundle> QueryIter<'w, T> {
    pub fn new(world: &'w World) -> Self {
        Self {
            current_archetype_index: 0,
            current_entity_index: 0,
            world,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn next_archetype(&self) -> Option<()> {
        let archetype = self.world.archetypes.get(self.current_archetype_index)?;
        Some(())
    }
}

impl<'w, T: ComponentBundle> Iterator for QueryIter<'w, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_archetype_index
    }
}
