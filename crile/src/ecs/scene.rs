use crate::{
    CameraComponent, DrawUniform, RenderInstance, RenderPass, SpriteRendererComponent,
    TransformComponent, World,
};

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
                transform: view_matrix * camera.camera.get_projection(),
            });
            render_pass.set_shader(render_pass.data.instanced_shader.clone());
            render_pass.draw_mesh_instanced(&render_pass.data.square_mesh, &self.render_instances);
        }
    }

    pub fn resize(&mut self, viewport_size: glam::Vec2) {
        for (camera,) in self.world.query_mut::<(CameraComponent,)>() {
            camera.camera.resize(viewport_size);
        }
    }
}
