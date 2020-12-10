use crate::assets::sprite::{MagFilterDef, MinFilterDef, SamplerDef, SpriteAsset, WrapDef};
use crate::assets::{Asset, AssetError, Loader};
use crate::render::Context;
use luminance_front::texture::{GenMipmaps, Texture};

use image::ImageFormat;
use luminance::shader::UniformType::Sampler1D;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlImageElement, Request, RequestInit, RequestMode, Response};

pub struct AsyncWebLoader;

impl Loader<SpriteAsset> for AsyncWebLoader {
    fn load(&mut self, asset_name: String) -> Asset<SpriteAsset> {
        let asset = Asset::new();

        let fut = load_texels(asset_name, asset.clone());
        wasm_bindgen_futures::spawn_local(fut);

        asset
    }

    fn upload_to_gpu(&self, ctx: &mut Context, inner: &mut SpriteAsset) -> Result<(), AssetError> {
        info!("Will upload to GPU");
        let tex = if let SpriteAsset::Loading(w, h, data, sampler) = inner {
            let mut tex = Texture::new(ctx, [*w, *h], 0, sampler.clone())?;
            tex.upload_raw(GenMipmaps::No, &data)?;
            tex
        } else {
            panic!("Expecting Loading variant.")
        };

        *inner = SpriteAsset::Uploaded(tex);

        Ok(())
    }
}

async fn load_texels(asset_name: String, mut asset: Asset<SpriteAsset>) {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    //
    // let mut json_name = PathBuf::from(&asset_name);
    // json_name.set_extension("json");
    let request =
        Request::new_with_str_and_init(&format!("/assets/sprites/{}", asset_name), &opts).unwrap();

    request.headers().set("Accept", "image/png").unwrap();

    let window = web_sys::window().unwrap();

    if let Ok(resp_value) = JsFuture::from(window.fetch_with_request(&request)).await {
        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        info!("status -> {}", resp.status());

        let blob = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
        info!(
            "IS ArrayBuffer = {:?} for {}",
            js_sys::ArrayBuffer::instanceof(&blob),
            asset_name
        );
        let ab = js_sys::ArrayBuffer::unchecked_from_js(blob);
        let array = js_sys::Uint8Array::new(&ab);
        let data: Vec<u8> = array.to_vec();

        match image::load(std::io::Cursor::new(data), ImageFormat::Png)
            .map(|img| img.flipv().to_rgba8())
        {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let img_data = img.into_raw();
                info!("Finished loading image for {}", asset_name);

                asset.set_loaded(SpriteAsset::Loading(
                    width,
                    height,
                    img_data,
                    SamplerDef {
                        wrap_r: WrapDef::ClampToEdge,
                        wrap_s: WrapDef::ClampToEdge,
                        wrap_t: WrapDef::ClampToEdge,
                        min_filter: MinFilterDef::Nearest,
                        mag_filter: MagFilterDef::Nearest,
                        depth_comparison: None,
                    }
                    .to_sampler(),
                ))
            }
            Err(e) => {
                error!("Error while loading image = {}", e);
                asset.set_error(e.into());
            }
        }
    }
}
