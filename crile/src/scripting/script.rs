use super::vector::Vector3;
use crate::{scripting::vector::Vector2, EntityId};

pub struct Script {
    bytecode: Vec<u8>,
}

#[derive(Default)]
pub struct ScriptingEngine {
    pub compiler: mlua::Compiler,
    // pub script_instances: hashbrown::HashMap<EntityId, ScriptInstance>,
    pub lua: mlua::Lua,
}

impl ScriptingEngine {
    pub fn run(&mut self, id: EntityId, script: &Script) -> mlua::Result<()> {
        self.lua.load(&script.bytecode).exec()?;

        Ok(())
    }

    pub fn compile(&self, source: &str) -> Script {
        Script {
            bytecode: self.compiler.compile(source),
        }
    }

    pub fn call_signal<'lua>(
        &'lua self,
        full_name: &'static str,
        args: impl mlua::IntoLuaMulti<'lua>,
    ) -> mlua::Result<()> {
        let signal_index: mlua::Table = self.lua.globals().get("__signals_index")?;
        let signal_list: mlua::Table = signal_index.get(full_name)?;

        let args = args.into_lua_multi(&self.lua)?;
        signal_list.for_each(move |_: usize, callback: mlua::Function| {
            callback.call(args.clone())?;
            Ok(())
        })?;

        Ok(())
    }

    pub fn setup(&mut self) -> mlua::Result<()> {
        self.lua = mlua::Lua::default();

        Vector3::register_lua_type(&self.lua)?;
        Vector2::register_lua_type(&self.lua)?;

        self.lua
            .globals()
            .set("__signals_index", self.lua.create_table()?)?;

        let main_events = self.lua.create_table()?;
        main_events.set("Update", self.make_signal("MainEvents.Update")?)?;
        self.lua.globals().set("MainEvents", main_events)?;

        Ok(())
    }

    fn make_signal(&self, full_name: &'static str) -> mlua::Result<mlua::Table> {
        let signal = self.lua.create_table()?;

        let signals_index: mlua::Table = self.lua.globals().get("__signals_index")?;
        signals_index.set(full_name, self.lua.create_table()?)?;

        // TODO: disconnect signal when entity destroy
        let connect_func = move |lua: &mlua::Lua, callback: mlua::Function| {
            let signals_index: mlua::Table = lua.globals().get("__signals_index")?;
            let signal_list: mlua::Table = signals_index.get(full_name)?;
            signal_list.push(callback)?;

            Ok(())
        };
        signal.set("connect", self.lua.create_function(connect_func)?)?;

        Ok(signal)
    }
}

// pub struct ScriptInstance {}

// impl ScriptInstance {
//     fn new(script: &Script) -> mlua::Result<Self> {
//         let lua = mlua::Lua::default();
//         lua.load(&script.bytecode).exec()?;
//         Ok(Self { lua })
//     }

//     fn try_execute(&self, function_name: &str) -> mlua::Result<()> {
//         if let Ok(function) = self.lua.globals().get::<_, mlua::Function>(function_name) {
//             function.call::<(), ()>(())?;
//         }

//         Ok(())
//     }
// }
