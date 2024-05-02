use std::path::{Path, PathBuf};

use crate::{RefId, Script, Texture, WgpuContext};

type AssetMap<A> = hashbrown::HashMap<PathBuf, RefId<A>>;

pub trait Asset {
    const PRETTY_NAME: &'static str;
    const FILE_EXTENSIONS: &'static [&'static str];

    fn load(wgpu: &WgpuContext, path: &Path) -> Option<Self>
    where
        Self: std::marker::Sized;

    // We need to function to get the specific map corresponding this this asset type
    fn get_map(manager: &mut AssetManager) -> &mut AssetMap<Self>;
}

impl Asset for Texture {
    const PRETTY_NAME: &'static str = "Image";
    const FILE_EXTENSIONS: &'static [&'static str] = &["png", "jpeg", "jpg"];

    fn load(wgpu: &WgpuContext, path: &Path) -> Option<Self> {
        log::trace!("Loading from {path:?}");
        let image = image::open(path)
            .inspect_err(|err| log::error!("Failed to load image {err}"))
            .ok()?;

        let texture = Texture::from_image(wgpu, image);
        Some(texture)
    }

    fn get_map(manager: &mut AssetManager) -> &mut AssetMap<Self> {
        &mut manager.textures
    }
}

impl Asset for Script {
    const PRETTY_NAME: &'static str = "Lua Script";
    const FILE_EXTENSIONS: &'static [&'static str] = &["lua", "luau"];

    fn load(_: &WgpuContext, path: &Path) -> Option<Self> {
        let compiler = mlua::Compiler::default();
        Some(Script {
            bytecode: compiler.compile(crate::read_file(path)?),
            source: Some(path.to_string_lossy().to_string()),
        })
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
    pub fn load<A: Asset>(&mut self, wgpu: &WgpuContext, path: &Path) -> Option<RefId<A>> {
        let map = A::get_map(self);
        if let Some(asset) = map.get(path) {
            return Some(asset.clone());
        }

        let asset = RefId::new(A::load(wgpu, path)?);
        map.insert(path.to_path_buf(), asset.clone());
        Some(asset)
    }
}
