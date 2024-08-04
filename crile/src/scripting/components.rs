use mlua::IntoLua;

use super::vector::*;
use crate::{
    with_components, CameraComponent, Scene, ScriptComponent, SpriteComponent, TransformComponent,
};

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

impl mlua::UserData for &mut CameraComponent {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("screen_to_world", |_, this, val: Vector2| {
            Ok(Vector2(this.screen_to_world(val.0)))
        });
    }
}

impl mlua::UserData for &mut SpriteComponent {}

impl mlua::UserData for &mut ScriptComponent {}

pub fn register_entity_funcs(lua: &mlua::Lua, scene: &'static Scene) -> mlua::Result<()> {
    // Class to access details about the entity like parent children and components
    lua.globals().set(
        "get_component",
        lua.create_function(|lua, component_name: String| {
            let index = lua.globals().get("entity_index")?;

            macro_rules! match_components {
                ([$($component: ty),*]) => {
                    match component_name.as_str() {
                        $(
                            stringify!($component) => scene
                                .world
                                .get::<$component>(index)
                                .map(|c| c.into_lua(lua)),
                        )*
                        _ => None,
                    }
                };
            }

            let value = with_components!(match_components);
            value.ok_or_else(move || {
                mlua::Error::RuntimeError(format!("\"{component_name}\" does not exist"))
            })?
        })?,
    )?;

    Ok(())
}
