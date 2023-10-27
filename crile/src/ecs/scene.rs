use super::{CameraComponent, SpriteRendererComponent, TransformComponent, World};
use crate::{DrawUniform, RenderInstance, RenderPass};

#[derive(Default)]
pub struct Scene {
    pub world: World,
    render_instances: Vec<RenderInstance>,
}

impl Scene {
    pub fn render(&mut self, render_pass: &mut RenderPass) {
        if let Some((camera_transform, camera)) = self
            .world
            .query::<(TransformComponent, CameraComponent)>()
            .next()
        {
            self.render_instances.clear();

            for (transform, sprite) in self
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
        for (camera,) in self.world.query_mut::<(CameraComponent,)>() {
            camera.set_viewport(viewport_size);
        }
    }
}
