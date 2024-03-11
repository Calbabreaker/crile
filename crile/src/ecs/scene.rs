use super::{CameraComponent, SpriteRendererComponent, TransformComponent, World};
use crate::{
    ComponentTuple, DrawUniform, EntityId, EntityRef, MetaDataComponent, RenderInstance, RenderPass,
};

pub struct Scene {
    pub world: World,
    root_entity_id: EntityId,
    render_instances: Vec<RenderInstance>,
}

impl Default for Scene {
    fn default() -> Self {
        let mut world = World::default();
        Self {
            root_entity_id: world.spawn((MetaDataComponent {
                name: "Root".to_owned(),
                ..Default::default()
            },)),
            world,
            render_instances: Vec::default(),
        }
    }
}

impl Scene {
    pub fn render(&mut self, render_pass: &mut RenderPass) {
        if let Some((_, (camera_transform, camera))) = self
            .world
            .query::<(TransformComponent, CameraComponent)>()
            .next()
        {
            self.render_instances.clear();

            for (id, (transform, sprite)) in self
                .world
                .query::<(TransformComponent, SpriteRendererComponent)>()
            {
                // Go through each parent and multiple by their transforms
                // TODO: a bit inefficient think about caching?
                let mut global_transform = transform.get_matrix();
                self.for_each_parent(id, &mut |parent_id| {
                    if let Some(transform) = self.world.get::<TransformComponent>(parent_id) {
                        global_transform = transform.get_matrix() * global_transform;
                    }
                });

                self.render_instances.push(RenderInstance {
                    transform: global_transform,
                    color: sprite.color,
                })
            }

            let view_matrix = camera_transform.get_matrix().inverse();

            render_pass.set_texture(&render_pass.data.white_texture);
            render_pass.set_uniform(DrawUniform {
                transform: view_matrix * camera.projection(),
            });
            render_pass.set_shader(render_pass.data.instanced_shader.clone());
            render_pass
                .draw_mesh_instanced(render_pass.data.square_mesh.view(), &self.render_instances);
        }
    }

    pub fn set_viewport(&mut self, viewport_size: glam::Vec2) {
        for (_, (camera,)) in self.world.query_mut::<(CameraComponent,)>() {
            camera.set_viewport(viewport_size);
        }
    }

    /// Spawns an entity with the components in the world with a parent or root if none provided
    pub fn spawn<T: ComponentTuple>(
        &mut self,
        name: &str,
        components: T,
        parent_id: Option<EntityId>,
    ) -> EntityId {
        let id = self.world.spawn(components);

        let parent_id = parent_id.unwrap_or(self.root_entity_id);
        let meta = self
            .world
            .get::<MetaDataComponent>(parent_id)
            .expect("invalid parent");
        meta.children.push(id);

        let mut entity = self.world.entity_mut(id).unwrap();
        entity.add(MetaDataComponent {
            name: name.to_owned(),
            parent: parent_id,
            ..Default::default()
        });

        id
    }

    /// Despawns the entity and its children recursively
    pub fn despawn(&mut self, id: EntityId) {
        let meta = self
            .world
            .get::<MetaDataComponent>(id)
            .expect("despawning invalid entity");

        // Remove the child from the children array inside the parent
        if let Some(parent_meta) = self.world.get::<MetaDataComponent>(meta.parent) {
            if let Some(pos) = parent_meta.children.iter().position(|x| *x == id) {
                parent_meta.children.remove(pos);
            }
        }

        let mut children_list = Vec::new();
        self.for_each_child(meta, &mut |id| children_list.push(id));
        for child in children_list {
            self.world.despawn(child);
        }

        self.world.despawn(id);
    }

    pub fn for_each_child(&self, meta: &MetaDataComponent, func: &mut impl FnMut(EntityId)) {
        for child in meta.children.clone() {
            if let Some(meta) = self.world.get::<MetaDataComponent>(child) {
                self.for_each_child(meta, func);
            }

            func(child);
        }
    }

    pub fn for_each_parent(&self, id: EntityId, func: &mut impl FnMut(EntityId)) {
        if let Some(meta) = self.world.get::<MetaDataComponent>(id) {
            if meta.parent != self.root_entity_id {
                self.for_each_parent(meta.parent, func);
                func(meta.parent);
            }
        }
    }

    pub fn root_entity(&self) -> EntityRef {
        self.world.entity(self.root_entity_id).unwrap()
    }
}
