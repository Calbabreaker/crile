use crate::Time;

pub fn register_class(lua: &mlua::Lua, time: &'static Time) -> mlua::Result<()> {
    let time_class = super::make_class(lua, "Time")?;

    time_class.set(
        "delta",
        lua.create_function(|_, ()| Ok(time.delta().as_secs_f32()))?,
    )?;

    Ok(())
}
