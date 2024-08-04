use std::any::TypeId;

use serde::{de::Error, Deserialize, Serialize};

use crate::{
    last_type_name, with_components, Archetype, Component, EntityRef, HierarchyId, Scene, TypeInfo,
};

#[derive(Default, Deserialize, Serialize)]
struct SerializedScene {
    entity: Vec<toml::Table>,
}

pub struct SceneSerializer;

impl SceneSerializer {
    pub fn serialize(scene: &Scene) -> Result<String, toml::ser::Error> {
        let mut output = SerializedScene::default();

        for index in scene.hierarchy_iter(Scene::ROOT_INDEX) {
            let mut table = toml::Table::new();
            let node = scene.get_node(index).unwrap();
            table.insert("id".to_owned(), toml::Value::Integer(node.id.0 as i64));
            table.insert("name".to_owned(), toml::Value::String(node.name.clone()));

            if index != Scene::ROOT_INDEX {
                table.insert(
                    "parent".to_owned(),
                    toml::Value::Integer(node.parent.0 as i64),
                );
            }

            let entity = scene.world.entity(index).unwrap();
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

            let id = get_value::<u32>(&entity_table, "id")?;
            let name = get_value::<String>(&entity_table, "name")?;

            let index = scene.world.spawn_raw(&type_infos, |index, archetype| {
                for (key, value) in &entity_table {
                    macro_rules! deserialize_components {
                        ( [$($component: ty),*]) => {{
                            $( deserialize_component::<$component>(key, value, archetype, index); )*
                        }};
                    }

                    with_components!(deserialize_components);
                }
            });

            if let Ok(parent_id) = get_value::<u32>(&entity_table, "parent") {
                scene.add_to_hierarchy(name, index, HierarchyId(id), HierarchyId(parent_id));
            } else {
                // Doesn't have a parent then must be the root
                if index != Scene::ROOT_INDEX {
                    return Err(toml::de::Error::custom(format!(
                        "Entity '{name}' listed without parents but was not the first entity",
                    )));
                }

                scene.add_to_hierarchy(name, index, HierarchyId(id), HierarchyId(0));
            }
        }

        if scene.hierarchy_nodes.is_empty() {
            return Err(toml::de::Error::custom("scene was empty"));
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
                false,
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
