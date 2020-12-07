#![allow(warnings)]
use spacegame::config::InputConfig;
use spacegame::core::input::ser::{Input, VirtualButton, VirtualKey};
use spacegame::gameplay::Action;
use std::collections::HashMap;

fn main() {
    let mut input_map = HashMap::new();
    input_map.insert(Action::Shoot, Input::Mouse(VirtualButton::Button1));
    input_map.insert(Action::MoveUp, Input::Key(VirtualKey::W));
    input_map.insert(Action::Boost, Input::Key(VirtualKey::Space));
    input_map.insert(Action::Pickup, Input::Key(VirtualKey::F));
    input_map.insert(Action::MoveLeft, Input::Key(VirtualKey::A));
    input_map.insert(Action::MoveRight, Input::Key(VirtualKey::D));
    input_map.insert(Action::RotateLeft, Input::Key(VirtualKey::Q));
    input_map.insert(Action::RotateRight, Input::Key(VirtualKey::E));
    let config = InputConfig(input_map);

    let to_str = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write("./assets/config/input.json", to_str.clone()).unwrap();
    println!("{}", to_str);
}
