use std::path::PathBuf;

use crate::Result;

pub fn locate_steam_dir_helper() -> Result<PathBuf> {
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
