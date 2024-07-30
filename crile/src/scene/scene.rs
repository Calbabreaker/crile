use std::cell::RefCell;

use crate::{
    CameraComponent, ComponentTuple, DrawUniform, EntityId, RefId, RenderInstance, RenderPass,
    SpriteComponent, Texture, TransformComponent, World,
};

#[derive(Clone, Default, Debug)]
pub struct HierachyNode {
    pub name: String,
    pub parent: EntityId,
    pub children: Vec<EntityId>,
}

impl HierachyNode {
    pub fn new(name: impl ToString, parent: EntityId) -> Self {
        Self {
            name: name.to_string(),
            parent,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct Scene {
    pub world: World,
    // Index will be same as entity id
    pub(crate) hierachy_nodes: Vec<HierachyNode>,
    render_instances_map: hashbrown::HashMap<RefId<Texture>, Vec<RenderInstance>>,
    pub running: bool,
}

impl Scene {
    pub const ROOT_ID: EntityId = 0;

    /// Creates a scene with a root entity
    pub fn with_root() -> Self {
        let mut scene = Scene::default();
        let id = scene.spawn("Root", (), 0);
        debug_assert!(
            id == Self::ROOT_ID,
            "id was not the root id for some reason"
        );
        scene
    }

    // TODO: Render back to front to support transparency
    pub fn render(&mut self, render_pass: &mut RenderPass, view_projection: glam::Mat4) {
        for instances in self.render_instances_map.values_mut() {
            instances.clear();
        }

        for (id, (transform, sprite)) in self.world.query::<(TransformComponent, SpriteComponent)>()
        {
            // Go through each parent and multiply by their transforms
            // TODO: a bit inefficient think about caching?
            let mut global_transform = transform.matrix();
            for (_, parent_id) in self.parent_iter(id) {
                if let Some(transform) = self.world.get::<TransformComponent>(parent_id) {
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
        parent_id: EntityId,
    ) -> EntityId {
        dbg!(name.to_string());
        let id = self.world.spawn(components);
        self.add_to_hierachy(name, id, parent_id);
        id
    }

    pub fn add_to_hierachy(&mut self, name: impl ToString, id: EntityId, parent_id: EntityId) {
        debug_assert!(self.world.exists(id), "entity {id} does not exist");
        debug_assert!(
            self.world.exists(parent_id),
            "parent entity {parent_id} does not exist"
        );

        if id >= self.hierachy_nodes.len() {
            self.hierachy_nodes
                .resize_with(id + 1, HierachyNode::default);
        }

        self.hierachy_nodes[id] = HierachyNode::new(name, parent_id);

        if id != Self::ROOT_ID {
            debug_assert!(id != parent_id, "id was the same as the parent id");
            self.hierachy_nodes[parent_id].children.push(id);
        }
    }

    /// Despawns the entity and its children recursively
    pub fn despawn(&mut self, id: EntityId) {
        let parent_index = self.hierachy_nodes[id].parent;
        let parent_node = &mut self.hierachy_nodes[parent_index];

        // Remove the child from the children array inside the parent
        if let Some(pos) = parent_node.children.iter().position(|x| *x == id) {
            parent_node.children.remove(pos);
        }

        for (_, child_id) in SceneHierarchyIter::new(&self.hierachy_nodes, id) {
            self.world.despawn(child_id);
        }
    }

    /// Returns an iterator that returns the entity itself then all its children and all its decendents
    pub fn iter(&self, id: EntityId) -> SceneHierarchyIter {
        SceneHierarchyIter::new(&self.hierachy_nodes, id)
    }
    /// Returns an iterator that goes through all the entity parents and all its ancestors
    pub fn parent_iter(&self, id: EntityId) -> SceneParentIter {
        SceneParentIter::new(&self.hierachy_nodes, id)
    }

    pub fn get_node(&self, id: EntityId) -> Option<&HierachyNode> {
        if !self.world.exists(id) {
            return None;
        }
        self.hierachy_nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: EntityId) -> Option<&mut HierachyNode> {
        if !self.world.exists(id) {
            return None;
        }
        self.hierachy_nodes.get_mut(id)
    }
}

pub struct SceneHierarchyIter<'a> {
    hierachy_nodes: &'a Vec<HierachyNode>,
}

impl<'a> SceneHierarchyIter<'a> {
    thread_local! {
        static NEXT_IDS_STACK: RefCell<Vec<EntityId>> = const { RefCell::new(Vec::new()) };
    }

    fn new(hierachy_nodes: &'a Vec<HierachyNode>, start_id: EntityId) -> Self {
        Self::NEXT_IDS_STACK.with(|stack| stack.borrow_mut().push(start_id));

        Self { hierachy_nodes }
    }
}

impl<'a> Iterator for SceneHierarchyIter<'a> {
    type Item = (&'a HierachyNode, EntityId);

    fn next(&mut self) -> Option<Self::Item> {
        Self::NEXT_IDS_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            let id = stack.pop()?;
            let node = &self.hierachy_nodes[id];
            stack.extend(node.children.iter().rev());
            Some((node, id))
        })
    }
}

pub struct SceneParentIter<'a> {
    hierachy_nodes: &'a Vec<HierachyNode>,
    next_parent_id: Option<usize>,
}

impl<'a> SceneParentIter<'a> {
    fn new(hierachy_nodes: &'a Vec<HierachyNode>, start_id: EntityId) -> Self {
        Self {
            hierachy_nodes,
            next_parent_id: hierachy_nodes.get(start_id).map(|node| node.parent),
        }
    }
}

impl<'a> Iterator for SceneParentIter<'a> {
    type Item = (&'a HierachyNode, EntityId);

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.next_parent_id?;
        let node = &self.hierachy_nodes[id];
        if id == Scene::ROOT_ID {
            self.next_parent_id = None;
        } else {
            self.next_parent_id = Some(node.parent);
        }

        Some((node, id))
    }
}
