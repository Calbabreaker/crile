use std::any::TypeId;

use super::{Archetype, ComponentTuple, QueryIter, QueryIterMut, TypeInfo};
use crate::{index_mut_twice, Component};

#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct EntityLocation {
    archetype_index: usize,
    component_index: usize,
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

    free_entity_indexs: Vec<usize>,
    pub(crate) entity_locations: Vec<EntityLocation>,
    valid_entity_locations: Vec<bool>, // Keep bool seperate to save memory because of alignement
}

impl World {
    pub fn spawn<T: ComponentTuple>(&mut self, components: T) -> usize {
        self.spawn_raw(&T::type_infos(), |index, archetype| {
            components.move_all(index, archetype)
        })
    }

    /// Spawns an entity with components specified by the type infos a function instead of tuple templates
    /// put_func expects a closure that will call archetype.put_component with the provided entity index
    pub fn spawn_raw(
        &mut self,
        type_infos: &[TypeInfo],
        put_func: impl FnOnce(usize, &mut Archetype),
    ) -> usize {
        let index = self.next_free_index();
        assert!(!self.exists(index), "id {index} already in use");

        let archetype_index = self.archetype_index_from_infos(type_infos);
        let archetype = &mut self.archetypes[archetype_index];

        let entity_index = archetype.new_entity(index);
        put_func(entity_index, archetype);

        if index >= self.entity_locations.len() {
            self.entity_locations
                .resize_with(index + 1, Default::default);
            self.valid_entity_locations.resize_with(index + 1, || false);
        }

        self.entity_locations[index] = EntityLocation {
            archetype_index,
            component_index: entity_index,
        };
        self.valid_entity_locations[index] = true;

        index
    }

    pub fn despawn(&mut self, index: usize) {
        let location = self
            .location(index)
            .expect("tried to despawn non-existent entity");

        let archetype = &mut self.archetypes[location.archetype_index];
        let moved_index = archetype.remove_entity(location.component_index, true);
        self.entity_locations[moved_index].component_index = location.component_index;
        self.valid_entity_locations[index] = false;

        self.free_entity_indexs.push(index);
    }

    pub fn query<T: ComponentTuple>(&self) -> QueryIter<T> {
        QueryIter::new(self)
    }

    pub fn query_mut<T: ComponentTuple>(&mut self) -> QueryIterMut<T> {
        QueryIterMut::new(self)
    }

    pub fn entity(&self, index: usize) -> Option<EntityRef> {
        Some(EntityRef::new(self, self.location(index)?, index))
    }

    pub fn entity_mut(&mut self, index: usize) -> Option<EntityMut> {
        Some(EntityMut::new(self, self.location(index)?, index))
    }

    /// Gets a component from the entity index
    /// Shorthand for self.entity(index)?.get<T>()?;
    pub fn get<T: Component>(&self, index: usize) -> Option<&mut T> {
        let location = self.location(index)?;
        let archetype = &self.archetypes[location.archetype_index];
        unsafe { archetype.borrow_component(location.component_index) }
    }

    pub fn exists(&self, index: usize) -> bool {
        self.location(index).is_some()
    }

    pub fn next_free_index(&mut self) -> usize {
        self.free_entity_indexs
            .pop()
            .unwrap_or(self.entity_locations.len())
    }

    fn location(&self, index: usize) -> Option<EntityLocation> {
        if *self.valid_entity_locations.get(index)? {
            self.entity_locations.get(index).copied()
        } else {
            None
        }
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
    index: usize,
}

impl<'a> EntityRef<'a> {
    fn new(world: &'a World, location: EntityLocation, index: usize) -> Self {
        Self {
            archetype: &world.archetypes[location.archetype_index],
            location,
            index,
        }
    }

    pub fn get<T: Component>(&'a self) -> Option<&'a mut T> {
        unsafe {
            self.archetype
                .borrow_component(self.location.component_index)
        }
    }

    pub fn has<T: Component>(&self) -> bool {
        self.archetype.has_component::<T>()
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

// Same as entity ref but with add and remove component functions
pub struct EntityMut<'a> {
    archetype: &'a mut Archetype,
    location: EntityLocation,
    index: usize,
    world: &'a mut World,
}

impl<'a> EntityMut<'a> {
    fn new(world: &'a mut World, location: EntityLocation, index: usize) -> Self {
        // Safety:
        // This archetype reference is never accessed after world is modified so it is safe to use
        let archetype =
            unsafe { &mut (*(world as *mut World)).archetypes[location.archetype_index] };
        Self {
            archetype,
            location,
            index,
            world,
        }
    }

    pub fn get<T: Component>(&'a self) -> Option<&'a mut T> {
        unsafe {
            self.archetype
                .borrow_component(self.location.component_index)
        }
    }

    pub fn has<T: Component>(&self) -> bool {
        self.archetype.has_component::<T>()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn add<T: Component>(&mut self, component: T) {
        // Make sure component doesn't have it's destructor called
        let component = std::mem::ManuallyDrop::new(component);

        // Get the new archetype that the entity belongs in with component added
        let new_type_info = TypeInfo::of::<T>();
        let mut type_infos = self.archetype.type_infos().to_vec();
        let pos = type_infos.binary_search(&new_type_info).unwrap_err();
        type_infos.insert(pos, new_type_info);

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
                        false,
                    );
                }

                // Add the requested component into the new archetype
                target_arch.put_component(
                    target_index,
                    &*component as *const T as *const u8,
                    TypeId::of::<T>(),
                    false,
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
                            false,
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
        let target_arch_index = self.world.archetype_index_from_infos(new_type_infos);

        if self.location.archetype_index == target_arch_index {
            return;
        }

        let (source_arch, target_arch) = index_mut_twice(
            &mut self.world.archetypes,
            self.location.archetype_index,
            target_arch_index,
        );

        let target_index = target_arch.new_entity(self.index);
        let source_index = self.location.component_index;
        modify_func(source_arch, target_arch, source_index, target_index);

        // Remove the old entity
        let moved_index = source_arch.remove_entity(source_index, false);
        self.world.entity_locations[moved_index].component_index = source_index;

        // Set the new archetype and location
        self.archetype = unsafe { &mut *(target_arch as *mut Archetype) };
        self.location.component_index = target_index;
        self.location.archetype_index = target_arch_index;
        self.world.entity_locations[self.index] = self.location;
    }
}
