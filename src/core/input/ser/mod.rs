use serde_derive::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum InputEvent {
    MouseEvent(VirtualButton, VirtualAction),
    KeyEvent(VirtualKey, VirtualAction),
    CursorPos(f64, f64),
    Nothing,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum VirtualAction {
    Pressed,
    Release,
    Repeat,
}

/// Input keys. Copy of glfw just for serialization.

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum VirtualKey {
    Space,
    Escape,

    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Enter,

    Right,
    Left,
    Down,
    Up,

    Unknown,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum VirtualButton {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    Key(VirtualKey),
    Mouse(VirtualButton),
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod glfw_impl {

    use super::*;
    use glfw::{Action, Key, MouseButton, WindowEvent};

    impl From<glfw::WindowEvent> for InputEvent {
        fn from(ev: WindowEvent) -> Self {
            match ev {
                WindowEvent::Key(k, _, a, _) => Self::KeyEvent(k.into(), a.into()),
                WindowEvent::MouseButton(k, a, _) => Self::MouseEvent(k.into(), a.into()),
                WindowEvent::CursorPos(x, y) => Self::CursorPos(x, y),
                _ => Self::Nothing,
            }
        }
    }

    impl From<glfw::Action> for VirtualAction {
        fn from(a: Action) -> Self {
            match a {
                Action::Release => Self::Release,
                Action::Repeat => Self::Repeat,
                Action::Press => Self::Pressed,
            }
        }
    }

    impl From<Key> for VirtualKey {
        fn from(key: Key) -> Self {
            use VirtualKey::*;
            match key {
                Key::Space => Space,
                Key::Num0 => Num0,
                Key::Num1 => Num1,
                Key::Num2 => Num2,
                Key::Num3 => Num3,
                Key::Num4 => Num4,
                Key::Num5 => Num5,
                Key::Num6 => Num6,
                Key::Num7 => Num7,
                Key::Num8 => Num8,
                Key::Num9 => Num9,

                Key::A => A,
                Key::B => B,
                Key::C => C,
                Key::D => D,
                Key::E => E,
                Key::F => F,
                Key::G => G,
                Key::H => H,
                Key::I => I,
                Key::J => J,
                Key::K => K,
                Key::L => L,
                Key::M => M,
                Key::N => N,
                Key::O => O,
                Key::P => P,
                Key::Q => Q,
                Key::R => R,
                Key::S => S,
                Key::T => T,
                Key::U => U,
                Key::V => V,
                Key::W => W,
                Key::X => X,
                Key::Y => Y,
                Key::Z => Z,

                Key::Escape => Escape,
                Key::Enter => Enter,

                Key::Right => Right,
                Key::Left => Left,
                Key::Down => Down,
                Key::Up => Up,
                _ => Unknown,
            }
        }
    }

    impl Into<MouseButton> for VirtualButton {
        fn into(self) -> glfw::MouseButton {
            match self {
                Self::Button1 => MouseButton::Button1,
                Self::Button2 => MouseButton::Button2,
                Self::Button3 => MouseButton::Button3,
                Self::Button4 => MouseButton::Button4,
                Self::Button5 => MouseButton::Button5,
                Self::Button6 => MouseButton::Button6,
                Self::Button7 => MouseButton::Button7,
                Self::Button8 => MouseButton::Button8,
            }
        }
    }

    impl From<MouseButton> for VirtualButton {
        fn from(btn: glfw::MouseButton) -> VirtualButton {
            match btn {
                MouseButton::Button1 => Self::Button1,
                MouseButton::Button2 => Self::Button2,
                MouseButton::Button3 => Self::Button3,
                MouseButton::Button4 => Self::Button4,
                MouseButton::Button5 => Self::Button5,
                MouseButton::Button6 => Self::Button6,
                MouseButton::Button7 => Self::Button5,
                MouseButton::Button8 => Self::Button8,
            }
        }
    }

    impl Into<Key> for VirtualKey {
        fn into(self) -> Key {
            use VirtualKey::*;
            match self {
                Escape => Key::Escape,
                Space => Key::Space,

                Num0 => Key::Num0,
                Num1 => Key::Num1,
                Num2 => Key::Num2,
                Num3 => Key::Num4,
                Num4 => Key::Num4,
                Num5 => Key::Num5,
                Num6 => Key::Num6,
                Num7 => Key::Num7,
                Num8 => Key::Num8,
                Num9 => Key::Num9,

                A => Key::A,
                B => Key::B,
                C => Key::C,
                D => Key::D,
                E => Key::E,
                F => Key::F,
                G => Key::G,
                H => Key::H,
                I => Key::I,
                J => Key::J,
                K => Key::K,
                L => Key::L,
                M => Key::M,
                N => Key::N,
                O => Key::O,
                P => Key::P,
                Q => Key::Q,
                R => Key::R,
                S => Key::S,
                T => Key::T,
                U => Key::U,
                V => Key::V,
                W => Key::W,
                X => Key::X,
                Y => Key::Y,
                Z => Key::Z,

                Enter => Key::Enter,

                Right => Key::Right,
                Left => Key::Left,
                Down => Key::Down,
                Up => Key::Up,
                Unknown => Key::Unknown,
            }
        }
    }
}
