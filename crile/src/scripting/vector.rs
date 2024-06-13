use mlua::UserDataFields;

macro_rules! make_vector_type {
    ($wrapper_type: ident, $inner_type: ty, $number_type: ty, [$($field: ident),*]) => {

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
            fields.add_field_method_set(stringify!($field), |_, this, val| {
                this.0.$field = val;
                Ok(())
            });
        )*
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        macro_rules! add_operator_func {
            ($method: ident, $operator: tt) => {
                methods.add_meta_function(
                    mlua::MetaMethod::$method,
                    |_, (a, b): (Self, Self)| Ok(Self(a.0 $operator b.0)),
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
                if let Ok(value) = ud.borrow::<Vector2>() {
                    return Ok(Self::from(*value));
                }
                if let Ok(value) = ud.borrow::<Vector3>() {
                    return Ok(Self::from(*value));
                }
            }
            mlua::Value::Number(number) => return Ok(Self(<$inner_type>::splat(number as $number_type))),
            mlua::Value::Integer(number) => return Ok(Self(<$inner_type>::splat(number as $number_type))),
            _ => (),
        }

        Err(mlua::Error::RuntimeError(format!("Expected a vector")))
    }
}
    }
}

impl From<Vector2> for Vector3 {
    fn from(value: Vector2) -> Self {
        Self(value.0.extend(0.))
    }
}

impl From<Vector3> for Vector2 {
    fn from(value: Vector3) -> Self {
        Self(value.0.truncate())
    }
}

make_vector_type!(Vector3, glam::Vec3, f32, [x, y, z]);
make_vector_type!(Vector2, glam::Vec2, f32, [x, y]);
