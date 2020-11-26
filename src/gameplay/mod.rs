use crate::core::input::InputAction;
use downcast_rs::__std::collections::hash_map::RandomState;
use glfw::{Key, MouseButton};
use std::collections::HashMap;

pub mod bullet;
pub mod camera;
pub mod collision;
pub mod delete;
pub mod enemy;
pub mod explosion;
pub mod gameover;
pub mod health;
pub mod inventory;
pub mod level;
pub mod physics;
pub mod pickup;
pub mod player;
pub mod steering;
pub mod trail;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Shoot,
    RotateLeft,
    RotateRight,
    Pickup,
    Boost,
}

pub fn get_default_button_mapping() -> HashMap<Key, Action> {
    let mut m = HashMap::new();
    m.insert(Key::Up, Action::MoveUp);
    m.insert(Key::W, Action::MoveUp);
    m.insert(Key::Down, Action::MoveDown);
    m.insert(Key::S, Action::MoveDown);
    m.insert(Key::Left, Action::MoveLeft);
    m.insert(Key::A, Action::MoveLeft);
    m.insert(Key::Right, Action::MoveRight);
    m.insert(Key::D, Action::MoveRight);
    m.insert(Key::Space, Action::Boost);
    m.insert(Key::Q, Action::RotateLeft);
    m.insert(Key::E, Action::RotateRight);
    m.insert(Key::F, Action::Pickup);
    m
}

pub fn get_default_mouse_mapping() -> HashMap<MouseButton, Action> {
    let mut m = HashMap::new();
    m.insert(MouseButton::Button1, Action::Shoot);
    m
}
impl InputAction for Action {
    fn get_default_key_mapping() -> HashMap<Key, Self> {
        get_default_button_mapping()
    }

    fn get_default_mouse_mapping() -> HashMap<MouseButton, Self> {
        get_default_mouse_mapping()
    }
    // fn from_key(key: Key) -> Option<Action> {
    //     match key {
    //         Key::Up => Some(Action::MoveUp),
    //         Key::W => Some(Action::MoveUp),
    //         Key::Down => Some(Action::MoveDown),
    //         Key::S => Some(Action::MoveDown),
    //         Key::Left => Some(Action::MoveLeft),
    //         Key::A => Some(Action::MoveLeft),
    //         Key::Right => Some(Action::MoveRight),
    //         Key::D => Some(Action::MoveRight),
    //         Key::Space => Some(Action::Boost),
    //         Key::Q => Some(Action::RotateLeft),
    //         Key::E => Some(Action::RotateRight),
    //         Key::F => Some(Action::Pickup),
    //         _ => None,
    //     }
    // }

    // fn from_mouse_button(btn: MouseButton) -> Option<Self>
    // where
    //     Self: std::marker::Sized,
    // {
    //     if let MouseButton::Button1 = btn {
    //         Some(Action::Shoot)
    //     } else {
    //         None
    //     }
    // }
}
