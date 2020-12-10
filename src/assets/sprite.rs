use super::{Asset, Loader};

use crate::assets::AssetError;
use image::ImageError;
use log::{error, info};
use luminance::depth_test::DepthComparison;
use luminance::pixel::NormRGBA8UI;
use luminance::texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Wrap};
use luminance_front::texture::Texture;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;

mod packed;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;

use crate::render::Context;
pub use packed::*;

pub enum SpriteAsset {
    Uploaded(Texture<Dim2, NormRGBA8UI>),
    Loading(u32, u32, Vec<u8>, Sampler),
}

impl SpriteAsset {
    pub fn texture(&mut self) -> Option<&mut Texture<Dim2, NormRGBA8UI>> {
        match self {
            SpriteAsset::Loading(_, _, _, _) => None,
            SpriteAsset::Uploaded(tex) => Some(tex),
        }
    }
}

impl Default for SpriteAsset {
    fn default() -> Self {
        SpriteAsset::Loading(0, 0, vec![], Sampler::default())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpriteAssetMetadata {
    pub sampler: SamplerDef,
}

impl Default for SpriteAssetMetadata {
    fn default() -> Self {
        Self {
            sampler: SamplerDef {
                wrap_r: WrapDef::ClampToEdge,
                wrap_s: WrapDef::ClampToEdge,
                wrap_t: WrapDef::ClampToEdge,
                min_filter: MinFilterDef::Nearest,
                mag_filter: MagFilterDef::Linear,
                depth_comparison: None,
            },
        }
    }
}

pub struct SpriteSyncLoader {
    base_path: PathBuf,
}

impl SpriteSyncLoader {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        let base_path = base_path.as_ref();
        Self {
            base_path: base_path.to_path_buf(),
        }
    }
}

impl SpriteSyncLoader {
    fn load_metadata(&self, asset_name: &str) -> SpriteAssetMetadata {
        let metadata_path = self.base_path.join(asset_name).with_extension("json");
        info!(
            "Will load {:?} metadata at {}",
            asset_name,
            metadata_path.display()
        );
        let metadata_str = std::fs::read_to_string(metadata_path);

        match metadata_str {
            Ok(metadata_str) => serde_json::from_str::<SpriteAssetMetadata>(&metadata_str)
                .unwrap_or_else(|e| {
                    error!(
                        "Cannot deserialize Metadata file, will use default instead = {:?}",
                        e
                    );
                    SpriteAssetMetadata::default()
                }),
            Err(_) => {
                info!(
                    "No metadata file for {}, Will use default instead.",
                    asset_name
                );
                SpriteAssetMetadata::default()
            }
        }
    }
}

impl Loader<SpriteAsset> for SpriteSyncLoader {
    fn load(&mut self, asset_name: String) -> Asset<SpriteAsset> {
        let mut asset = Asset::new();
        let asset_path = self.base_path.join(&asset_name);
        let metadata = self.load_metadata(&asset_name);
        let sampler = metadata.sampler.to_sampler();

        match load_texels(asset_path) {
            Ok((w, h, data)) => asset.set_loaded(SpriteAsset::Loading(w, h, data, sampler)),
            Err(e) => {
                error!("Error while loading {} = {}", asset_name, e);
                asset.set_error(e.into());
            }
        }

        info!("Finished loading texture");
        asset
    }

    fn upload_to_gpu(&self, ctx: &mut Context, inner: &mut SpriteAsset) -> Result<(), AssetError> {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SamplerDef {
    /// How should we wrap around the *r* sampling coordinate?
    pub wrap_r: WrapDef,
    /// How should we wrap around the *s* sampling coordinate?
    pub wrap_s: WrapDef,
    /// How should we wrap around the *t* sampling coordinate?
    pub wrap_t: WrapDef,
    /// Minification filter.
    pub min_filter: MinFilterDef,
    /// Magnification filter.
    pub mag_filter: MagFilterDef,

    pub depth_comparison: Option<DepthComparisonDef>,
}

impl SamplerDef {
    fn to_sampler(&self) -> Sampler {
        Sampler {
            depth_comparison: self
                .depth_comparison
                .as_ref()
                .map(|d| d.to_depth_comparison()),
            wrap_r: self.wrap_r.to_wrap(),
            wrap_s: self.wrap_s.to_wrap(),
            wrap_t: self.wrap_t.to_wrap(),
            min_filter: self.min_filter.to_min_filter(),
            mag_filter: self.mag_filter.to_mag_filter(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WrapDef {
    /// If textures coordinates lay outside of *[0;1]*, they will be clamped to either *0* or *1* for
    /// every components.
    ClampToEdge,
    /// Textures coordinates are repeated if they lay outside of *[0;1]*. Picture this as:
    ///
    /// ```ignore
    /// // given the frac function returning the fractional part of a floating number:
    /// coord_ith = frac(coord_ith); // always between [0;1]
    /// ```
    Repeat,
    /// Same as `Repeat` but it will alternatively repeat between *[0;1]* and *[1;0]*.
    MirroredRepeat,
}

impl WrapDef {
    fn to_wrap(&self) -> Wrap {
        match *self {
            Self::ClampToEdge => Wrap::ClampToEdge,
            Self::MirroredRepeat => Wrap::MirroredRepeat,
            Self::Repeat => Wrap::Repeat,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MinFilterDef {
    Nearest,
    Linear,
    NearestMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapNearest,
    LinearMipmapLinear,
}

impl MinFilterDef {
    fn to_min_filter(&self) -> MinFilter {
        match *self {
            Self::Linear => MinFilter::Linear,
            Self::Nearest => MinFilter::Nearest,
            Self::NearestMipmapNearest => MinFilter::NearestMipmapNearest,
            Self::NearestMipmapLinear => MinFilter::NearestMipmapLinear,
            Self::LinearMipmapNearest => MinFilter::LinearMipmapNearest,
            Self::LinearMipmapLinear => MinFilter::LinearMipmapLinear,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MagFilterDef {
    Nearest,
    Linear,
}
impl MagFilterDef {
    fn to_mag_filter(&self) -> MagFilter {
        match *self {
            Self::Linear => MagFilter::Linear,
            Self::Nearest => MagFilter::Nearest,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DepthComparisonDef {
    /// Depth test never succeeds.
    Never,
    /// Depth test always succeeds.
    Always,
    /// Depth test succeeds if `a == b`.
    Equal,
    /// Depth test succeeds if `a != b`.
    NotEqual,
    /// Depth test succeeds if `a < b`.
    Less,
    /// Depth test succeeds if `a <= b`.
    LessOrEqual,
    /// Depth test succeeds if `a > b`.
    Greater,
    /// Depth test succeeds if `a >= b`.
    GreaterOrEqual,
}

impl DepthComparisonDef {
    fn to_depth_comparison(&self) -> DepthComparison {
        match *self {
            Self::Never => DepthComparison::Never,
            Self::Always => DepthComparison::Always,
            Self::Equal => DepthComparison::Equal,
            Self::NotEqual => DepthComparison::NotEqual,
            Self::Less => DepthComparison::Less,
            Self::LessOrEqual => DepthComparison::LessOrEqual,
            Self::Greater => DepthComparison::Greater,
            Self::GreaterOrEqual => DepthComparison::GreaterOrEqual,
        }
    }
}

pub fn load_texels<P: AsRef<Path>>(path: P) -> Result<(u32, u32, Vec<u8>), ImageError> {
    let img = image::open(path).map(|img| img.flipv().to_rgba8())?;
    let (width, height) = img.dimensions();
    Ok((width, height, img.into_raw()))
}
