use std::path::PathBuf;

use crate::Result;

pub fn locate_steam_dir() -> Result<PathBuf> {
    locate_steam_dir_helper()
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn locate_steam_dir_helper() -> Result<PathBuf> {
    use crate::error::{Error, LocateError};
    Err(Error::locate(LocateError::Unsupported))
}

#[cfg(target_os = "windows")]
fn locate_steam_dir_helper() -> Result<PathBuf> {
    use crate::error::{Error, LocateError};

    use winreg::{
        enums::{HKEY_LOCAL_MACHINE, KEY_READ},
        RegKey,
    };

    let io_to_locate_err = |io_err| Error::locate(LocateError::winreg(io_err));

    // Locating the Steam installation location is a bit more complicated on Windows

    // Steam's installation location can be found in the registry
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let installation_regkey = hklm
        // 32-bit
        .open_subkey_with_flags("SOFTWARE\\Wow6432Node\\Valve\\Steam", KEY_READ)
        .or_else(|_| {
            // 64-bit
            hklm.open_subkey_with_flags("SOFTWARE\\Valve\\Steam", KEY_READ)
        })
        .map_err(io_to_locate_err)?;

    // The InstallPath key will contain the full path to the Steam directory
    let install_path_str: String = installation_regkey
        .get_value("InstallPath")
        .map_err(io_to_locate_err)?;

    let install_path = PathBuf::from(install_path_str);
    Ok(install_path)
}

#[cfg(target_os = "macos")]
fn locate_steam_dir_helper() -> Result<PathBuf> {
    use crate::{error::LocateError, Error};
    // Steam's installation location is pretty easy to find on macOS, as it's always in
    // $USER/Library/Application Support
    let home_dir = home::home_dir().ok_or_else(|| Error::locate(LocateError::no_home()))?;

    // Find Library/Application Support/Steam
    let install_path = home_dir.join("Library/Application Support/Steam");
    Ok(install_path)
}

#[cfg(target_os = "linux")]
fn locate_steam_dir_helper() -> Result<PathBuf> {
    use std::env;

    use crate::error::{Error, LocateError, ValidationError};

    // Steam's installation location is pretty easy to find on Linux, too, thanks to the symlink in $USER
    let home_dir = home::home_dir().ok_or_else(|| Error::locate(LocateError::no_home()))?;
    let snap_dir = match env::var("SNAP_USER_DATA") {
        Ok(snap_dir) => PathBuf::from(snap_dir),
        Err(_) => home_dir.join("snap"),
    };

    let steam_paths = vec![
        // Flatpak steam install directories
        home_dir.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        home_dir.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
        home_dir.join(".var/app/com.valvesoftware.Steam/.steam/root"),
        // Standard install directories
        home_dir.join(".local/share/Steam"),
        home_dir.join(".steam/steam"),
        home_dir.join(".steam/root"),
        home_dir.join(".steam"),
        // Snap steam install directories
        snap_dir.join("steam/common/.local/share/Steam"),
        snap_dir.join("steam/common/.steam/steam"),
        snap_dir.join("steam/common/.steam/root"),
    ];

    steam_paths
        .into_iter()
        .find(|x| x.is_dir())
        .ok_or_else(|| Error::validation(ValidationError::missing_dir()))
}
