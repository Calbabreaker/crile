use crate::{
    CameraComponent, Engine, RenderPass, Scene, ScriptComponent, ScriptingEngine,
    TransformComponent,
};

pub struct SceneRunner {
    scripting: ScriptingEngine,
}

impl SceneRunner {
    /// # Safety
    /// TODO: scene needs to live for the duration of ScriptingEngine, perhaps don't use raw ptrs
    pub unsafe fn new(scene: &mut Scene, engine: &Engine) -> Self {
        SceneRunner {
            scripting: ScriptingEngine::new(scene, engine),
        }
    }

    pub fn start(&mut self, scene: &mut Scene) -> mlua::Result<()> {
        self.scripting.setup()?;

        for (id, (script,)) in scene.world.query::<(ScriptComponent,)>() {
            if let Some(script) = &script.script {
                self.scripting.run(id, script)?;
            }
        }

        Ok(())
    }

    pub fn update(&mut self) -> mlua::Result<()> {
        self.scripting.call_signal("MainEvents.Update", ())
    }

    pub fn render(&mut self, render_pass: &mut RenderPass, scene: &mut Scene) {
        if let Some((_, (camera_transform, camera))) = scene
            .world
            .query_mut::<(TransformComponent, CameraComponent)>()
            .next()
        {
            camera.update_projection(camera_transform.matrix());
            let view_projection = camera.view_projection;
            scene.render(render_pass, view_projection);
        }
    }

    pub fn stop(&mut self) {}
}
