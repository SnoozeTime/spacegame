use crate::resources::Resources;
use crate::paths::get_assets_path;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedData {
    is_infinite_unlocked: bool,
    wave_record: usize,
}

impl Default for SavedData {
    fn default() -> Self {
        Self {
            is_infinite_unlocked: false,
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

    let base_path = get_assets_path();
    let save_path = base_path.join("data.bin");
    let data = bincode::serialize(&*d)?;
    std::fs::write(save_path, data)?;
    Ok(())
}

pub fn save_unlocked(resources: &Resources) -> Result<(), anyhow::Error> {
    let mut d = resources
        .fetch_mut::<SavedData>()
        .expect("Should have SavedData...");
    d.is_infinite_unlocked = true;

    let base_path = get_assets_path();
    let save_path = base_path.join("data.bin");
    let data = bincode::serialize(&*d)?;
    std::fs::write(save_path, data)?;
    Ok(())
}

pub fn read_saved_data() -> SavedData {
    let base_path = get_assets_path();
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
