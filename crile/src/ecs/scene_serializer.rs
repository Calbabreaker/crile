use serde::{Deserialize, Serialize};

use crate::{
    CameraComponent, EntityId, MetaDataComponent, Scene, SpriteComponent, TransformComponent,
};

#[derive(Default, Deserialize, Serialize)]
struct SerializedEntity {
    id: EntityId,
    #[serde(skip_serializing_if = "Option::is_none")]
    camera: Option<CameraComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sprite: Option<SpriteComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transform: Option<TransformComponent>,
    meta: Option<MetaDataComponent>,
}

#[derive(Default, Deserialize, Serialize)]
struct SerializedScene {
    entities: Vec<SerializedEntity>,
}

pub struct SceneSerializer;

impl SceneSerializer {
    pub fn serialize(scene: &Scene) -> Result<String, serde_yaml::Error> {
        let root_meta = scene.root_meta();
        let mut output = SerializedScene::default();

        scene.for_each_child(root_meta, &mut |id| {
            output.entities.push(SerializedEntity {
                camera: scene.world.get(id).cloned(),
                sprite: scene.world.get(id).cloned(),
                transform: scene.world.get(id).cloned(),
                meta: scene.world.get(id).cloned(),
                id,
            });
        });

        serde_yaml::to_string(&output)
    }

    pub fn deserialize(source: String) -> Result<Scene, serde_yaml::Error> {
        let mut scene = Scene::default();
        let output = serde_yaml::from_str::<SerializedScene>(&source)?;

        for entity in output.entities {
            scene.world.spawn_with_id((()), entity.id);
        }

        Ok(scene)
    }
}
