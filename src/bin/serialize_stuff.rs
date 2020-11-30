#![allow(warnings)]
use spacegame::config::InputConfig;
use spacegame::core::input::ser::{BasicKey, BasicMouseButton, Input};
use spacegame::gameplay::Action;
use std::collections::HashMap;

fn main() {
    let mut input_map = HashMap::new();
    input_map.insert(Action::Shoot, Input::Mouse(BasicMouseButton::Button1));
    input_map.insert(Action::MoveUp, Input::Key(BasicKey::W));
    input_map.insert(Action::Boost, Input::Key(BasicKey::Space));
    input_map.insert(Action::Pickup, Input::Key(BasicKey::F));
    input_map.insert(Action::MoveLeft, Input::Key(BasicKey::A));
    input_map.insert(Action::MoveRight, Input::Key(BasicKey::D));
    input_map.insert(Action::RotateLeft, Input::Key(BasicKey::Q));
    input_map.insert(Action::RotateRight, Input::Key(BasicKey::E));
    let config = InputConfig(input_map);

    let to_str = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write("./assets/config/input.json", to_str.clone()).unwrap();
    println!("{}", to_str);
}
