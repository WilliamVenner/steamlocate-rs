use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Store {
    pub(crate) software: Software,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Software {
    pub(crate) valve: Valve,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Valve {
    pub(crate) steam: Steam,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Steam {
    #[serde(rename = "CompatToolMapping")]
    pub(crate) mapping: HashMap<u32, CompatTool>,
}

/// An instance of a compatibility tool.
#[derive(Deserialize, Debug, Clone)]
pub struct CompatTool {
    /// The name of the tool.
    ///
    /// Example: `proton_411`
    pub name: Option<String>,

    // Unknown option, may be used in the future
    pub config: Option<String>,

    // Unknown option, may be used in the future
    pub priority: Option<u64>,
}
