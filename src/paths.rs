use std::path::PathBuf;

pub fn get_assets_path() -> PathBuf {
    PathBuf::from(std::env::var("ASSET_PATH").unwrap_or("assets/".to_string()))
}
