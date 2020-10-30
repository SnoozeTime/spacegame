use log::debug;
use luminance::backend::texture::Texture as TextureBackend;
use luminance::context::GraphicsContext;
use luminance::pixel::NormRGBA8UI;
use luminance::texture::{Dim2, GenMipmaps, Sampler, Texture};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct SpriteCache<B>
where
    B: GraphicsContext + 'static,
    B::Backend: luminance::backend::texture::Texture<
        luminance::texture::Dim2,
        luminance::pixel::NormRGBA8UI,
    >,
{
    pub inner: HashMap<String, Texture<<B as GraphicsContext>::Backend, Dim2, NormRGBA8UI>>,
}

impl<B> Default for SpriteCache<B>
where
    B: GraphicsContext,
    B::Backend: luminance::backend::texture::Texture<
        luminance::texture::Dim2,
        luminance::pixel::NormRGBA8UI,
    >,
{
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

pub fn load_sprites<B>(surface: &mut B) -> SpriteCache<B>
where
    B: GraphicsContext,
    B::Backend: TextureBackend<Dim2, NormRGBA8UI>,
{
    let mut sprite_cache = SpriteCache::default();

    let base_path = std::env::var("ASSET_PATH").unwrap_or("".to_string());
    for n in &[
        "Enemy2",
        "Enemy3",
        "round_bullet",
        "fast_bullet",
        "round_bullet_2",
        "Proto-ship",
        "P-blue-a",
        "EnemyBoss2",
    ] {
        let path = PathBuf::from(&base_path).join(format!("assets/sprites/{}.png", n));
        debug!("Will load from {:?}", path);
        sprite_cache
            .inner
            .insert(n.to_lowercase(), load_from_disk(surface, path));
    }

    sprite_cache
}

fn load_from_disk<B, P: AsRef<Path>>(
    surface: &mut B,
    path: P,
) -> Texture<B::Backend, Dim2, NormRGBA8UI>
where
    B: GraphicsContext,
    B::Backend: TextureBackend<Dim2, NormRGBA8UI>,
{
    let img = image::open(path).map(|img| img.flipv().to_rgba()).unwrap();
    let (width, height) = img.dimensions();
    let texels = img.into_raw();

    // create the luminance texture; the third argument is the number of mipmaps we want (leave it
    // to 0 for now) and the latest is the sampler to use when sampling the texels in the
    // shader (we’ll just use the default one)
    let mut tex = Texture::new(surface, [width, height], 0, Sampler::default())
        .expect("luminance texture creation");

    // the first argument disables mipmap generation (we don’t care so far)
    tex.upload_raw(GenMipmaps::No, &texels).unwrap();

    tex
}
