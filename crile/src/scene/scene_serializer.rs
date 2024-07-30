use std::any::TypeId;

use serde::{de::Error, Deserialize, Serialize};

use crate::{
    last_type_name, with_components, Archetype, Component, EntityId, EntityRef, Scene, TypeInfo,
};

#[derive(Default, Deserialize, Serialize)]
struct SerializedScene {
    entity: Vec<toml::Table>,
}

pub struct SceneSerializer;

impl SceneSerializer {
    pub fn serialize(scene: &Scene) -> Result<String, toml::ser::Error> {
        let mut output = SerializedScene::default();

        for (node, id) in scene.iter(Scene::ROOT_ID) {
            let mut table = toml::Table::new();
            table.insert("id".to_owned(), toml::Value::Integer(id as i64));
            table.insert("name".to_owned(), toml::Value::String(node.name.clone()));

            if id != Scene::ROOT_ID {
                table.insert(
                    "parent".to_owned(),
                    toml::Value::Integer(node.parent as i64),
                );
            }

            let entity = scene.world.entity(id).unwrap();
            macro_rules! serialize_components {
                ( [$($component: ty),*]) => {{
                    $( serialize_component::<$component>(&mut table, entity)?; )*
                }};
            }

            with_components!(serialize_components);
            output.entity.push(table);
        }

        toml::to_string(&output)
    }

    pub fn deserialize(source: String) -> Result<Scene, toml::de::Error> {
        let mut scene = Scene::default();
        let output = toml::from_str::<SerializedScene>(&source)?;

        for entity_table in output.entity {
            let mut type_infos = Vec::new();

            for key in entity_table.keys() {
                macro_rules! add_component_types {
                    ( [$($component: ty),*]) => {{
                        $( add_component_type::<$component>(&mut type_infos, key); )*
                    }};
                }

                with_components!(add_component_types);
            }

            type_infos.sort_unstable();

            let id = get_value::<EntityId>(&entity_table, "id")?;
            let name = get_value::<String>(&entity_table, "name")?;

            if id != scene.world.next_free_id() {
                log::warn!(
                    "Entity '{name}' with id {id} skipped an id, some memory maybe be ununsed"
                );
            }

            scene.world.spawn_raw(id, &type_infos, |index, archetype| {
                for (key, value) in &entity_table {
                    macro_rules! deserialize_components {
                        ( [$($component: ty),*]) => {{
                            $( deserialize_component::<$component>(key, value, archetype, index); )*
                        }};
                    }

                    with_components!(deserialize_components);
                }
            });

            if let Ok(parent_id) = get_value::<EntityId>(&entity_table, "parent") {
                scene.add_to_hierachy(name, id, parent_id);
            } else {
                // Doesn't have a parent then must be the root
                if id != Scene::ROOT_ID {
                    return Err(toml::de::Error::custom(format!(
                        "Entity '{name}' listed without parents but was not the root entity (id {})", Scene::ROOT_ID,
                    )));
                }

                scene.add_to_hierachy(name, id, 0);
            }
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

fn add_component_type<T: Component>(type_infos: &mut Vec<TypeInfo>, key: &str) {
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
            .inspect_err(|err| log::error!("Failed to deserialize {type_name}: {err}"))
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

fn get_value<T: for<'a> serde::Deserialize<'a>>(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<T, toml::de::Error> {
    table
        .get(key)
        .cloned()
        .ok_or_else(|| toml::de::Error::custom("No root entity found"))?
        .try_into()
}
