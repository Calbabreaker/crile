use std::any::TypeId;

use serde::{de::Error, Deserialize, Serialize};

use crate::{with_components, Archetype, Component, EntityRef, MetaDataComponent, Scene, TypeInfo};

#[derive(Default, Deserialize, Serialize)]
struct SerializedScene {
    entities: Vec<toml::Table>,
}

pub struct SceneSerializer;

impl SceneSerializer {
    pub fn serialize(scene: &Scene) -> Result<String, toml::ser::Error> {
        let mut output = SerializedScene::default();

        for (id, (meta,)) in scene.world.query::<(MetaDataComponent,)>() {
            let mut table = toml::Table::new();

            table.insert("id".into(), (id as i64).into());

            table.insert("MetaDataComponent".into(), toml::Value::try_from(meta)?);

            let entity = scene.world.entity(id).unwrap();
            macro_rules! serialize_components {
                ( [$($component: ty),*]) => {{
                    $( serialize_component::<$component>(&mut table, entity)?; )*
                }};
            }

            with_components!(serialize_components);
            output.entities.push(table);
        }

        toml::to_string(&output)
    }

    pub fn deserialize(source: String) -> Result<Scene, toml::de::Error> {
        let mut scene = Scene::empty();
        let output = toml::from_str::<SerializedScene>(&source)?;

        for entity in output.entities {
            if !entity.contains_key("MetaDataComponent") {
                return Err(toml::de::Error::custom(
                    "Entity found with no metadata component",
                ));
            }

            let mut type_infos = Vec::new();

            for key in entity.keys() {
                macro_rules! deserialize_component_types {
                    ( [$($component: ty),*]) => {{
                        $( deserialize_component_type::<$component>(&mut type_infos, key); )*
                    }};
                }

                with_components!(deserialize_component_types);
                deserialize_component_types!([MetaDataComponent]);
            }

            let id = entity
                .get("id")
                .ok_or(toml::de::Error::missing_field("id"))
                .and_then(|v| v.clone().try_into())?;

            type_infos.sort_unstable();

            scene.world.spawn_raw(id, &type_infos, |index, archetype| {
                for (key, value) in &entity {
                    macro_rules! deserialize_components {
                        ( [$($component: ty),*]) => {{
                            $( deserialize_component::<$component>(key, value, archetype, index); )*
                        }};
                    }

                    with_components!(deserialize_components);
                    deserialize_components!([MetaDataComponent])
                }
            });
        }

        if scene.world.entity(Scene::ROOT_ID).is_none() {
            return Err(toml::de::Error::custom("No root entity found"));
        }

        Ok(scene)
    }
}

fn serialize_component<T: Component + serde::Serialize>(
    table: &mut toml::Table,
    entity: EntityRef,
) -> Result<(), toml::ser::Error> {
    let type_name = last_type_name::<T>();
    if let Some(component) = entity.get::<T>() {
        let serialized = toml::Value::try_from(component)?;
        table.insert(type_name.into(), serialized);
    }

    Ok(())
}

fn deserialize_component_type<T: Component>(type_infos: &mut Vec<TypeInfo>, key: &str) {
    let type_name = last_type_name::<T>();
    if key == type_name {
        type_infos.push(TypeInfo::of::<T>());
    }
}

fn deserialize_component<T: Component + for<'a> serde::Deserialize<'a>>(
    key: &str,
    value: &toml::Value,
    archetype: &mut Archetype,
    index: usize,
) {
    let type_name = last_type_name::<T>();
    if key == type_name {
        let component = value
            .clone()
            .try_into()
            .inspect_err(|err| log::error!("Failed to deserialize component {type_name}: {err}"))
            .unwrap_or_default();
        let component = std::mem::ManuallyDrop::new(component);

        unsafe {
            archetype.put_component(
                index,
                &*component as *const T as *const u8,
                TypeId::of::<T>(),
            );
        }
    }
}

fn last_type_name<T: 'static>() -> &'static str {
    let name = std::any::type_name::<T>();
    name.split("::").last().unwrap_or(name)
}
