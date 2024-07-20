mod components;
mod input;
mod script;
mod time;
mod vector;

pub use script::*;

// Implements mlua::IntoLua and mlua::FromLua for struct with the fields
macro_rules! impl_mlua_conversion {
    ($struct: ty, [$($field: ident),*]) => {
        impl<'lua> mlua::IntoLua<'lua> for $struct {
            fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
                let table = lua.create_table()?;
                $(
                    table.set(stringify!($field), self.$field)?;
                )*
                table.into_lua(lua)
            }
        }

        impl<'lua> mlua::FromLua<'lua> for $struct {
            fn from_lua(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
                let table = mlua::Table::from_lua(value, lua)?;
                Ok(Self {
                    $(
                        $field: table.get(stringify!($field))?,
                    )*
                })
            }
        }
    }
}

pub fn make_class<'a>(lua: &'a mlua::Lua, name: &'static str) -> mlua::Result<mlua::Table<'a>> {
    let class = lua.create_table()?;
    lua.globals().set(name, class)?;
    lua.globals().get(name)
}

pub(crate) use impl_mlua_conversion;
