pub use super::*;

#[test]
pub fn spawn_hierachy() {
    let mut scene = Scene::with_root();
    let parent_id = scene.spawn("Parent", (TransformComponent::default(),), Scene::ROOT_ID);
    let child_id = scene.spawn("Child", (TransformComponent::default(),), parent_id);

    assert_eq!(scene.get_node(parent_id).unwrap().name, "Parent");
    assert_eq!(scene.get_node(parent_id).unwrap().children, vec![child_id]);
    assert_eq!(scene.get_node(parent_id).unwrap().parent, Scene::ROOT_ID);
    assert_eq!(scene.get_node(child_id).unwrap().name, "Child");
    assert_eq!(scene.get_node(child_id).unwrap().children, vec![]);
    assert_eq!(scene.get_node(child_id).unwrap().parent, parent_id);
}

#[test]
pub fn children_iterator() {
    let mut scene = Scene::with_root();
    scene.spawn("Parent2", (TransformComponent::default(),), Scene::ROOT_ID);
    let parent_id = scene.spawn("Parent", (TransformComponent::default(),), Scene::ROOT_ID);
    scene.spawn("Child2", (TransformComponent::default(),), parent_id);
    scene.spawn("Child", (TransformComponent::default(),), parent_id);

    let mut iter = scene.iter(Scene::ROOT_ID);
    let (root, id1) = iter.next().unwrap();
    assert_eq!(root.name, "Root");
    assert_eq!(id1, Scene::ROOT_ID);

    assert_eq!(iter.next().unwrap().0.name, "Parent");
    assert_eq!(iter.next().unwrap().0.name, "Child");
    assert_eq!(iter.next().unwrap().0.name, "Child2");
    assert_eq!(iter.next().unwrap().0.name, "Parent2");
}

#[test]
pub fn parent_iterator() {
    let mut scene = Scene::with_root();
    let parent_id = scene.spawn("Parent", (TransformComponent::default(),), Scene::ROOT_ID);
    scene.spawn("Child", (TransformComponent::default(),), parent_id);
    let child_id = scene.spawn("Child2", (TransformComponent::default(),), parent_id);
    scene.spawn("Parent2", (TransformComponent::default(),), Scene::ROOT_ID);

    let mut iter = scene.parent_iter(child_id);
    let (parent1, id1) = iter.next().unwrap();
    assert_eq!(parent1.name, "Parent");
    assert_eq!(id1, parent_id);
    assert_eq!(iter.next().unwrap().0.name, "Root");
}
