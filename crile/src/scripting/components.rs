use super::vector::*;
use crate::TransformComponent;

impl mlua::UserData for TransformComponent {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("translation", |_, this| Ok(Vector3(this.translation)));
        fields.add_field_method_set("translation", |_, this, val: Vector3| {
            this.translation = val.0;
            Ok(())
        });

        fields.add_field_method_get("rotation", |_, this| Ok(Vector3(this.rotation)));
        fields.add_field_method_set("rotation", |_, this, val: Vector3| {
            this.rotation = val.0;
            Ok(())
        });

        fields.add_field_method_get("scale", |_, this| Ok(Vector3(this.scale)));
        fields.add_field_method_set("scale", |_, this, val: Vector3| {
            this.scale = val.0;
            Ok(())
        });
    }
}

fn register_component_types(lua: &mlua::Lua) -> mlua::Result<()> {
    lua.globals().set(
        "get_component",
        lua.create_function(|_, ()| {
            let func = 1;
            Ok(())
        })?,
    )?;

    Ok(())
}
