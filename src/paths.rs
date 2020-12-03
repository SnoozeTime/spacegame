use dirs::data_dir;
use std::path::PathBuf;

pub fn get_assets_path() -> PathBuf {
    PathBuf::from(std::env::var("ASSET_PATH").unwrap_or("assets/".to_string()))
}

pub fn get_save_path() -> PathBuf {
    if let Some(mut save_dir) = data_dir() {
        save_dir.push("everfight");
        if let Err(err) = std::fs::create_dir_all(save_dir.clone()) {
            error!("Failed to create save directory {}: {}", save_dir.to_string_lossy(), err);
        }
        save_dir
    } else {
        get_assets_path()
    }
}
