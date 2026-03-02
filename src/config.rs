use serde::{Deserialize, Serialize};

use crate::keys::Key;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_modifiers")]
    pub modifiers: Vec<Key>,
    pub mappings: Vec<Mapping>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            modifiers: default_modifiers(),
            mappings: Default::default(),
        }
    }
}

fn default_modifiers() -> Vec<Key> {
    vec![
        Key::CtrlLeft,
        Key::CtrlRight,
        Key::AltLeft,
        Key::AltRight,
        Key::ShiftLeft,
        Key::ShiftRight,
    ]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub on: Vec<Key>,
    pub send: Vec<Key>,
}
