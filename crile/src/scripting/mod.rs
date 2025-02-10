mod components;
mod input;
mod script;
mod time;
mod vector;

pub use script::*;

// Implements mlua::IntoLua and mlua::FromLua for struct with the fields
macro_rules! impl_mlua_conversion {
    ($struct: ty, [$($field: ident),*]) => {
        impl mlua::IntoLua for $struct {
            fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
                let table = lua.create_table()?;
                $(
                    table.set(stringify!($field), self.$field)?;
                )*
                table.into_lua(lua)
            }
        }

        impl mlua::FromLua for $struct {
            fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
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

pub fn make_class(lua: &mlua::Lua, name: &'static str) -> mlua::Result<mlua::Table> {
    let class = lua.create_table()?;
    lua.globals().set(name, class)?;
    lua.globals().get(name)
}

pub(crate) use impl_mlua_conversion;
