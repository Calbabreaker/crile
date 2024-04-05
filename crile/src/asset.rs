use std::path::{Path, PathBuf};

use crate::{Engine, RefId, Script, Texture};

type AssetMap<A> = hashbrown::HashMap<PathBuf, RefId<A>>;

pub trait Asset {
    const PICKER_NAME: &'static str;
    const FILE_EXTENSIONS: &'static [&'static str];

    fn load(engine: &Engine, path: &Path) -> Option<Self>
    where
        Self: std::marker::Sized;

    // We need to function to get the specific map corresponding this this asset type
    fn get_map(manager: &mut AssetManager) -> &mut AssetMap<Self>;
}

impl Asset for Texture {
    const PICKER_NAME: &'static str = "Image";
    const FILE_EXTENSIONS: &'static [&'static str] = &["png", "jpeg", "jpg"];

    fn load(engine: &Engine, path: &Path) -> Option<Self> {
        let image = image::open(path)
            .inspect_err(|err| log::error!("Failed to load image {err}"))
            .ok()?;
        let texture = Texture::from_image(&engine.gfx.wgpu, image);
        Some(texture)
    }

    fn get_map(manager: &mut AssetManager) -> &mut AssetMap<Self> {
        &mut manager.textures
    }
}

impl Asset for Script {
    const PICKER_NAME: &'static str = "Lua Script";
    const FILE_EXTENSIONS: &'static [&'static str] = &["lua", "luau"];

    fn load(engine: &Engine, path: &Path) -> Option<Self> {
        Some(engine.scripting.compile(&crate::read_file(path)?))
    }

    fn get_map(manager: &mut AssetManager) -> &mut AssetMap<Self> {
        &mut manager.scripts
    }
}

#[derive(Default)]
pub struct AssetManager {
    textures: AssetMap<Texture>,
    scripts: AssetMap<Script>,
}

impl AssetManager {
    pub fn load<A: Asset>(&mut self, engine: &Engine, path: &Path) -> Option<RefId<A>> {
        let map = A::get_map(self);
        if let Some(asset) = map.get(path) {
            return Some(asset.clone());
        }

        log::info!("Loading {path:?}");
        let asset = RefId::new(A::load(engine, path)?);
        map.insert(path.to_path_buf(), asset.clone());
        Some(asset)
    }
}
