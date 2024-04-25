use mlua::UserDataFields;

macro_rules! make_vector_type {
    ($wrapper_type: ident, $inner_type: ty, [$($field: ident),*]) => {

/// Math vector used for lua scripting only
/// We need to wrap around the glam::Vec* as rust doesn't allow implementing traits onto it outside of the crate
/// Note: the fields are readonly since setting something like transform.rotation.z += 1 won't do anything
#[derive(Clone, Copy, Debug)]
pub struct $wrapper_type(pub $inner_type);

impl $wrapper_type {
    pub fn register_lua_type(lua: &mlua::Lua) -> mlua::Result<()> {
        let class = lua.create_table()?;
        class.set(
            "new",
            lua.create_function(|_, ($($field,)*)| -> mlua::Result<$wrapper_type> {
                Ok($wrapper_type(<$inner_type>::new($($field,)*)))
            })?,
        )?;

        class.set(":ONE", $wrapper_type(<$inner_type>::ONE))?;
        class.set_readonly(true);
        lua.globals().set(stringify!($wrapper_type), class)?;
        Ok(())
    }
}

impl<'lua> mlua::FromLua<'lua> for $wrapper_type {
    fn from_lua(value: mlua::Value<'lua>, _: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => unreachable!(),
        }
    }
}

impl mlua::UserData for $wrapper_type {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        $(
            fields.add_field_method_get(stringify!($field), |_, this| {
                Ok(this.0.$field)
            });
        )*
    }

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(
            mlua::MetaMethod::Add,
            |_, (vec1, vec2): ($wrapper_type, $wrapper_type)| Ok($wrapper_type(vec1.0 + vec2.0)),
        );
        methods.add_meta_function(
            mlua::MetaMethod::Mul,
            |_, (vec1, vec2): ($wrapper_type, $wrapper_type)| Ok($wrapper_type(vec1.0 * vec2.0)),
        );
        methods.add_meta_function(
            mlua::MetaMethod::Div,
            |_, (vec1, vec2): ($wrapper_type, $wrapper_type)| Ok($wrapper_type(vec1.0 / vec2.0)),
        );
        methods.add_meta_function(
            mlua::MetaMethod::Sub,
            |_, (vec1, vec2): ($wrapper_type, $wrapper_type)| Ok($wrapper_type(vec1.0 - vec2.0)),
        );
    }
}

    }
}

make_vector_type!(Vector3, glam::Vec3, [x, y, z]);
make_vector_type!(Vector2, glam::Vec2, [x, y]);
