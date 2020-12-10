use crate::paths::get_save_path;
use crate::resources::Resources;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedData {
    is_infinite_unlocked: bool,
    wave_record: usize,
}

impl Default for SavedData {
    fn default() -> Self {
        Self {
            is_infinite_unlocked: if cfg!(target_arch = "wasm32") {
                true
            } else {
                false
            },
            wave_record: 0,
        }
    }
}

pub fn is_infinite_unlocked(resources: &Resources) -> bool {
    let d = resources
        .fetch::<SavedData>()
        .expect("Should have SavedData...");
    d.is_infinite_unlocked
}

pub fn get_wave_record(resources: &Resources) -> usize {
    let d = resources
        .fetch::<SavedData>()
        .expect("Should have SavedData...");
    d.wave_record
}

pub fn save_new_wave_record(resources: &Resources, new_record: usize) -> Result<(), anyhow::Error> {
    let mut d = resources
        .fetch_mut::<SavedData>()
        .expect("Should have SavedData...");

    if new_record > d.wave_record {
        d.wave_record = new_record;
    }
    save_data(&*d)?;
    Ok(())
}

pub fn save_unlocked(resources: &Resources) -> Result<(), anyhow::Error> {
    let mut d = resources
        .fetch_mut::<SavedData>()
        .expect("Should have SavedData...");
    d.is_infinite_unlocked = true;
    save_data(&*d)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_saved_data() -> SavedData {
    let base_path = get_save_path();
    let save_path = base_path.join("data.bin");

    if let Ok(data) = std::fs::read(&save_path) {
        if let Ok(saved) = bincode::deserialize(&data) {
            return saved;
        }
    }

    let d = SavedData::default();
    if let Err(e) = std::fs::write(save_path, bincode::serialize(&d).expect("Error here...")) {
        error!("Cannot save game data = {:?}", e);
    }
    d
}

#[cfg(target_arch = "wasm32")]
pub fn read_saved_data() -> SavedData {
    let window = web_sys::window().unwrap();
    let local_storage = window
        .local_storage()
        .expect("Should have access to local storage")
        .unwrap();

    if let Ok(Some(data)) = local_storage.get_item("saved_data") {
        if let Ok(saved) = serde_json::from_str(&data) {
            return saved;
        }
    }

    let d = SavedData::default();
    local_storage
        .set_item("saved_data", &serde_json::to_string(&d).unwrap())
        .unwrap();
    d
}

#[cfg(not(target_arch = "wasm32"))]
fn save_data(d: &SavedData) -> Result<(), anyhow::Error> {
    let base_path = get_save_path();
    let save_path = base_path.join("data.bin");
    let data = bincode::serialize(d)?;
    std::fs::write(save_path, data)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn save_data(d: &SavedData) -> Result<(), anyhow::Error> {
    let window = web_sys::window().unwrap();
    let local_storage = window
        .local_storage()
        .expect("Should have access to local storage")
        .unwrap();

    local_storage
        .set_item("saved_data", &serde_json::to_string(d).unwrap())
        .unwrap();
    Ok(())
}
