use serde::{Deserialize, Serialize};

use crate::{event::Combination, keys::Key};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    // TODO: don't reuse Combination here, this mixes config and business data structures
    pub on: Combination,
    pub send: Vec<Key>,
}
