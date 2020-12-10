use crate::core::input::ser::{Input, VirtualButton, VirtualKey};
use crate::gameplay::Action;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub lateral_thrust: f32,
    pub rotation_delta: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            lateral_thrust: 600.0,
            rotation_delta: 0.05,
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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InputConfig(pub HashMap<Action, Input>);

impl InputConfig {
    pub fn input_maps(self) -> (HashMap<VirtualKey, Action>, HashMap<VirtualButton, Action>) {
        let mut btn_map = HashMap::new();
        let mut key_map = HashMap::new();

        for (action, input) in self.0 {
            match input {
                Input::Key(k) => key_map.insert(k, action),
                Input::Mouse(btn) => btn_map.insert(btn, action),
            };
        }

        (key_map, btn_map)
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct AudioConfig {
    pub background_volume: u32,
    pub effects_volume: u32,
    pub channel_nb: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            background_volume: 100,
            effects_volume: 100,
            channel_nb: 15,
        }
    }
}
