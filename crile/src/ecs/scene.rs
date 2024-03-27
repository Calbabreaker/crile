use super::{CameraComponent, SpriteComponent, TransformComponent, World};
use crate::{
    ComponentTuple, DrawUniform, EntityId, EntityRef, MetaDataComponent, RefId, RenderInstance,
    RenderPass, Texture,
};

pub struct Scene {
    pub world: World,
    render_instances_map: hashbrown::HashMap<RefId<Texture>, Vec<RenderInstance>>,
}

impl Default for Scene {
    fn default() -> Self {
        let mut world = World::default();
        world.spawn_with_id(
            Self::ROOT_ID,
            (MetaDataComponent {
                name: "Root".to_owned(),
                ..Default::default()
            },),
        );

        Self {
            world,
            render_instances_map: Default::default(),
        }
    }
}

impl Scene {
    pub const ROOT_ID: EntityId = 0;

    /// Creates a completely empty scene with no root entity
    /// Normally you would use Scene::default()
    pub fn empty() -> Self {
        Self {
            world: World::default(),
            render_instances_map: Default::default(),
        }
    }

    pub fn render_runtime(&mut self, render_pass: &mut RenderPass) {
        if let Some((_, (camera_transform, camera))) = self
            .world
            .query::<(TransformComponent, CameraComponent)>()
            .next()
        {
            self.render(
                render_pass,
                camera.projection() * camera_transform.matrix().inverse(),
            );
        }
    }

    pub fn render(&mut self, render_pass: &mut RenderPass, view_projection: glam::Mat4) {
        for instances in self.render_instances_map.values_mut() {
            instances.clear()
        }

        for (id, (transform, sprite)) in self.world.query::<(TransformComponent, SpriteComponent)>()
        {
            // Go through each parent and multiply by their transforms
            // TODO: a bit inefficient think about caching?
            let mut global_transform = transform.matrix();
            self.for_each_parent(id, &mut |parent_id| {
                if let Some(transform) = self.world.get::<TransformComponent>(parent_id) {
                    global_transform = transform.matrix() * global_transform;
                }
            });

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

    pub fn set_viewport(&mut self, viewport_size: glam::UVec2) {
        for (_, (camera,)) in self.world.query_mut::<(CameraComponent,)>() {
            camera.viewport_size = viewport_size.as_vec2();
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

        let parent_id = parent_id.unwrap_or(Self::ROOT_ID);
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
            if meta.parent != Self::ROOT_ID {
                self.for_each_parent(meta.parent, func);
                func(meta.parent);
            }
        }
    }

    pub fn root_entity(&self) -> EntityRef {
        self.world
            .entity(Self::ROOT_ID)
            .expect("root entity somehow does not exist")
    }

    pub fn root_meta(&self) -> &mut MetaDataComponent {
        self.world
            .get(Self::ROOT_ID)
            .expect("root entity somehow has no meta")
    }
}
