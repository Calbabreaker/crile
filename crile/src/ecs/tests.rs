use std::{any::TypeId, rc::Rc};

use super::*;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct Metadata {
    data: Rc<String>,
    stuff: Vec<String>,
}

#[derive(Default, Clone, Debug, PartialEq)]
struct Empty;

#[test]
#[should_panic]
fn empty_component() {
    let mut world = World::default();
    let id = world.spawn((Empty,));
    assert_eq!(id, 0);
    assert_eq!(*world.get::<Empty>(id).unwrap(), Empty);
}

#[test]
fn normal_spawn_1_component() {
    let mut world = World::default();
    let id = world.spawn((Position { x: 1., y: 2. },));
    assert_eq!(id, 0);

    let position = world.get::<Position>(0).unwrap();
    assert_eq!(position.x, 1.);
    assert_eq!(position.y, 2.);
}

#[test]
fn spawn_raw_1_component() {
    let mut world = World::default();
    let position = Position { x: 1., y: 2. };
    world.spawn_raw(&[TypeInfo::of::<Position>()], |index, archetype| unsafe {
        archetype.clone_component(
            index,
            &position as *const Position as *const u8,
            TypeId::of::<Position>(),
        );
    });

    let position = world.get::<Position>(0).unwrap();
    assert_eq!(position.x, 1.);
    assert_eq!(position.y, 2.);
}

#[test]
fn despawn_with_dropable() {
    let mut world = World::default();
    let data = Rc::new("data".to_string());
    let meta = Metadata {
        data: data.clone(),
        ..Default::default()
    };

    let id = world.spawn((meta,));
    assert_eq!(Rc::strong_count(&data), 2);

    world.despawn(id);
    assert_eq!(world.get::<Metadata>(id), None);
    assert_eq!(Rc::strong_count(&data), 1);
}

#[test]
fn add_get_remove_component() {
    let mut world = World::default();
    let id = world.spawn((Velocity::default(),));

    let mut entity = world.entity_mut(id).unwrap();
    assert!(entity.has::<Velocity>());
    assert_eq!(*entity.get::<Velocity>().unwrap(), Velocity::default());

    let position = Position { x: 1., y: 2. };
    entity.add(position);
    assert_eq!(*entity.get::<Position>().unwrap(), position);

    entity.remove::<Position>();
    assert!(!entity.has::<Position>());
    assert_eq!(*entity.get::<Velocity>().unwrap(), Velocity::default());
}

#[test]
fn query_world() {
    let mut world = World::default();
    let mut position = Position { x: 1., y: 2. };
    let mut velocity = Velocity { x: 1., y: 2. };
    world.spawn((position, velocity));
    world.spawn((velocity, position));
    assert_eq!(world.archetypes.len(), 1);

    let mut query = world.query_mut::<(Position, Velocity)>();
    assert_eq!(query.next().unwrap(), (0, (&mut position, &mut velocity)));
    assert_eq!(query.next().unwrap(), (1, (&mut position, &mut velocity)));
}

#[test]
#[should_panic]
#[ignore = "TODO: fix this test"]
fn multiple_borrow() {
    let mut world = World::default();
    let id = world.spawn((Metadata::default(),));

    let a = world.get::<Metadata>(id).unwrap();
    a.stuff.push("test".to_string());

    world.get::<Metadata>(id).unwrap();
}

#[test]
#[should_panic]
fn spawn_duplicate_components() {
    let mut world = World::default();
    world.spawn((Position { x: 1., y: 2. }, Position { x: 2., y: 2. }));
}
