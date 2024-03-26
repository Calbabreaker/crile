use std::any::TypeId;

use super::{Archetype, ComponentTuple, QueryIter, QueryIterMut, TypeInfo};
use crate::index_mut_twice;

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

    free_entity_ids: Vec<EntityId>,
    next_free_id: EntityId,
    entity_locations: Vec<EntityLocation>,
}

impl World {
    pub fn spawn<T: ComponentTuple>(&mut self, components: T) -> EntityId {
        let id = self.free_entity_ids.pop().unwrap_or(self.next_free_id);
        self.spawn_with_id(id, components);

        id
    }

    pub fn spawn_with_id<T: ComponentTuple>(&mut self, id: EntityId, components: T) {
        self.spawn_raw(id, &T::type_infos(), |index, put_func| {
            components.take_all(index, put_func)
        });
    }

    /// Spawns a entity with component by directly dropping into archetype
    /// take_func expects a closure that will call archetype.put_component with the provided entity index
    pub fn spawn_raw(
        &mut self,
        id: EntityId,
        type_infos: &[TypeInfo],
        take_func: impl FnOnce(usize, &mut Archetype),
    ) {
        assert!(
            self.entity_locations.get(id).map_or(true, |l| !l.valid),
            "id {id} already in use"
        );

        let archetype_index = self.archetype_set.index_from_infos(type_infos);
        let archetype = &mut self.archetype_set.archetypes[archetype_index];

        let entity_index = archetype.new_entity(id);
        take_func(entity_index, archetype);

        if id >= self.entity_locations.len() {
            self.entity_locations
                .resize_with(id + 1, EntityLocation::default);
            self.next_free_id = id + 1;
        }

        self.entity_locations[id] = EntityLocation {
            archetype_index,
            entity_index,
            valid: true,
        };
    }

    pub fn despawn(&mut self, id: EntityId) {
        let location = &mut self.entity_locations[id];
        location.valid = false;

        let archetype = &mut self.archetype_set.archetypes[location.archetype_index];
        let moved_id = archetype.remove_entity(location.entity_index, true);
        self.entity_locations[moved_id].entity_index = location.entity_index;

        self.free_entity_ids.push(id);
    }

    pub fn query<T: ComponentTuple>(&self) -> QueryIter<T> {
        QueryIter::new(self)
    }

    pub fn query_mut<T: ComponentTuple>(&mut self) -> QueryIterMut<T> {
        QueryIterMut::new(self)
    }

    pub fn entity(&self, id: EntityId) -> Option<EntityRef> {
        Some(EntityRef::new(self, self.location(id)?, id))
    }

    pub fn entity_mut(&mut self, id: EntityId) -> Option<EntityMut> {
        Some(EntityMut::new(self, self.location(id)?, id))
    }

    /// Gets a component from the entity id
    /// Shorthand for self.entity(id)?.get<T>()?;
    pub fn get<T: 'static>(&self, id: EntityId) -> Option<&mut T> {
        let location = self.location(id)?;
        let archetype = &self.archetype_set.archetypes[location.archetype_index];
        archetype.borrow_component(location.entity_index)
    }

    fn location(&self, id: EntityId) -> Option<EntityLocation> {
        let location = self.entity_locations.get(id)?;
        if location.valid {
            Some(*location)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct ArchetypeSet {
    pub(crate) archetypes: Vec<Archetype>,
    /// Maps to the archetype index inside self.archetypes from a array of component type ids
    type_ids_index_map: hashbrown::HashMap<Box<[TypeInfo]>, usize>,
}

impl ArchetypeSet {
    fn index_from_infos(&mut self, infos: &[TypeInfo]) -> usize {
        // Returns the archetype with the ids or creates a new one
        *self.type_ids_index_map.entry_ref(infos).or_insert_with(|| {
            let archetype = Archetype::new(infos);
            self.archetypes.push(archetype);
            self.archetypes.len() - 1
        })
    }
}

#[derive(Clone, Copy)]
pub struct EntityRef<'a> {
    archetype: &'a Archetype,
    location: EntityLocation,
    id: EntityId,
}

impl<'a> EntityRef<'a> {
    fn new(world: &'a World, location: EntityLocation, id: EntityId) -> Self {
        let archetype = &world.archetype_set.archetypes[location.archetype_index];

        Self {
            archetype,
            location,
            id,
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&mut T> {
        self.archetype.borrow_component(self.location.entity_index)
    }

    pub fn has<T: 'static>(&self) -> bool {
        self.archetype.has_component::<T>()
    }

    pub fn id(&self) -> EntityId {
        self.id
    }
}

// Same as entity ref but with add and remove component functions
pub struct EntityMut<'a> {
    archetype: &'a Archetype,
    location: EntityLocation,
    id: EntityId,
    world: &'a mut World,
}

impl<'a> EntityMut<'a> {
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

    pub fn has<T: 'static>(&self) -> bool {
        self.archetype.has_component::<T>()
    }

    pub fn add<T: 'static>(&mut self, component: T) {
        // Get the new archetype that the entity belongs in with component added
        let mut type_infos = self.archetype.type_info_iter().collect::<Vec<_>>();
        let pos = type_infos.binary_search(&TypeInfo::of::<T>()).unwrap_err();
        type_infos.insert(pos, TypeInfo::of::<T>());

        let entity_index = self.location.entity_index;
        self.modify_components(
            &type_infos,
            |source_arch, target_arch, _, target_index| unsafe {
                // Move all the components into the new archetype
                for array in source_arch.component_arrays.iter() {
                    let offset = array.type_info.layout.size() * entity_index;
                    target_arch.put_component(
                        target_index,
                        array.ptr.add(offset),
                        array.type_info.id,
                    );
                }

                // Add the requested component into the new archetype
                target_arch.put_component(
                    target_index,
                    &component as *const T as *const u8,
                    TypeId::of::<T>(),
                );
            },
        );

        // Make sure component doesn't have it's destructor called
        std::mem::forget(component);
    }

    pub fn remove<T: 'static>(&mut self) {
        // Get the new archetype that the entity belongs in with component removed
        let type_infos = self
            .archetype
            .type_info_iter()
            .filter(|info| info.id != TypeId::of::<T>())
            .collect::<Box<_>>();

        let entity_index = self.location.entity_index;
        self.modify_components(
            &type_infos,
            |source_arch, target_arch, source_index, target_index| unsafe {
                // Move all the components into the new archetype except for the removed component
                for array in source_arch.component_arrays.iter_mut() {
                    if array.type_info.id == TypeId::of::<T>() {
                        // Call drop on removed component
                        array.drop_component(source_index);
                    } else {
                        // Put the component into the new archetype
                        let offset = array.type_info.layout.size() * entity_index;
                        target_arch.put_component(
                            target_index,
                            array.ptr.add(offset),
                            array.type_info.id,
                        );
                    }
                }
            },
        );
    }

    fn modify_components(
        &mut self,
        new_type_infos: &[TypeInfo],
        modify_func: impl Fn(&mut Archetype, &mut Archetype, usize, usize),
    ) {
        let target_arch_index = self.world.archetype_set.index_from_infos(new_type_infos);

        if self.location.archetype_index == target_arch_index {
            return;
        }

        let (source_arch, target_arch) = index_mut_twice(
            &mut self.world.archetype_set.archetypes,
            self.location.archetype_index,
            target_arch_index,
        );

        let target_index = target_arch.new_entity(self.id);
        let source_index = self.location.entity_index;
        modify_func(source_arch, target_arch, source_index, target_index);

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
