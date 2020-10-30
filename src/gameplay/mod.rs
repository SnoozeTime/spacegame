use crate::core::input::InputAction;
use glfw::Key;

pub mod bullet;
pub mod collision;
pub mod delete;
pub mod enemy;
pub mod gameover;
pub mod health;
pub mod level;
pub mod player;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Shoot,
}

impl InputAction for Action {
    fn from_key(key: Key) -> Option<Action> {
        match key {
            Key::Up => Some(Action::MoveUp),
            Key::Down => Some(Action::MoveDown),
            Key::Left => Some(Action::MoveLeft),
            Key::Right => Some(Action::MoveRight),
            Key::Space => Some(Action::Shoot),
            _ => None,
        }
    }
}
