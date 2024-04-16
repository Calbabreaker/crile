use super::vector::Vector3;
use crate::{
    impl_mlua_conversion, scripting::vector::Vector2, EntityId, TransformComponent, World,
};

pub struct Script {
    pub bytecode: Vec<u8>,
    pub source: Option<String>,
}

pub struct ScriptingEngine {
    pub lua: mlua::Lua,
    pub active_world: World,
}

impl ScriptingEngine {
    pub fn new() -> Self {
        Self {
            lua: mlua::Lua::default(),
        }
    }

    pub fn setup(&mut self) -> mlua::Result<()> {
        Vector3::register_lua_type(&self.lua)?;
        Vector2::register_lua_type(&self.lua)?;

        self.lua
            .globals()
            .set("__signals_index", self.lua.create_table()?)?;

        let main_events = self.lua.create_table()?;
        main_events.set("Update", self.make_signal("MainEvents.Update")?)?;
        self.lua.globals().set("MainEvents", main_events)?;

        // Class to access details about the entity like parent children and components
        let entity_class = self.lua.create_table()?;

        entity_class.set(
            "get_component",
            self.lua.create_function(|lua, component_name: String| {
                match component_name.as_str() {
                    "TransformComponent" => world.get::<TransformComponent>(entity),

                    _ => Err(mlua::Error::RuntimeError(format!(
                        "Component {component_name} does not exist"
                    ))),
                }
            })?,
        )?;
        self.lua.globals().set("entity", entity_class)?;

        Ok(())
    }

    pub fn run(&mut self, id: EntityId, script: &Script) -> mlua::Result<()> {
        let entity: mlua::Table = self.lua.globals().get("entity")?;
        entity.set("id", id);
        let chunk = self
            .lua
            .load(&script.bytecode)
            .set_name(script.source.clone().unwrap_or_default());

        chunk.exec();

        Ok(())
    }

    pub fn call_signal<'lua>(
        &'lua self,
        full_name: &'static str,
        args: impl mlua::IntoLuaMulti<'lua>,
    ) -> mlua::Result<()> {
        let signal_index: mlua::Table = self.lua.globals().get("__signals_index")?;
        let signal_list: mlua::Table = signal_index.get(full_name)?;

        let args = args.into_lua_multi(&self.lua)?;
        signal_list.for_each(move |_: usize, signal: Signal| {
            let entity: mlua::Table = self.lua.globals().get("entity")?;
            entity.set("id", signal.caller_entity_id);

            signal.callback.call(args.clone())?;
            Ok(())
        })?;

        Ok(())
    }

    fn make_signal(&self, full_name: &'static str) -> mlua::Result<mlua::Table> {
        let signal = self.lua.create_table()?;

        // Create a list for the signal name
        let signals_index: mlua::Table = self.lua.globals().get("__signals_index")?;
        signals_index.set(full_name, self.lua.create_table()?)?;

        // TODO: disconnect signal when entity destroy
        let connect_func = move |lua: &mlua::Lua, callback: mlua::Function| {
            let signals_index: mlua::Table = lua.globals().get("__signals_index")?;
            let signal_list: mlua::Table = signals_index.get(full_name)?;

            let entity: mlua::Table = lua.globals().get("entity")?;

            let signal = Signal {
                callback,
                caller_entity_id: entity.get("id")?,
            };
            signal_list.push(signal)?;

            Ok(())
        };
        signal.set("connect", self.lua.create_function(connect_func)?)?;

        Ok(signal)
    }
}

struct Signal<'lua> {
    callback: mlua::Function<'lua>,
    caller_entity_id: EntityId,
}

impl_mlua_conversion!(Signal::<'lua>, [callback, caller_entity_id]);
