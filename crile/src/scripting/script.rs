use super::vector::Vector3;
use crate::{scripting::vector::Vector2, EntityId};

pub struct Script {
    bytecode: Vec<u8>,
}

#[derive(Default)]
pub struct ScriptingEngine {
    pub compiler: mlua::Compiler,
    pub script_instances: hashbrown::HashMap<EntityId, ScriptInstance>,
}

impl ScriptingEngine {
    pub fn create_instance(&mut self, id: EntityId, script: &Script) -> mlua::Result<()> {
        let instance = ScriptInstance::new(script)?;

        Vector3::register_lua_type(&instance.lua)?;
        Vector2::register_lua_type(&instance.lua)?;

        instance.try_execute("setup")?;
        self.script_instances.insert(id, instance);
        Ok(())
    }

    pub fn update_instance(&mut self, id: EntityId) -> mlua::Result<()> {
        if let Some(instance) = self.script_instances.get(&id) {
            instance.try_execute("update")?;
        }

        Ok(())
    }

    pub fn compile(&self, source: &str) -> Script {
        Script {
            bytecode: self.compiler.compile(source),
        }
    }

    pub fn clear(&mut self) {
        self.script_instances.clear();
    }
}

pub struct ScriptInstance {
    lua: mlua::Lua,
}

impl ScriptInstance {
    fn new(script: &Script) -> mlua::Result<Self> {
        let lua = mlua::Lua::default();
        lua.load(&script.bytecode).exec()?;
        Ok(Self { lua })
    }

    fn try_execute(&self, function_name: &str) -> mlua::Result<()> {
        if let Ok(function) = self.lua.globals().get::<_, mlua::Function>(function_name) {
            function.call::<(), ()>(())?;
        }

        Ok(())
    }
}
