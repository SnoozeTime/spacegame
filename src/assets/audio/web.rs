use crate::assets::audio::Audio;
use crate::assets::{Asset, Loader};
use serde::de::DeserializeOwned;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlImageElement, Request, RequestInit, RequestMode, Response};

pub struct AsyncWebLoader {
    endpoint: String,
}

impl AsyncWebLoader {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl Loader<Audio> for AsyncWebLoader {
    fn load(&mut self, asset_name: String) -> Asset<Audio> {
        let asset = Asset::new();
        let fut = load_audio(self.endpoint.clone(), asset_name, asset.clone());
        wasm_bindgen_futures::spawn_local(fut);
        asset
    }
}

async fn load_audio(endpoint: String, asset_name: String, mut asset: Asset<Audio>) {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    //
    let mut json_name = PathBuf::from(&endpoint).join(&asset_name);
    let request =
        Request::new_with_str_and_init(&format!("{}", json_name.display()), &opts).unwrap();

    request
        .headers()
        .set("Accept", "audio/vnd.wav, audio/ogg, 	audio/vorbis")
        .unwrap();

    let window = web_sys::window().unwrap();

    if let Ok(resp_value) = JsFuture::from(window.fetch_with_request(&request)).await {
        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        match JsFuture::from(resp.array_buffer().unwrap()).await {
            Ok(ab) => {
                let ab: js_sys::ArrayBuffer = ab.dyn_into().unwrap();
                let array = js_sys::Uint8Array::new(&ab);

                let data: Vec<u8> = array.to_vec();

                // Use serde to parse the JSON into a struct.
                asset.set_loaded(Audio::File(data))
            }
            Err(e) => {
                error!("Error while loading prefab = {:?}", e);
            }
        }
    }
}
