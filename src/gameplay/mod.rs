use crate::core::input::InputAction;
use glfw::{Key, MouseButton};

pub mod bullet;
pub mod camera;
pub mod collision;
pub mod delete;
pub mod enemy;
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
}

impl InputAction for Action {
    fn from_key(key: Key) -> Option<Action> {
        match key {
            Key::Up => Some(Action::MoveUp),
            Key::W => Some(Action::MoveUp),
            Key::Down => Some(Action::MoveDown),
            Key::S => Some(Action::MoveDown),
            Key::Left => Some(Action::MoveLeft),
            Key::A => Some(Action::MoveLeft),
            Key::Right => Some(Action::MoveRight),
            Key::D => Some(Action::MoveRight),
            Key::Space => Some(Action::Shoot),
            Key::Q => Some(Action::RotateLeft),
            Key::E => Some(Action::RotateRight),
            Key::F => Some(Action::Pickup),
            _ => None,
        }
    }

    fn from_mouse_button(btn: MouseButton) -> Option<Self>
    where
        Self: std::marker::Sized,
    {
        if let MouseButton::Button1 = btn {
            Some(Action::Shoot)
        } else {
            None
        }
    }
}
