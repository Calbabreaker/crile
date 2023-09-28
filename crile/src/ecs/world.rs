use std::any::TypeId;

use super::archetype::Archetype;

pub type EntityID = usize;

/// This ECS works by storing a bunch of archetypes where each archetype stores all the components
/// of entties that have the same components.
/// This means that it is more optimized for querying components rather than adding/removing
/// components.
#[derive(Default)]
pub struct World {
    archetypes: Vec<Archetype>,
    archetype_index_map: hashbrown::HashMap<Box<[TypeId]>, usize>,
    entity_count: EntityID,
}

impl World {
    pub fn spawn(&mut self, component_ids: Box<[TypeId]>) -> EntityID {
        let id = self.entity_count;
        self.entity_count += 1;

        let archetype = self.get_archetype(component_ids);

        id
    }

    pub fn query(&mut self, component_ids: Box<[TypeId]>) {
        let archetype = self.get_archetype(component_ids);
    }

    fn get_archetype(&mut self, component_ids: Box<[TypeId]>) -> &Archetype {
        match self.archetype_index_map.get(&component_ids) {
            Some(index) => &self.archetypes[*index],
            None => {
                let archetype = Archetype::new(&component_ids);
                self.archetypes.push(archetype);
                self.archetype_index_map
                    .insert(component_ids, self.archetypes.len() - 1);
                self.archetypes.last().unwrap()
            }
        }
    }
}
