use std::cell::RefCell;

use rand::Rng;

use crate::{
    CameraComponent, ComponentTuple, DrawUniform, NoHashHashMap, RefId, RenderInstance, RenderPass,
    SpriteComponent, Texture, TransformComponent, World,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct HierarchyId(pub u32);

#[derive(Clone, Default, Debug)]
pub struct HierarchyNode {
    pub name: String,
    pub parent: HierarchyId,
    pub children: Vec<HierarchyId>,
    pub id: HierarchyId,
}

impl HierarchyNode {
    pub fn new(name: impl ToString, id: HierarchyId, parent: HierarchyId) -> Self {
        Self {
            name: name.to_string(),
            parent,
            id,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct Scene {
    pub world: World,
    /// Maps entity index (inside world) to hierachy node information
    pub(crate) hierarchy_nodes: Vec<HierarchyNode>,
    /// Maps a hierarchy id to an entity index
    pub(crate) hierachy_id_index_map: NoHashHashMap<HierarchyId, usize>,
    render_instances_map: NoHashHashMap<RefId<Texture>, Vec<RenderInstance>>,
    pub running: bool,
}

impl Scene {
    /// Index of the root entity (this will always be zero since it is the first one spawned in)
    pub const ROOT_INDEX: usize = 0;

    /// Creates a scene with a root entity
    pub fn with_root() -> Self {
        let mut scene = Scene::default();
        let id = scene.random_hierarchy_id();
        scene.world.spawn(());
        scene.add_to_hierarchy("Root", Self::ROOT_INDEX, id, HierarchyId(0));
        scene
    }

    // TODO: Render back to front to support transparency
    pub fn render(&mut self, render_pass: &mut RenderPass, view_projection: glam::Mat4) {
        for instances in self.render_instances_map.values_mut() {
            instances.clear();
        }

        for (index, (transform, sprite)) in
            self.world.query::<(TransformComponent, SpriteComponent)>()
        {
            // Go through each parent and multiply by their transforms
            // TODO: a bit inefficient think about caching?
            let mut global_transform = transform.matrix();
            for parent_index in self.ancestor_iter(index) {
                if let Some(transform) = self.world.get::<TransformComponent>(parent_index) {
                    global_transform = transform.matrix() * global_transform;
                }
            }

            let texture = sprite
                .texture
                .as_ref()
                .unwrap_or(&render_pass.data.white_texture);
            let instances = self.render_instances_map.entry_ref(texture).or_default();

            instances.push(RenderInstance {
                transform: global_transform
                    * glam::Mat4::from_scale(texture.view().size().as_vec2().extend(1.)),
                color: sprite.color,
            })
        }

        render_pass.set_uniform(DrawUniform {
            transform: view_projection,
        });
        render_pass.set_shader(render_pass.data.instanced_shader.clone());
        for (texture, instances) in &self.render_instances_map {
            if !instances.is_empty() {
                render_pass.set_texture(texture);
                render_pass.draw_mesh_instanced(render_pass.data.square_mesh.view(), instances);
            }
        }
    }

    pub fn set_viewport(&mut self, viewport_size: glam::Vec2) {
        for (_, (camera,)) in self.world.query_mut::<(CameraComponent,)>() {
            camera.dirty = true;
            camera.viewport_size = viewport_size;
        }
    }

    /// Spawns an entity with the components in the world with a parent
    pub fn spawn<T: ComponentTuple>(
        &mut self,
        name: impl ToString,
        components: T,
        parent_index: usize,
    ) -> usize {
        let index = self.world.spawn(components);
        let parent_id = self.hierarchy_nodes[parent_index].id;
        let entity_id = self.random_hierarchy_id();
        self.add_to_hierarchy(name, index, entity_id, parent_id);
        index
    }

    pub fn add_to_hierarchy(
        &mut self,
        name: impl ToString,
        entity_index: usize,
        entity_id: HierarchyId,
        parent_id: HierarchyId,
    ) {
        debug_assert!(
            self.world.exists(entity_index),
            "entity {entity_index} does not exist"
        );

        if entity_index >= self.hierarchy_nodes.len() {
            self.hierarchy_nodes
                .resize_with(entity_index + 1, HierarchyNode::default);
        }

        self.hierarchy_nodes[entity_index] = HierarchyNode::new(name, entity_id, parent_id);
        self.hierachy_id_index_map.insert(entity_id, entity_index);

        if entity_index != Self::ROOT_INDEX {
            debug_assert!(
                self.hierachy_id_index_map.contains_key(&parent_id),
                "parent {parent_id:?} does not exist"
            );
            let parent_node = self.get_node_mut(self.id_to_index(parent_id)).unwrap();
            parent_node.children.push(entity_id);
        }
    }

    /// Despawns the entity and its children recursively
    pub fn despawn(&mut self, entity_index: usize) {
        assert!(
            entity_index != Self::ROOT_INDEX,
            "cannot despawn the root entity"
        );

        let node_id = self.hierarchy_nodes[entity_index].id;
        let parent_index = self
            .ancestor_iter(entity_index)
            .next()
            .expect("parent invalid");

        // Remove the child from the children array inside the parent
        let parent_node = &mut self.hierarchy_nodes[parent_index];
        if let Some(pos) = parent_node.children.iter().position(|x| *x == node_id) {
            parent_node.children.remove(pos);
        }

        let to_remove = self.hierarchy_iter(entity_index).collect::<Vec<_>>();
        for index in to_remove {
            self.world.despawn(index);
            let node = &self.hierarchy_nodes[index];
            self.hierachy_id_index_map.remove(&node.id);
        }
    }

    /// Returns an iterator that returns the entity itself then all its children and all its decendents
    pub fn hierarchy_iter(&self, entity_index: usize) -> SceneHierarchyIter {
        SceneHierarchyIter::new(self, entity_index)
    }

    /// Returns an iterator that goes through all the entity parents and all its ancestors
    pub fn ancestor_iter(&self, entity_index: usize) -> SceneAncestorIter {
        SceneAncestorIter::new(self, entity_index)
    }

    pub fn random_hierarchy_id(&self) -> HierarchyId {
        let id = HierarchyId(rand::thread_rng().gen());
        if self.hierachy_id_index_map.contains_key(&id) {
            // Regen id if conflicts
            self.random_hierarchy_id()
        } else {
            id
        }
    }

    pub fn id_to_index(&self, id: HierarchyId) -> usize {
        *self
            .hierachy_id_index_map
            .get(&id)
            .expect("id should exist")
    }

    pub fn get_node(&self, entity_index: usize) -> Option<&HierarchyNode> {
        self.hierarchy_nodes.get(entity_index)
    }

    pub fn get_node_mut(&mut self, entity_index: usize) -> Option<&mut HierarchyNode> {
        self.hierarchy_nodes.get_mut(entity_index)
    }

    pub fn root_node(&self) -> &HierarchyNode {
        self.hierarchy_nodes
            .get(Self::ROOT_INDEX)
            .expect("should be a root node")
    }
}

pub struct SceneHierarchyIter<'a> {
    scene: &'a Scene,
}

impl<'a> SceneHierarchyIter<'a> {
    thread_local! {
        static NEXT_INDEXES_STACK: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
    }

    fn new(scene: &'a Scene, start_index: usize) -> Self {
        Self::NEXT_INDEXES_STACK.with(|stack| stack.borrow_mut().push(start_index));

        Self { scene }
    }
}

impl<'a> Iterator for SceneHierarchyIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Self::NEXT_INDEXES_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let index = stack.pop()?;
            let node = self.scene.hierarchy_nodes.get(index)?;
            stack.extend(node.children.iter().rev().map(|id| {
                // Add the children entity indexes
                self.scene.id_to_index(*id)
            }));

            Some(index)
        })
    }
}

pub struct SceneAncestorIter<'a> {
    scene: &'a Scene,
    next_index: usize,
}

impl<'a> SceneAncestorIter<'a> {
    fn new(scene: &'a Scene, start_index: usize) -> Self {
        Self {
            scene,
            next_index: start_index,
        }
    }
}

impl<'a> Iterator for SceneAncestorIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == Scene::ROOT_INDEX {
            return None;
        }

        let node = self.scene.hierarchy_nodes.get(self.next_index)?;
        let parent_index = *self.scene.hierachy_id_index_map.get(&node.parent)?;
        self.next_index = parent_index;
        Some(parent_index)
    }
}
