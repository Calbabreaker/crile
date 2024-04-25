use crate::{
    CameraComponent, Engine, RenderPass, Scene, ScriptComponent, ScriptingEngine,
    TransformComponent,
};

pub struct SceneRuntime {
    pub scene: Scene,
    scripting: ScriptingEngine,
}

impl SceneRuntime {
    pub fn new(mut scene: Scene, engine: &Engine) -> Self {
        SceneRuntime {
            scripting: ScriptingEngine::new(&mut scene, engine),
            scene,
        }
    }

    pub fn start(&mut self) -> mlua::Result<()> {
        self.scripting.setup()?;

        for (id, (script,)) in self.scene.world.query::<(ScriptComponent,)>() {
            if let Some(script) = &script.script {
                self.scripting.run(id, script)?;
            }
        }

        Ok(())
    }

    pub fn update(&mut self) -> mlua::Result<()> {
        self.scripting.call_signal("MainEvents.Update", ())
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) {
        let world = &self.scene.world;
        if let Some((_, (camera_transform, camera))) = world
            .query::<(TransformComponent, CameraComponent)>()
            .next()
        {
            let view_projection = camera.projection() * camera_transform.matrix().inverse();
            self.scene.render(render_pass, view_projection);
        }
    }

    pub fn stop(&mut self) {}
}
