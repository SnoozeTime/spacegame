use super::{Asset, AssetError, Loader};

use downcast_rs::__std::path::PathBuf;
use log::{error, info};
use luminance::context::GraphicsContext;
use luminance::depth_test::DepthComparison;
use luminance::pixel::NormRGBA8UI;
use luminance::texture::{Dim2, GenMipmaps, MagFilter, MinFilter, Sampler, Texture, Wrap};
use luminance_gl::GL33;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;

pub enum SpriteAsset<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    Uploaded(Texture<S::Backend, Dim2, NormRGBA8UI>),
    Loading(u32, u32, Vec<u8>, Sampler),
}

impl<S> SpriteAsset<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    pub fn texture(&mut self) -> Option<&mut Texture<S::Backend, Dim2, NormRGBA8UI>> {
        match self {
            SpriteAsset::Loading(_, _, _, _) => None,
            SpriteAsset::Uploaded(tex) => Some(tex),
        }
    }
}

impl<S> Default for SpriteAsset<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    fn default() -> Self {
        SpriteAsset::Loading(0, 0, vec![], Sampler::default())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SpriteAssetMetadata {
    sampler: SamplerDef,
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

impl<S> Loader<S, SpriteAsset<S>> for SpriteSyncLoader
where
    S: GraphicsContext<Backend = GL33>,
{
    fn load(&mut self, asset_name: &str) -> Asset<SpriteAsset<S>> {
        let mut asset = Asset::new();

        let asset_path = self.base_path.join(asset_name);
        // Try to load the metadata first.
        let metadata_path = asset_path.with_extension("json");

        info!(
            "Will load {:?} metadata at {}",
            asset_name,
            metadata_path.display()
        );
        let metadata_str = std::fs::read_to_string(metadata_path);
        if let Err(e) = metadata_str {
            asset.set_error(AssetError::IoError(e));
            error!("No metadata for {}", asset_name);
            return asset;
        }

        let metadata: Result<SpriteAssetMetadata, _> = serde_json::from_str(&metadata_str.unwrap());

        match metadata {
            Ok(metadata) => {
                let sampler = metadata.sampler.to_sampler();
                let (w, h, data) = load_texels(asset_path);
                asset.set_loaded(SpriteAsset::Loading(w, h, data, sampler));
                info!("Finished loading texture");
            }
            Err(e) => {
                error!("Metadata cannot be deserialized from json = {}", e);

                asset.set_error(AssetError::JsonError(e));
            }
        }

        asset
    }

    fn upload_to_gpu(&self, ctx: &mut S, inner: &mut SpriteAsset<S>) {
        let tex = if let SpriteAsset::Loading(w, h, data, sampler) = inner {
            let mut tex = Texture::new(ctx, [*w, *h], 0, sampler.clone())
                .expect("luminance texture creation");
            tex.upload_raw(GenMipmaps::No, data).unwrap();
            tex
        } else {
            panic!("Expecting Loading variant.")
        };

        *inner = SpriteAsset::Uploaded(tex);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SamplerDef {
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

#[derive(Debug, Serialize, Deserialize)]
enum WrapDef {
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

fn load_texels<P: AsRef<Path>>(path: P) -> (u32, u32, Vec<u8>) {
    let img = image::open(path).map(|img| img.flipv().to_rgba()).unwrap();
    let (width, height) = img.dimensions();
    (width, height, img.into_raw())
}
