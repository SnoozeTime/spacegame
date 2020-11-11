use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

pub fn load_config<T, P: AsRef<Path>>(path: P) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|e| e.into())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub thrust: f32,
    pub lateral_thrust: f32,
    pub damping: f32,
    pub rotation_delta: f32,
    pub max_speed: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            thrust: 1000.0,
            lateral_thrust: 600.0,
            damping: 100.0,
            rotation_delta: 0.05,
            max_speed: 200.0,
        }
    }
}

impl PlayerConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<PlayerConfig, Box<dyn Error>> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| e.into())
    }
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GameEngineConfig {
    pub show_gizmos: bool,
}
