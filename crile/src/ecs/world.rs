use std::any::TypeId;

use super::{Archetype, ComponentTuple, QueryIter, QueryIterMut, TypeInfo};
use crate::{index_mut_twice, NoHashHashMap};

pub type EntityId = usize;

#[derive(Clone, Copy, Default, Debug)]
struct EntityLocation {
    archetype_index: usize,
    entity_index: usize,
    valid: bool,
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
    pub(crate) archetype_set: ArchetypeSet,
    /// Maps to the archetype index inside the archetype set from a component tuple id which is faster to compute
    tuple_id_index_map: NoHashHashMap<TypeId, usize>,

    free_entity_ids: Vec<EntityId>,
    entity_count: EntityId,
    entity_locations: Vec<EntityLocation>,
}

impl World {
    pub fn spawn<T: ComponentTuple>(&mut self, components: T) -> EntityId {
        let id = self.alloc_entity();

        // First use the index the map with the comopnent tuple id since that's faster to compute with
        let archetype_index = *self.tuple_id_index_map.entry(T::id()).or_insert_with(|| {
            // Then try with the (sorted) type ids
            self.archetype_set
                .index_from_ids(&T::type_ids(), &T::type_infos())
        });

        let archetype = &mut self.archetype_set.archetypes[archetype_index];
        let entity_index = archetype.new_entity(id);
        components.take_all(|ptr, id| unsafe {
            archetype.put_component(entity_index, ptr, id);
        });

        self.entity_locations[id] = EntityLocation {
            archetype_index,
            entity_index,
            valid: true,
        };

        id
    }

    pub fn despawn(&mut self, id: EntityId) {
        let location = &self.entity_locations[id];
        let archetype = &mut self.archetype_set.archetypes[location.archetype_index];
        let moved_id = archetype.remove_entity(location.entity_index, true);
        self.entity_locations[moved_id].entity_index = location.entity_index;

        self.free_entity_ids.push(id);
        self.entity_locations[id].valid = false;
    }

    pub fn query<T: ComponentTuple>(&self) -> QueryIter<T> {
        QueryIter::new(self)
    }

    pub fn query_mut<T: ComponentTuple>(&mut self) -> QueryIterMut<T> {
        QueryIterMut::new(self)
    }

    pub fn entity(&mut self, id: EntityId) -> EntityRef {
        let location = self.entity_locations[id];
        EntityRef::new(self, location, id)
    }

    fn alloc_entity(&mut self) -> EntityId {
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

#[derive(Default)]
pub struct ArchetypeSet {
    pub(crate) archetypes: Vec<Archetype>,
    /// Maps to the archetype index inside self.archetypes from a array of component type ids
    type_ids_index_map: hashbrown::HashMap<Box<[TypeId]>, usize>,
}

impl ArchetypeSet {
    fn index_from_ids(&mut self, ids: &[TypeId], infos: &[TypeInfo]) -> usize {
        *self.type_ids_index_map.entry_ref(ids).or_insert_with(|| {
            let archetype = Archetype::new(infos);
            self.archetypes.push(archetype);
            self.archetypes.len() - 1
        })
    }
}

pub struct EntityRef<'a> {
    archetype: &'a Archetype,
    location: EntityLocation,
    id: EntityId,
    world: &'a mut World,
}

impl<'a> EntityRef<'a> {
    fn new(world: &'a mut World, location: EntityLocation, id: EntityId) -> Self {
        // Safety:
        // This archetype reference is never accessed after world is modified so it is safe to use
        let archetype = unsafe {
            &(*(world as *const World)).archetype_set.archetypes[location.archetype_index]
            //
        };

        Self {
            archetype,
            location,
            id,
            world,
        }
    }

    // TODO: add borrow checking probably unsafe right now
    pub fn get<T: 'static>(&self) -> Option<&mut T> {
        self.archetype.borrow_component(self.location.entity_index)
    }

    pub fn add<T: 'static>(&mut self, component: T) {
        // Get the new archetype that the entity belongs in with component added
        let mut type_infos = self.archetype.type_info_iter().collect::<Vec<_>>();
        let pos = type_infos.binary_search(&TypeInfo::of::<T>()).unwrap_err();
        type_infos.insert(pos, TypeInfo::of::<T>());

        self.modify_components(&type_infos, |source_arch, target_arch, target_index| {
            // Move all the components into the new archetype
            for array in source_arch.component_arrays.iter() {
                unsafe {
                    target_arch.put_component(target_index, array.ptr, array.type_info.id);
                }
            }

            // Add the requested component into the new archetype
            unsafe {
                target_arch.put_component(
                    target_index,
                    &component as *const T as *const u8,
                    TypeId::of::<T>(),
                );
            }
        });
    }

    pub fn remove<T: 'static>(&mut self) {
        // Get the new archetype that the entity belongs in with component removed
        let type_infos = self
            .archetype
            .type_info_iter()
            .filter(|info| info.id != TypeId::of::<T>())
            .collect::<Box<_>>();

        self.modify_components(&type_infos, |source_arch, target_arch, target_index| {
            // Move all the components into the new archetype except for the removed component
            for array in source_arch.component_arrays.iter() {
                if array.type_info.id != TypeId::of::<T>() {
                    unsafe {
                        target_arch.put_component(target_index, array.ptr, array.type_info.id);
                    }
                }
            }
        });
    }

    fn modify_components(
        &mut self,
        new_type_infos: &[TypeInfo],
        modify_func: impl Fn(&mut Archetype, &mut Archetype, usize),
    ) {
        let type_ids = new_type_infos
            .iter()
            .map(|info| info.id)
            .collect::<Box<_>>();
        let target_arch_index = self
            .world
            .archetype_set
            .index_from_ids(&type_ids, new_type_infos);

        if self.location.archetype_index == target_arch_index {
            return;
        }

        let (source_arch, target_arch) = index_mut_twice(
            &mut self.world.archetype_set.archetypes,
            self.location.archetype_index,
            target_arch_index,
        );

        let target_index = target_arch.new_entity(self.id);
        modify_func(source_arch, target_arch, target_index);

        // Remove the old entity
        let moved_id = source_arch.remove_entity(self.location.entity_index, false);
        self.world.entity_locations[moved_id].entity_index = self.location.entity_index;

        // Set the new archetype and location
        self.archetype = unsafe { &*(target_arch as *const Archetype) };
        self.location.entity_index = target_index;
        self.location.archetype_index = target_arch_index;
        self.world.entity_locations[self.id] = self.location;
    }

    pub fn id(&self) -> EntityId {
        self.id
    }
}
