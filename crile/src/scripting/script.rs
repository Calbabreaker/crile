use crate::{impl_mlua_conversion, Engine, Scene, WindowId};

pub struct Script {
    pub bytecode: Vec<u8>,
    pub source: Option<String>,
}

pub struct ScriptingEngine {
    pub lua: mlua::Lua,
    // We store raw ptr because the scripts need constant access to Scene
    // Probably better way to do this that is safe
    pub(crate) scene: *mut Scene,
    engine: *const Engine,
    window_id: WindowId,
}

impl ScriptingEngine {
    /// # Safety
    /// scene and engine needs to live for the duration of ScriptingEngine
    /// TODO: perhaps don't use raw ptrs
    pub unsafe fn new(scene: &mut Scene, engine: &Engine, window_id: WindowId) -> Self {
        Self {
            lua: mlua::Lua::default(),
            scene: scene as *mut Scene,
            engine: engine as *const Engine,
            window_id,
        }
    }

    pub fn setup(&mut self) -> mlua::Result<()> {
        let lua = &self.lua;
        let scene = unsafe { &mut *self.scene };
        let engine = unsafe { &*self.engine };
        let window = engine
            .get_window(self.window_id)
            .ok_or(mlua::Error::runtime("Window ID invalid"))?;

        super::vector::Vector3::register_class(lua)?;
        super::vector::Vector2::register_class(lua)?;
        super::input::register_class(lua, &window.input)?;
        super::time::register_class(lua, &engine.time)?;
        super::components::register_entity_funcs(lua, scene)?;

        lua.globals().set("__signals_index", lua.create_table()?)?;

        let main_events = lua.create_table()?;
        main_events.set("Update", self.make_signal("MainEvents.Update")?)?;
        main_events.set("FixedUpdate", self.make_signal("MainEvents.FixedUpdate")?)?;
        self.lua.globals().set("MainEvents", main_events)?;

        Ok(())
    }

    pub fn run(&mut self, entity_index: usize, script: &Script) -> mlua::Result<()> {
        self.lua.globals().set("entity_index", entity_index)?;
        self.lua
            .load(&script.bytecode)
            .set_name(script.source.clone().unwrap_or_default())
            .exec()
    }

    pub fn call_signal(&self, full_name: &'static str) -> mlua::Result<()> {
        let signal_index: mlua::Table = self.lua.globals().get("__signals_index")?;
        let signal_list: mlua::Table = signal_index.get(full_name)?;

        signal_list.for_each(move |_: usize, signal: Signal| {
            self.lua
                .globals()
                .set("entity_index", signal.caller_entity_index)?;

            signal.callback.call::<_, ()>(())?;
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

            let signal_connect_info = Signal {
                callback,
                caller_entity_index: lua.globals().get("entity_index")?,
            };
            signal_list.push(signal_connect_info)
        };
        signal.set("connect", self.lua.create_function(connect_func)?)?;

        Ok(signal)
    }
}

struct Signal<'lua> {
    callback: mlua::Function<'lua>,
    caller_entity_index: usize,
}

impl_mlua_conversion!(Signal::<'lua>, [callback, caller_entity_index]);
