use super::vector::{Vector2, Vector3};
use crate::{impl_mlua_conversion, Engine, EntityId, KeyCode, Scene, TransformComponent};
use mlua::IntoLua;

pub struct Script {
    pub bytecode: Vec<u8>,
    pub source: Option<String>,
}

pub struct ScriptingEngine {
    pub lua: mlua::Lua,
    // We store raw ptr because the scripts need constant access to Scene
    // Probably better way to do this that is safe
    scene: *mut Scene,
    engine: *const Engine,
}

impl ScriptingEngine {
    pub(crate) unsafe fn new(scene: &mut Scene, engine: &Engine) -> Self {
        Self {
            lua: mlua::Lua::default(),
            scene: scene as *mut Scene,
            engine: engine as *const Engine,
        }
    }

    pub fn setup(&mut self) -> mlua::Result<()> {
        let lua = &self.lua;
        let scene = unsafe { &mut *self.scene };
        let engine = unsafe { &*self.engine };

        Vector3::register_lua_type(lua)?;
        Vector2::register_lua_type(lua)?;

        lua.globals().set("__signals_index", lua.create_table()?)?;

        let main_events = lua.create_table()?;
        main_events.set("Update", self.make_signal("MainEvents.Update")?)?;
        self.lua.globals().set("MainEvents", main_events)?;

        let input_class = lua.create_table()?;
        // input_class.set(
        //     "key_pressed",
        //     lua.create_function(|lua, keycode: String| {
        //         Ok(engine.input.key_pressed(usize as KeyCode))
        //     }),
        // );

        // Class to access details about the entity like parent children and components
        let entity_class = lua.create_table()?;

        entity_class.set(
            "get_component",
            lua.create_function(|lua, component_name: String| {
                let entity: mlua::Table = lua.globals().get("entity")?;
                let id = entity.get("id")?;

                let value = match component_name.as_str() {
                    "TransformComponent" => scene
                        .world
                        .get::<TransformComponent>(id)
                        .map(|c| c.into_lua(lua)),
                    _ => None,
                };

                value.ok_or_else(move || {
                    mlua::Error::RuntimeError(format!("\"{component_name}\" does not exist"))
                })?
            })?,
        )?;

        lua.globals().set("Input", input_class)?;
        lua.globals().set("entity", entity_class)?;

        Ok(())
    }

    pub fn run(&mut self, id: EntityId, script: &Script) -> mlua::Result<()> {
        let entity: mlua::Table = self.lua.globals().get("entity")?;
        entity.set("id", id)?;
        self.lua
            .load(&script.bytecode)
            .set_name(script.source.clone().unwrap_or_default())
            .exec()
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
            entity.set("id", signal.caller_entity_id)?;

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
            signal_list.push(signal)
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
