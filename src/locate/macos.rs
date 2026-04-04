use std::{env, path::PathBuf};

use crate::{error::LocateError, Error, Result};

pub fn locate_steam_dir_helper() -> Result<PathBuf> {
    // Steam's installation location is pretty easy to find on macOS, as it's always in
    // $USER/Library/Application Support
    let home_dir = env::home_dir().ok_or_else(|| Error::locate(LocateError::no_home()))?;

    // Find Library/Application Support/Steam
    let install_path = home_dir.join("Library/Application Support/Steam");
    Ok(install_path)
}
