use crate::steamcompat::SteamCompat;
use std::collections::HashMap;
use std::path::Path;

#[derive(Default, Clone, Debug)]
pub(crate) struct SteamCompats {
    pub(crate) tools: HashMap<u32, Option<SteamCompat>>,
}

impl SteamCompats {
    pub(crate) fn discover_tool(&mut self, steamdir: &Path, app_id: u32) {
        self.tools
            .insert(app_id, SteamCompat::new(steamdir, app_id));
    }
}
