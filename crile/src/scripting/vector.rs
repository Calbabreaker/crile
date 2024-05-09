use mlua::UserDataFields;

macro_rules! make_vector_type {
    ($wrapper_type: ident, $inner_type: ty, [$($field: ident),*]) => {

/// Math vector used for lua scripting only
/// We need to wrap around the glam::Vec* as rust doesn't allow implementing traits onto it outside of the crate
/// Note: the fields are readonly since setting something like transform.rotation.z += 1 won't do anything
#[derive(Clone, Copy, Debug)]
pub struct $wrapper_type(pub $inner_type);

impl $wrapper_type {
    pub fn register_class(lua: &mlua::Lua) -> mlua::Result<()> {
        let class = lua.create_table()?;
        class.set(
            "new",
            lua.create_function(|_, ($($field,)*)| -> mlua::Result<$wrapper_type> {
                Ok($wrapper_type(<$inner_type>::new($($field,)*)))
            })?,
        )?;

        class.set("ONE", $wrapper_type(<$inner_type>::ONE))?;
        class.set("ZERO", $wrapper_type(<$inner_type>::ZERO))?;
        class.set_readonly(true);
        lua.globals().set(stringify!($wrapper_type), class)?;
        Ok(())
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

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        macro_rules! add_operator_func {
            ($method: ident, $operator: tt) => {
                methods.add_meta_function(
                    mlua::MetaMethod::$method,
                    |_, (a, b): ($wrapper_type, $wrapper_type)| Ok($wrapper_type(a.0 $operator b.0)),
                );
            }
        }

        add_operator_func!(Mul, *);
        add_operator_func!(Add, +);
        add_operator_func!(Div, /);
        add_operator_func!(Sub, -);
    }
}

impl<'lua> mlua::FromLua<'lua> for $wrapper_type {
    fn from_lua(value: mlua::Value<'lua>, _: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => {
                if let Ok(value) = ud.borrow::<Self>() {
                    return Ok(*value);
                }

                // TODO impletement implicit conveersion
                // if let Ok(value) = ud.borrow::<$other_vector_type>() {
                //     return Ok(Self(<$inner_type>::new($(value.0.$field,)*)));
                // }

                Err(mlua::Error::RuntimeError(format!("Expected a vector")))
            }
            _ => unreachable!(),
        }
    }
}
    }
}

make_vector_type!(Vector3, glam::Vec3, [x, y, z]);
make_vector_type!(Vector2, glam::Vec2, [x, y]);
