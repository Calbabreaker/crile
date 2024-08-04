pub use super::*;

#[test]
pub fn spawn_hierachy() {
    let mut scene = Scene::with_root();
    let parent = scene.spawn(
        "Parent",
        (TransformComponent::default(),),
        Scene::ROOT_INDEX,
    );
    let child = scene.spawn("Child", (TransformComponent::default(),), parent);
    let parent_node = scene.get_node(parent).unwrap();
    let child_node = scene.get_node(child).unwrap();

    assert_eq!(parent_node.name, "Parent");
    assert_eq!(parent_node.children, vec![child_node.id]);
    assert_eq!(parent_node.parent, scene.root_node().id);
    assert_eq!(child_node.name, "Child");
    assert_eq!(child_node.children, vec![]);
    assert_eq!(child_node.parent, parent_node.id);
}

#[test]
pub fn children_iterator() {
    let mut scene = Scene::with_root();
    let parent = scene.spawn(
        "Parent",
        (TransformComponent::default(),),
        Scene::ROOT_INDEX,
    );
    let child = scene.spawn("Child", (TransformComponent::default(),), parent);
    let child2 = scene.spawn("Child2", (TransformComponent::default(),), parent);
    let parent2 = scene.spawn(
        "Parent2",
        (TransformComponent::default(),),
        Scene::ROOT_INDEX,
    );

    let mut iter = scene.hierarchy_iter(Scene::ROOT_INDEX);
    let index = iter.next().unwrap();
    assert_eq!(scene.get_node(index).unwrap().name, "Root");
    assert_eq!(index, Scene::ROOT_INDEX);

    assert_eq!(iter.next().unwrap(), parent);
    assert_eq!(iter.next().unwrap(), child);
    assert_eq!(iter.next().unwrap(), child2);
    assert_eq!(iter.next().unwrap(), parent2);
    assert_eq!(iter.next(), None);
}

#[test]
pub fn parent_iterator() {
    let mut scene = Scene::with_root();
    let parent = scene.spawn(
        "Parent",
        (TransformComponent::default(),),
        Scene::ROOT_INDEX,
    );
    scene.spawn("Child", (TransformComponent::default(),), parent);
    let child = scene.spawn("Child2", (TransformComponent::default(),), parent);
    scene.spawn(
        "Parent2",
        (TransformComponent::default(),),
        Scene::ROOT_INDEX,
    );

    let mut iter = scene.ancestor_iter(child);
    let index = iter.next().unwrap();
    assert_eq!(scene.get_node(index).unwrap().name, "Parent");
    assert_eq!(index, parent);
    assert_eq!(iter.next().unwrap(), Scene::ROOT_INDEX);
    assert_eq!(iter.next(), None);
}

// #[test]
// pub fn despawn() {
//     let mut scene = Scene::with_root();
//     let parent = scene.spawn(
//         "Parent",
//         (TransformComponent::default(),),
//         Scene::ROOT_INDEX,
//     );
//     scene.spawn("Child", (TransformComponent::default(),), parent);
//     scene.spawn("Child2", (TransformComponent::default(),), parent);
//     let parent2 = scene.spawn(
//         "Parent2",
//         (TransformComponent::default(),),
//         Scene::ROOT_INDEX,
//     );

//     scene.despawn(parent);

//     assert_eq!(
//         scene.get_node(Scene::ROOT_INDEX).unwrap().children,
//         vec![scene.get_node(parent2).unwrap().id]
//     );
// }
