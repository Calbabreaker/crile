use crate::{RefId, Texture, WGPUContext};

#[derive(Default)]
pub struct AssetLibrary {
    textures: hashbrown::HashMap<std::path::PathBuf, RefId<Texture>>,
}

impl AssetLibrary {
    pub fn load_texture(
        &mut self,
        wgpu: &WGPUContext,
        path: &std::path::Path,
    ) -> Option<RefId<Texture>> {
        log::info!("Loading {path:?}");
        if let Some(texture) = self.textures.get(path) {
            return Some(texture.clone());
        }

        match image::open(path) {
            Ok(image) => {
                let texture = RefId::new(Texture::from_image(wgpu, image));
                self.textures.insert(path.to_path_buf(), texture.clone());
                Some(texture)
            }
            Err(err) => {
                log::error!("Failed to load image {err}");
                None
            }
        }
    }
}
