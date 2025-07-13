use std::path::PathBuf;

use crate::{locate::InstallationType, Result};

pub fn locate_steam_dir_helper() -> Result<Vec<(PathBuf, InstallationType)>> {
    use std::{collections::BTreeSet, env};

    use crate::error::{Error, LocateError};

    // Steam's installation location is pretty easy to find on Linux, too, thanks to the symlink in $USER
    let home_dir = home::home_dir().ok_or_else(|| Error::locate(LocateError::no_home()))?;
    let snap_dir = match env::var("SNAP_USER_DATA") {
        Ok(snap_dir) => PathBuf::from(snap_dir),
        Err(_) => home_dir.join("snap"),
    };

    let mut path_deduper = BTreeSet::new();
    let unique_paths = vec![
        (
            home_dir.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
            InstallationType::LinuxFlatpak,
        ),
        (
            home_dir.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
            InstallationType::LinuxFlatpak,
        ),
        (
            home_dir.join(".var/app/com.valvesoftware.Steam/.steam/root"),
            InstallationType::LinuxFlatpak,
        ),
        (
            home_dir.join(".local/share/Steam"),
            InstallationType::LinuxStandard,
        ),
        (
            home_dir.join(".steam/steam"),
            InstallationType::LinuxStandard,
        ),
        (
            home_dir.join(".steam/root"),
            InstallationType::LinuxStandard,
        ),
        (
            home_dir.join(".steam/debian-installation"),
            InstallationType::LinuxStandard,
        ),
        (
            snap_dir.join("steam/common/.local/share/Steam"),
            InstallationType::LinuxSnap,
        ),
        (
            snap_dir.join("steam/common/.steam/steam"),
            InstallationType::LinuxSnap,
        ),
        (
            snap_dir.join("steam/common/.steam/root"),
            InstallationType::LinuxSnap,
        ),
    ]
    .into_iter()
    .filter(|(path, _)| path.is_dir())
    .filter_map(|(path, installation_type)| {
        let resolved_path = path.read_link().unwrap_or_else(|_| path.clone());
        path_deduper
            .insert(resolved_path.clone())
            .then_some((path, installation_type))
    })
    .collect();
    Ok(unique_paths)
}
