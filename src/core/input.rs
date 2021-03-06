use crate::{HEIGHT, WIDTH};
use glfw::{Key, MouseButton, WindowEvent};
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub mod ser;
pub trait InputAction: Hash + Eq + PartialEq + Clone + DeserializeOwned {
    fn get_default_key_mapping() -> HashMap<Key, Self>;
    fn get_default_mouse_mapping() -> HashMap<MouseButton, Self>;
}

pub struct Axis<A>
where
    A: InputAction,
{
    pub left: A,
    pub right: A,
}

#[derive(Debug, Default)]
pub struct Input<A>
where
    A: InputAction,
{
    /// true for pressed
    action_state: HashMap<A, bool>,
    just_pressed: HashSet<A>,

    mouse_pos: glam::Vec2,

    key_mapping: HashMap<Key, A>,
    mouse_mapping: HashMap<MouseButton, A>,
}

impl<A> Input<A>
where
    A: InputAction,
{
    pub fn new(key_mapping: HashMap<Key, A>, mouse_mapping: HashMap<MouseButton, A>) -> Self {
        Self {
            action_state: HashMap::default(),
            just_pressed: HashSet::default(),
            mouse_pos: glam::Vec2::zero(),
            key_mapping,
            mouse_mapping,
        }
    }

    pub fn prepare(&mut self) {
        self.just_pressed.clear();
    }
    pub fn process_event(&mut self, ev: WindowEvent) {
        match ev {
            WindowEvent::Key(key, _, glfw::Action::Press, _) => {
                if let Some(action) = self.key_mapping.get(&key).cloned() {
                    self.action_state.insert(action.clone(), true);
                    self.just_pressed.insert(action);
                }
            }
            WindowEvent::Key(key, _, glfw::Action::Release, _) => {
                if let Some(action) = self.key_mapping.get(&key).cloned() {
                    self.action_state.insert(action, false);
                }
            }
            WindowEvent::MouseButton(btn, glfw::Action::Press, _) => {
                if let Some(action) = self.mouse_mapping.get(&btn).cloned() {
                    self.action_state.insert(action.clone(), true);
                    self.just_pressed.insert(action);
                }
            }
            WindowEvent::MouseButton(btn, glfw::Action::Release, _) => {
                if let Some(action) = self.mouse_mapping.get(&btn).cloned() {
                    self.action_state.insert(action, false);
                }
            }
            WindowEvent::CursorPos(x, y) => self.mouse_pos = glam::vec2(x as f32, y as f32),
            _ => {}
        }
    }

    pub fn get_axis(&self, axis: Axis<A>) -> f32 {
        match (
            self.action_state.get(&axis.left).unwrap_or(&false),
            self.action_state.get(&axis.right).unwrap_or(&false),
        ) {
            (&true, &true) => 1.0,
            (&true, &false) => -1.0,
            (&false, &true) => 1.0,
            (&false, &false) => 0.0,
        }
    }

    pub fn is_just_pressed(&self, action: A) -> bool {
        self.just_pressed.contains(&action)
    }

    pub fn mouse_position(&self) -> glam::Vec2 {
        glam::vec2(
            (self.mouse_pos.x / WIDTH as f32) * 2.0 - 1.0,
            ((HEIGHT as f32 - self.mouse_pos.y) / HEIGHT as f32) * 2.0 - 1.0,
        )
    }
}
