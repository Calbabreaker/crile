use std::any::TypeId;

use super::{Archetype, ComponentTuple, QueryIter, QueryIterMut, TypeInfo};
use crate::{index_mut_twice, Component};

pub type EntityId = usize;

#[derive(Clone, Copy, Default, Debug)]
struct EntityLocation {
    archetype_index: usize,
    entity_index: usize,
}

/// Contains all the data for an ECS instance.
/// This ECS works by storing a bunch of archetypes where each archetype stores all the components
/// of entities that have the same set of components.
/// Then when querying, it will go to each archetype and will only get components in that archetype
/// if that archetype contains the components in the query.
/// This means that it is more optimized for querying components rather than adding/removing components
/// and having few different archetypes.
#[derive(Default, Clone)]
pub struct World {
    pub(crate) archetypes: Vec<Archetype>,
    /// Maps to the archetype index inside self.archetypes from a array of component type ids
    type_ids_index_map: hashbrown::HashMap<Box<[TypeInfo]>, usize>,

    free_entity_ids: Vec<EntityId>,
    next_free_id: EntityId,
    entity_locations: Vec<Option<EntityLocation>>,
}

impl World {
    pub fn spawn<T: ComponentTuple>(&mut self, components: T) -> EntityId {
        let id = self.next_free_id();
        self.spawn_with_id(id, components);

        id
    }

    pub fn spawn_with_id<T: ComponentTuple>(&mut self, id: EntityId, components: T) {
        self.spawn_raw(id, &T::type_infos(), |index, archetype| {
            components.take_all(index, archetype)
        });
    }

    /// Spawns a entity with component by directly putting into archetype
    /// take_func expects a closure that will call archetype.put_component with the provided entity index
    pub fn spawn_raw(
        &mut self,
        id: EntityId,
        type_infos: &[TypeInfo],
        take_func: impl FnOnce(usize, &mut Archetype),
    ) {
        assert!(!self.exists(id), "id {id} already in use");

        let archetype_index = self.archetype_index_from_infos(type_infos);
        let archetype = &mut self.archetypes[archetype_index];

        let entity_index = archetype.new_entity(id);
        take_func(entity_index, archetype);

        if id >= self.entity_locations.len() as EntityId {
            self.entity_locations.resize_with(id + 1, Option::default);
            self.next_free_id = id + 1;
        }

        self.entity_locations[id] = Some(EntityLocation {
            archetype_index,
            entity_index,
        });
    }

    pub fn despawn(&mut self, id: EntityId) {
        let location = self.entity_locations[id]
            .take()
            .expect("Tried to despawn already despawned entity");

        let archetype = &mut self.archetypes[location.archetype_index];
        let moved_id = archetype.remove_entity(location.entity_index, true);
        if let Some(moved_location) = self.entity_locations[moved_id].as_mut() {
            moved_location.entity_index = location.entity_index;
        }

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
    pub fn get<T: Component>(&self, id: EntityId) -> Option<&mut T> {
        let location = self.location(id)?;
        let archetype = &self.archetypes[location.archetype_index];
        unsafe { archetype.borrow_component(location.entity_index) }
    }

    pub fn exists(&self, id: EntityId) -> bool {
        self.location(id).is_some()
    }

    pub fn next_free_id(&mut self) -> EntityId {
        self.free_entity_ids.pop().unwrap_or(self.next_free_id)
    }

    fn location(&self, id: EntityId) -> Option<EntityLocation> {
        *self.entity_locations.get(id)?
    }

    fn archetype_index_from_infos(&mut self, infos: &[TypeInfo]) -> usize {
        // Returns the archetype with the ids or creates a new one
        *self.type_ids_index_map.entry_ref(infos).or_insert_with(|| {
            let archetype = Archetype::new(infos.iter().cloned().collect());
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
        Self {
            archetype: &world.archetypes[location.archetype_index],
            location,
            id,
        }
    }

    pub fn get<T: Component>(&'a self) -> Option<&'a mut T> {
        unsafe { self.archetype.borrow_component(self.location.entity_index) }
    }

    pub fn has<T: Component>(&self) -> bool {
        self.archetype.has_component::<T>()
    }

    pub fn id(&self) -> EntityId {
        self.id
    }
}

// Same as entity ref but with add and remove component functions
pub struct EntityMut<'a> {
    archetype: &'a mut Archetype,
    location: EntityLocation,
    id: EntityId,
    world: &'a mut World,
}

impl<'a> EntityMut<'a> {
    fn new(world: &'a mut World, location: EntityLocation, id: EntityId) -> Self {
        // Safety:
        // This archetype reference is never accessed after world is modified so it is safe to use
        let archetype =
            unsafe { &mut (*(world as *mut World)).archetypes[location.archetype_index] };
        Self {
            archetype,
            location,
            id,
            world,
        }
    }

    pub fn get<T: Component>(&'a self) -> Option<&'a mut T> {
        unsafe { self.archetype.borrow_component(self.location.entity_index) }
    }

    pub fn has<T: Component>(&self) -> bool {
        self.archetype.has_component::<T>()
    }

    pub fn add<T: Component>(&mut self, component: T) {
        // Make sure component doesn't have it's destructor called
        let component = std::mem::ManuallyDrop::new(component);

        // Get the new archetype that the entity belongs in with component added
        let mut type_infos = self.archetype.type_infos().to_vec();
        let pos = type_infos.binary_search(&TypeInfo::of::<T>()).unwrap_err();
        type_infos.insert(pos, TypeInfo::of::<T>());

        self.modify_components(
            &type_infos,
            |source_arch, target_arch, source_index, target_index| unsafe {
                // Move all the components into the new archetype
                for array in source_arch.component_arrays.iter() {
                    let offset = source_index * array.type_info.layout.size();
                    target_arch.put_component(
                        target_index,
                        array.ptr.add(offset),
                        array.type_info.id,
                    );
                }

                // Add the requested component into the new archetype
                target_arch.put_component(
                    target_index,
                    &*component as *const T as *const u8,
                    TypeId::of::<T>(),
                );
            },
        );
    }

    pub fn remove<T: Component>(&mut self) {
        // Remove the component from the type infos
        let mut type_infos = self.archetype.type_infos().to_vec();
        let pos = type_infos.binary_search(&TypeInfo::of::<T>()).unwrap();
        type_infos.remove(pos);

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
                        let offset = array.type_info.layout.size() * source_index;
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
        modify_func: impl Fn(&mut Archetype, &mut Archetype, EntityId, EntityId),
    ) {
        let target_arch_index = self.world.archetype_index_from_infos(new_type_infos);

        if self.location.archetype_index == target_arch_index {
            return;
        }

        let (source_arch, target_arch) = index_mut_twice(
            &mut self.world.archetypes,
            self.location.archetype_index,
            target_arch_index,
        );

        let target_index = target_arch.new_entity(self.id);
        let source_index = self.location.entity_index;
        modify_func(source_arch, target_arch, source_index, target_index);

        // Remove the old entity
        let moved_id = source_arch.remove_entity(source_index, false);
        self.world.entity_locations[moved_id].unwrap().entity_index = source_index;

        // Set the new archetype and location
        self.archetype = unsafe { &mut *(target_arch as *mut Archetype) };
        self.location.entity_index = target_index;
        self.location.archetype_index = target_arch_index;
        self.world.entity_locations[self.id] = Some(self.location);
    }

    pub fn id(&self) -> EntityId {
        self.id
    }
}
