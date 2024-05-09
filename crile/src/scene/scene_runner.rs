use crate::{
    CameraComponent, RenderPass, Scene, ScriptComponent, ScriptingEngine, TransformComponent,
    Window,
};

pub struct SceneRunner {
    scripting: ScriptingEngine,
}

impl SceneRunner {
    /// # Safety
    /// scene needs to live for the duration of ScriptingEngine
    /// TODO: perhaps don't use raw ptrs
    pub unsafe fn new(scene: &mut Scene, window: &Window) -> Self {
        SceneRunner {
            scripting: ScriptingEngine::new(scene, window),
        }
    }

    pub fn start(&mut self) -> mlua::Result<()> {
        self.scripting.setup()?;

        let scene = unsafe { &mut *self.scripting.scene };

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

    pub fn fixed_update(&mut self) -> mlua::Result<()> {
        self.scripting.call_signal("MainEvents.FixedUpdate", ())
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) {
        let scene = unsafe { &mut *self.scripting.scene };

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
