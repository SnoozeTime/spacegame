use crate::assets::sprite::SamplerDef;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "packed")]
use crate::assets::sprite::SpriteSyncLoader;

#[derive(Serialize, Deserialize)]
pub struct PackedSpriteAsset {
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
    pub sampler: SamplerDef,
}

#[derive(Serialize, Deserialize)]
pub struct Packed {
    pub content: HashMap<String, PackedSpriteAsset>,
}

#[cfg(feature = "packed")]
const SPRITES: &[u8] = include_bytes!("../../../packed.bin");

#[cfg(feature = "packed")]
pub struct SpritePackLoader {
    packed: Packed,
    fallback_loader: SpriteSyncLoader,
}

#[cfg(feature = "packed")]
mod implementation {
    use super::*;
    use crate::assets::sprite::SpriteAsset;
    use crate::assets::{Asset, AssetError, Loader};
    use crate::render::Context;
    use luminance::context::GraphicsContext;
    use luminance::texture::{GenMipmaps, Texture};
    use std::path::PathBuf;

    impl SpritePackLoader {
        pub fn new(base_path: PathBuf) -> Self {
            Self {
                packed: bincode::deserialize(SPRITES).unwrap(),
                fallback_loader: SpriteSyncLoader::new(base_path),
            }
        }
    }

    impl Loader<SpriteAsset, String> for SpritePackLoader {
        fn load(&mut self, asset_name: String) -> Asset<SpriteAsset> {
            let mut asset = Asset::new();
            if let Some(sprite) = self.packed.content.get(&asset_name) {
                asset.set_loaded(SpriteAsset::Loading(
                    sprite.w,
                    sprite.h,
                    sprite.data.clone(),
                    sprite.sampler.clone().to_sampler(),
                ))
            } else {
                return self.fallback_loader.load(asset_name);
            }

            asset
        }

        fn upload_to_gpu(
            &self,
            ctx: &mut Context,
            inner: &mut SpriteAsset,
        ) -> Result<(), AssetError> {
            let tex = if let SpriteAsset::Loading(w, h, data, sampler) = inner {
                let mut tex = Texture::new(ctx, [*w, *h], 0, sampler.clone())?;
                tex.upload_raw(GenMipmaps::No, data)?;
                tex
            } else {
                panic!("Expecting Loading variant.")
            };

            *inner = SpriteAsset::Uploaded(tex);

            Ok(())
        }
    }
}
