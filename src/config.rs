use serde::{Deserialize, Serialize};

use crate::keys::Key;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub modes: Vec<Mode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mode {
    #[serde(default = "default_mode_name")]
    pub name: String,
    #[serde(default = "default_modifiers")]
    pub modifiers: Vec<Key>,
    pub mappings: Vec<Mapping>,
}

impl Default for Mode {
    fn default() -> Self {
        Self {
            name: default_mode_name(),
            modifiers: default_modifiers(),
            mappings: Default::default(),
        }
    }
}

fn default_mode_name() -> String {
    "default".into()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub on: Vec<Key>,
    pub send: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Action {
    KeyCombination(Vec<Key>),
    ModeChange(String),
}
