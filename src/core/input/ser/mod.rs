use glfw::Key;
use serde_derive::{Deserialize, Serialize};

/// Input keys. Copy of glfw just for serialization.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum BasicKey {
    Space,

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
}

impl Into<Key> for BasicKey {
    fn into(self) -> Key {
        use BasicKey::*;
        match self {
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
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum BasicMouseButton {
    Button1,
    Button2,
    Button3,
}

impl Into<glfw::MouseButton> for BasicMouseButton {
    fn into(self) -> glfw::MouseButton {
        match self {
            Self::Button1 => glfw::MouseButton::Button1,
            Self::Button2 => glfw::MouseButton::Button2,
            Self::Button3 => glfw::MouseButton::Button3,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    Key(BasicKey),
    Mouse(BasicMouseButton),
}
