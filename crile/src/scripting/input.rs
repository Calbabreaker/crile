use crate::{scripting::vector::Vector2, Input, KeyCode, MouseButton};
use std::str::FromStr;

macro_rules! impl_enum_from_lua_str {
    ($type: ty) => {
        impl mlua::FromLua for $type {
            fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
                let string = value
                    .as_string()
                    .ok_or_else(|| mlua::Error::RuntimeError(format!("Expected a string")))?
                    .to_str()?;

                Self::from_str(&string).map_err(|_| {
                    mlua::Error::RuntimeError(format!(
                        "{} '{}' is not valid.",
                        stringify!($type),
                        string
                    ))
                })
            }
        }
    };
}

impl_enum_from_lua_str!(KeyCode);
impl_enum_from_lua_str!(MouseButton);

pub fn register_class(lua: &mlua::Lua, input: &'static Input) -> mlua::Result<()> {
    let input_class = super::make_class(lua, "Input")?;

    macro_rules! set_input_funcs {
    ($code_type: ty, [$($func_name: ident),*]) => {
        $(
            input_class.set(
                stringify!($func_name),
                lua.create_function(|_, code| {
                    Ok(input.$func_name(code))
                })?,
            )?;
        )*
    };
}
    set_input_funcs!(KeyCode, [key_pressed, key_just_pressed, key_just_released]);
    set_input_funcs!(
        MouseButton,
        [mouse_pressed, mouse_just_pressed, mouse_just_released]
    );

    input_class.set(
        "mouse_position",
        lua.create_function(|_, ()| Ok(Vector2(input.mouse_position())))?,
    )?;

    input_class.set(
        "get_vector",
        lua.create_function(|_, (negative_x, negative_y, positive_x, positive_y)| {
            Ok(Vector2(input.get_vector(
                negative_x, negative_y, positive_x, positive_y,
            )))
        })?,
    )?;

    Ok(())
}
