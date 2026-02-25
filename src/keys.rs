use input_event_codes::{
    KEY_0, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9, KEY_A, KEY_B,
    KEY_BACKSPACE, KEY_C, KEY_CAPSLOCK, KEY_D, KEY_DELETE, KEY_DOWN, KEY_E, KEY_END, KEY_F, KEY_G,
    KEY_H, KEY_HOME, KEY_I, KEY_J, KEY_K, KEY_KP0, KEY_KP1, KEY_KP2, KEY_KP3, KEY_KP4, KEY_KP5,
    KEY_KP6, KEY_KP7, KEY_KP8, KEY_KP9, KEY_L, KEY_LEFT, KEY_LEFTALT, KEY_LEFTCTRL, KEY_LEFTSHIFT,
    KEY_M, KEY_N, KEY_O, KEY_P, KEY_Q, KEY_R, KEY_RIGHT, KEY_RIGHTALT, KEY_RIGHTCTRL,
    KEY_RIGHTSHIFT, KEY_S, KEY_SEMICOLON, KEY_T, KEY_U, KEY_UP, KEY_V, KEY_W, KEY_X, KEY_Y, KEY_Z,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    #[serde(rename = "down")]
    ArrowDown,
    #[serde(rename = "left")]
    ArrowLeft,
    #[serde(rename = "right")]
    ArrowRight,
    #[serde(rename = "up")]
    ArrowUp,

    Home,
    End,

    Capslock,

    CtrlLeft,
    CtrlRight,
    ShiftLeft,
    ShiftRight,
    AltLeft,
    AltRight,

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

    #[serde(rename = "0")]
    Num0,
    #[serde(rename = "1")]
    Num1,
    #[serde(rename = "2")]
    Num2,
    #[serde(rename = "3")]
    Num3,
    #[serde(rename = "4")]
    Num4,
    #[serde(rename = "5")]
    Num5,
    #[serde(rename = "6")]
    Num6,
    #[serde(rename = "7")]
    Num7,
    #[serde(rename = "8")]
    Num8,
    #[serde(rename = "9")]
    Num9,

    NumPad0,
    NumPad1,
    NumPad2,
    NumPad3,
    NumPad4,
    NumPad5,
    NumPad6,
    NumPad7,
    NumPad8,
    NumPad9,

    #[serde(rename = ";")]
    Semicolon,
    Backspace,
    Delete,

    Other(u16),
}

impl From<u16> for Key {
    fn from(value: u16) -> Self {
        match value {
            KEY_DOWN!() => Self::ArrowDown,
            KEY_LEFT!() => Self::ArrowLeft,
            KEY_RIGHT!() => Self::ArrowRight,
            KEY_UP!() => Self::ArrowUp,

            KEY_HOME!() => Self::Home,
            KEY_END!() => Self::End,

            KEY_CAPSLOCK!() => Self::Capslock,

            KEY_LEFTCTRL!() => Self::CtrlLeft,
            KEY_RIGHTCTRL!() => Self::CtrlRight,
            KEY_LEFTSHIFT!() => Self::ShiftLeft,
            KEY_RIGHTSHIFT!() => Self::ShiftRight,
            KEY_LEFTALT!() => Self::AltLeft,
            KEY_RIGHTALT!() => Self::AltRight,

            KEY_A!() => Self::A,
            KEY_B!() => Self::B,
            KEY_C!() => Self::C,
            KEY_D!() => Self::D,
            KEY_E!() => Self::E,
            KEY_F!() => Self::F,
            KEY_G!() => Self::G,
            KEY_H!() => Self::H,
            KEY_I!() => Self::I,
            KEY_J!() => Self::J,
            KEY_K!() => Self::K,
            KEY_L!() => Self::L,
            KEY_M!() => Self::M,
            KEY_N!() => Self::N,
            KEY_O!() => Self::O,
            KEY_P!() => Self::P,
            KEY_Q!() => Self::Q,
            KEY_R!() => Self::R,
            KEY_S!() => Self::S,
            KEY_T!() => Self::T,
            KEY_U!() => Self::U,
            KEY_V!() => Self::V,
            KEY_W!() => Self::W,
            KEY_X!() => Self::X,
            KEY_Y!() => Self::Y,
            KEY_Z!() => Self::Z,

            KEY_0!() => Self::Num0,
            KEY_1!() => Self::Num1,
            KEY_2!() => Self::Num2,
            KEY_3!() => Self::Num3,
            KEY_4!() => Self::Num4,
            KEY_5!() => Self::Num5,
            KEY_6!() => Self::Num6,
            KEY_7!() => Self::Num7,
            KEY_8!() => Self::Num8,
            KEY_9!() => Self::Num9,
            KEY_KP0!() => Self::NumPad0,
            KEY_KP1!() => Self::NumPad1,
            KEY_KP2!() => Self::NumPad2,
            KEY_KP3!() => Self::NumPad3,
            KEY_KP4!() => Self::NumPad4,
            KEY_KP5!() => Self::NumPad5,
            KEY_KP6!() => Self::NumPad6,
            KEY_KP7!() => Self::NumPad7,
            KEY_KP8!() => Self::NumPad8,
            KEY_KP9!() => Self::NumPad9,

            KEY_SEMICOLON!() => Self::Semicolon,
            KEY_BACKSPACE!() => Self::Backspace,
            KEY_DELETE!() => Self::Delete,

            code => Self::Other(code),
        }
    }
}

impl From<Key> for u16 {
    fn from(value: Key) -> Self {
        match value {
            Key::ArrowDown => KEY_DOWN!(),
            Key::ArrowLeft => KEY_LEFT!(),
            Key::ArrowRight => KEY_RIGHT!(),
            Key::ArrowUp => KEY_UP!(),

            Key::Home => KEY_HOME!(),
            Key::End => KEY_END!(),

            Key::Capslock => KEY_CAPSLOCK!(),

            Key::CtrlLeft => KEY_LEFTCTRL!(),
            Key::CtrlRight => KEY_RIGHTCTRL!(),
            Key::ShiftLeft => KEY_LEFTSHIFT!(),
            Key::ShiftRight => KEY_RIGHTSHIFT!(),
            Key::AltLeft => KEY_LEFTALT!(),
            Key::AltRight => KEY_RIGHTALT!(),

            Key::A => KEY_A!(),
            Key::B => KEY_B!(),
            Key::C => KEY_C!(),
            Key::D => KEY_D!(),
            Key::E => KEY_E!(),
            Key::F => KEY_F!(),
            Key::G => KEY_G!(),
            Key::H => KEY_H!(),
            Key::I => KEY_I!(),
            Key::J => KEY_J!(),
            Key::K => KEY_K!(),
            Key::L => KEY_L!(),
            Key::M => KEY_M!(),
            Key::N => KEY_N!(),
            Key::O => KEY_O!(),
            Key::P => KEY_P!(),
            Key::Q => KEY_Q!(),
            Key::R => KEY_R!(),
            Key::S => KEY_S!(),
            Key::T => KEY_T!(),
            Key::U => KEY_U!(),
            Key::V => KEY_V!(),
            Key::W => KEY_W!(),
            Key::X => KEY_X!(),
            Key::Y => KEY_Y!(),
            Key::Z => KEY_Z!(),
            Key::Num0 => KEY_0!(),
            Key::Num1 => KEY_1!(),
            Key::Num2 => KEY_2!(),
            Key::Num3 => KEY_3!(),
            Key::Num4 => KEY_4!(),
            Key::Num5 => KEY_5!(),
            Key::Num6 => KEY_6!(),
            Key::Num7 => KEY_7!(),
            Key::Num8 => KEY_8!(),
            Key::Num9 => KEY_9!(),
            Key::NumPad0 => KEY_KP0!(),
            Key::NumPad1 => KEY_KP1!(),
            Key::NumPad2 => KEY_KP2!(),
            Key::NumPad3 => KEY_KP3!(),
            Key::NumPad4 => KEY_KP4!(),
            Key::NumPad5 => KEY_KP5!(),
            Key::NumPad6 => KEY_KP6!(),
            Key::NumPad7 => KEY_KP7!(),
            Key::NumPad8 => KEY_KP8!(),
            Key::NumPad9 => KEY_KP9!(),

            Key::Semicolon => KEY_SEMICOLON!(),
            Key::Backspace => KEY_BACKSPACE!(),
            Key::Delete => KEY_DELETE!(),

            Key::Other(code) => code,
        }
    }
}
