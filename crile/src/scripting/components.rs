use mlua::IntoLua;

use super::vector::*;
use crate::{Scene, TransformComponent};

impl mlua::UserData for &mut TransformComponent {
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

pub fn register_entity_class(lua: &mlua::Lua, scene: &'static Scene) -> mlua::Result<()> {
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

    lua.globals().set("entity", entity_class)?;

    Ok(())
}
