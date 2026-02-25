use serde::{Deserialize, Serialize};

use crate::keys::Key;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub on: Vec<Key>,
    pub send: Vec<Key>,
}
