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

            for (_, (transform, sprite)) in self
                .world
                .query::<(TransformComponent, SpriteRendererComponent)>()
            {
                self.render_instances.push(RenderInstance {
                    transform: transform.get_matrix(),
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

    pub fn spawn<T: ComponentTuple>(
        &mut self,
        components: T,
        parent_id: Option<EntityId>,
    ) -> EntityId {
        let id = self.world.spawn(components);

        let parent_id = parent_id.unwrap_or(self.root_entity_id);
        let entity = self.world.entity(parent_id);
        let meta = entity.get::<MetaDataComponent>().unwrap();
        meta.children.push(id);

        id
    }

    pub fn root_entity(&self) -> EntityRef {
        self.world.entity(self.root_entity_id)
    }
}
