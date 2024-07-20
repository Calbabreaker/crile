use crate::{CameraComponent, RenderPass, ScriptComponent, ScriptingEngine, TransformComponent};

pub struct SceneRunner {
    scripting: ScriptingEngine,
}

impl SceneRunner {
    pub fn new(scripting: ScriptingEngine) -> Self {
        SceneRunner { scripting }
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
        self.scripting.call_signal("MainEvents.Update")
    }

    pub fn fixed_update(&mut self) -> mlua::Result<()> {
        self.scripting.call_signal("MainEvents.FixedUpdate")
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
