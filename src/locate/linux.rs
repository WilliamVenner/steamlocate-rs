use std::path::PathBuf;

use crate::Result;

pub fn locate_steam_dir_helper() -> Result<Vec<PathBuf>> {
    use std::{collections::BTreeSet, env};

    use crate::error::{Error, LocateError};

    // Steam's installation location is pretty easy to find on Linux, too, thanks to the symlink in $USER
    let home_dir = home::home_dir().ok_or_else(|| Error::locate(LocateError::no_home()))?;
    let snap_dir = match env::var("SNAP_USER_DATA") {
        Ok(snap_dir) => PathBuf::from(snap_dir),
        Err(_) => home_dir.join("snap"),
    };

    let mut path_deduper = BTreeSet::new();
    let unique_paths = [
        // Flatpak steam install directories
        home_dir.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        home_dir.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
        home_dir.join(".var/app/com.valvesoftware.Steam/.steam/root"),
        // Standard install directories
        home_dir.join(".local/share/Steam"),
        home_dir.join(".steam/steam"),
        home_dir.join(".steam/root"),
        home_dir.join(".steam/debian-installation"),
        // Snap steam install directories
        snap_dir.join("steam/common/.local/share/Steam"),
        snap_dir.join("steam/common/.steam/steam"),
        snap_dir.join("steam/common/.steam/root"),
    ]
    .into_iter()
    .filter(|path| path.is_dir())
    .filter_map(|path| {
        let resolved_path = path.read_link().unwrap_or_else(|_| path.clone());
        path_deduper.insert(resolved_path.clone()).then_some(path)
    })
    .collect();
    Ok(unique_paths)
}
