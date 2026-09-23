use std::{env, ffi::OsStr, fs, path::Path, path::PathBuf};

use crate::{error::LocateError, Error, Result};

pub fn locate_steam_dir_helper() -> Result<Vec<PathBuf>> {
    let home_dir = env::home_dir().ok_or_else(|| Error::locate(LocateError::no_home()))?;

    let native_install_path = home_dir.join("Library/Application Support/Steam");
    let mut install_paths = Vec::new();
    if native_install_path.join("steamapps").is_dir() {
        install_paths.push(native_install_path);
    }

    let bottles_dir = home_dir.join("Library/Application Support/CrossOver/Bottles");
    install_paths.extend(find_crossover_steam_dirs(&bottles_dir)?);

    Ok(install_paths)
}

fn find_crossover_steam_dirs(bottles_dir: &Path) -> Result<Vec<PathBuf>> {
    let bottle_entries = match fs::read_dir(bottles_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(Error::io(error, bottles_dir)),
    };

    let mut install_paths = Vec::new();
    for bottle_entry in bottle_entries {
        let bottle_path = bottle_entry
            .map_err(|error| Error::io(error, bottles_dir))?
            .path();

        for program_files in ["Program Files (x86)", "Program Files"] {
            let install_path = bottle_path
                .join("drive_c")
                .join(program_files)
                .join("Steam");
            if install_path.join("steamapps").is_dir() {
                install_paths.push(install_path);
            }
        }
    }

    install_paths.sort();
    install_paths.dedup();
    Ok(install_paths)
}

pub(crate) fn resolve_crossover_library_paths(
    steam_dir: &Path,
    library_paths: Vec<PathBuf>,
) -> Vec<PathBuf> {
    let Some(drive_c) = steam_dir
        .ancestors()
        .find(|ancestor| ancestor.file_name() == Some(OsStr::new("drive_c")))
    else {
        return library_paths;
    };
    let Some(bottle_dir) = drive_c.parent() else {
        return library_paths;
    };

    library_paths
        .into_iter()
        .map(|path| {
            let Some(windows_path) = path.to_str() else {
                return path;
            };
            let bytes = windows_path.as_bytes();
            if bytes.len() < 3
                || !bytes[0].is_ascii_alphabetic()
                || bytes[1] != b':'
                || !matches!(bytes[2], b'\\' | b'/')
            {
                return path;
            }

            let drive = bytes[0].to_ascii_lowercase();
            let mut resolved = if drive == b'c' {
                drive_c.to_owned()
            } else {
                bottle_dir
                    .join("dosdevices")
                    .join(format!("{}:", char::from(drive)))
            };
            for component in windows_path[3..]
                .split(['\\', '/'])
                .filter(|component| !component.is_empty())
            {
                resolved.push(component);
            }
            resolved
        })
        .collect()
}
