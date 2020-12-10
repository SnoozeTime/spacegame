use crate::assets::{Asset, Loader};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;

pub enum Audio {
    Empty,
    File(Vec<u8>),
}

impl Default for Audio {
    fn default() -> Self {
        Self::Empty
    }
}

pub struct AudioSyncLoader {
    base_path: PathBuf,
}

impl AudioSyncLoader {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        let base_path = base_path.as_ref();
        Self {
            base_path: base_path.to_path_buf(),
        }
    }
}

impl Loader<Audio, String> for AudioSyncLoader {
    fn load(&mut self, asset_name: String) -> Asset<Audio> {
        let mut asset = Asset::new();
        let path = self.base_path.join(asset_name);
        info!("Will load audio at {:?}", path);

        match File::open(path) {
            Ok(mut file) => {
                let mut content = vec![];
                match file.read_to_end(&mut content) {
                    Ok(_) => {
                        info!("Finished loading");
                        asset.set_loaded(Audio::File(content))
                    }
                    Err(e) => {
                        error!("Error while loading file");
                        asset.set_error(e.into())
                    }
                }
            }
            Err(e) => {
                error!("Error while loading file");
                asset.set_error(e.into())
            }
        }

        asset
    }
}
