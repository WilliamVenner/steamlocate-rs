use std::{fs, path::Path};

use keyvalues_serde::from_str;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Store {
    #[serde(rename = "Software")]
    pub software: Software,
}

#[derive(Deserialize, Debug)]
pub struct Software {
    #[serde(rename = "Valve")]
    pub valve: Valve,
}

#[derive(Deserialize, Debug)]
pub struct Valve {
    #[serde(rename = "Steam")]
    pub steam: Steam,
}

#[derive(Deserialize, Debug)]
pub struct Steam {
    #[serde(rename = "CompatToolMapping")]
    pub mapping: HashMap<String, SteamCompat>,
}

/// An instance of a compatibility tool.
#[derive(Deserialize, Debug, Clone)]
pub struct SteamCompat {
    /// The name of the tool.
    ///
    /// Example: `proton_411`
    pub name: Option<String>,

    // Unknown option, may be used in the future
    pub config: Option<String>,

    // Unknown option, may be used in the future
    pub priority: Option<u64>,
}

impl SteamCompat {
    pub(crate) fn new(steamdir: &Path, app_id: &u32) -> Option<SteamCompat> {
        let steamdir_config_path = steamdir.join("config").join("config.vdf");

        let vdf_text = fs::read_to_string(steamdir_config_path).ok()?;
        let root: Store = from_str(&vdf_text).unwrap();

        let app_id_str: &str = &app_id.to_string();

        Some(root.software.valve.steam.mapping.get(app_id_str)?.clone())
    }
}
