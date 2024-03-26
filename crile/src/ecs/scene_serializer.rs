use std::any::TypeId;

use serde::{de::Error, Deserialize, Serialize};

use crate::{with_components, Archetype, EntityRef, MetaDataComponent, Scene, TypeInfo};

#[derive(Default, Deserialize, Serialize)]
struct SerializedScene {
    entities: Vec<serde_yaml::Mapping>,
}

pub struct SceneSerializer;

impl SceneSerializer {
    pub fn serialize(scene: &Scene) -> serde_yaml::Result<String> {
        let mut output = SerializedScene::default();

        for (id, (meta,)) in scene.world.query::<(MetaDataComponent,)>() {
            let mut mapping = serde_yaml::Mapping::new();

            mapping.insert(
                serde_yaml::Value::String("id".into()),
                serde_yaml::Value::Number(id.into()),
            );

            mapping.insert(
                serde_yaml::Value::String("MetaDataComponent".into()),
                serde_yaml::to_value(meta)?,
            );

            let entity = scene.world.entity(id).unwrap();
            macro_rules! serialize_components {
                ( [$($component: ty),*]) => {{
                    $( serialize_component::<$component>(&mut mapping, entity)?; )*
                }};
            }

            with_components!(serialize_components);
            output.entities.push(mapping);
        }

        serde_yaml::to_string(&output)
    }

    pub fn deserialize(source: String) -> serde_yaml::Result<Scene> {
        let mut scene = Scene::empty();
        let output = serde_yaml::from_str::<SerializedScene>(&source)?;

        for entity in output.entities {
            if !entity.contains_key("MetaDataComponent") {
                return Err(serde_yaml::Error::custom(
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

            type_infos.sort_unstable();
            let id = get_with_key(&entity, "id")?;
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
            return Err(serde_yaml::Error::custom("No root entity found"));
        }

        Ok(scene)
    }
}

fn serialize_component<T: 'static + serde::Serialize>(
    mapping: &mut serde_yaml::Mapping,
    entity: EntityRef,
) -> serde_yaml::Result<()> {
    let type_name = last_type_name::<T>();
    if let Some(component) = entity.get::<T>() {
        let serialized = serde_yaml::to_value(component)?;
        mapping.insert(serde_yaml::Value::String(type_name.into()), serialized);
    }

    Ok(())
}

fn deserialize_component_type<T: 'static>(type_infos: &mut Vec<TypeInfo>, key: &serde_yaml::Value) {
    let type_name = last_type_name::<T>();
    if key.as_str() == Some(type_name) {
        type_infos.push(TypeInfo::of::<T>());
    }
}

fn deserialize_component<T: 'static + for<'a> serde::Deserialize<'a> + Default>(
    key: &serde_yaml::Value,
    value: &serde_yaml::Value,
    archetype: &mut Archetype,
    index: usize,
) {
    let type_name = last_type_name::<T>();
    if key.as_str() == Some(type_name) {
        let component = serde_yaml::from_value::<T>(value.clone())
            .inspect_err(|err| log::error!("Failed to deserialize component {type_name}: {err}"))
            .unwrap_or_default();

        unsafe {
            archetype.put_component(
                index,
                &component as *const T as *const u8,
                TypeId::of::<T>(),
            );
            std::mem::forget(component);
        }
    }
}

fn get_with_key<T: for<'a> serde::Deserialize<'a>>(
    map: &serde_yaml::Mapping,
    key: &'static str,
) -> serde_yaml::Result<T> {
    map.get(key)
        .ok_or(serde_yaml::Error::missing_field(key))
        .and_then(|value| serde_yaml::from_value::<T>(value.clone()))
}

fn last_type_name<T: 'static>() -> &'static str {
    let name = std::any::type_name::<T>();
    name.split("::").last().unwrap_or(name)
}
