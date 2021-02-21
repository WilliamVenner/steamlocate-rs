#[macro_use] extern crate lazy_static;

use std::{collections::HashMap, path::PathBuf};

#[cfg(target_os="windows")]
use winreg::{RegKey, enums::{HKEY_LOCAL_MACHINE, KEY_READ}};
#[cfg(not(target_os="windows"))]
extern crate dirs;

mod steamapp;
use steamapp::SteamApp;

mod steamapps;
use steamapps::SteamApps;

mod libraryfolders;
use libraryfolders::LibraryFolders;

#[derive(Default, Clone, Debug)]
pub struct SteamDir {
	pub path: PathBuf,
	pub(crate) steam_apps: SteamApps,
	pub(crate) libraryfolders: LibraryFolders,
}

impl SteamDir {
	pub fn get_app(&mut self, app_id: &u32) -> Option<&SteamApp> {
		let steam_apps = &mut self.steam_apps;

		if !steam_apps.apps.contains_key(app_id) {
			let libraryfolders = &mut self.libraryfolders;
			if !libraryfolders.discovered { libraryfolders.discover(&self.path); }
			steam_apps.discover_app(libraryfolders, app_id);
		}

		steam_apps.apps.get(app_id).unwrap().as_ref()
	}

	pub fn get_apps(&mut self) -> &HashMap<u32, Option<SteamApp>> {
		let steam_apps = &mut self.steam_apps;
		if !steam_apps.discovered {
			let libraryfolders = &mut self.libraryfolders;
			if !libraryfolders.discovered { libraryfolders.discover(&self.path); }
			steam_apps.discover_apps(libraryfolders);
		}
		&steam_apps.apps
	}

	#[cfg(target_os="windows")]
	pub fn locate() -> Option<SteamDir> {
		// Locating the Steam installation location is a bit more complicated on Windows

		// Steam's installation location can be found in the registry
		let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
		let installation_regkey = match
			hklm.open_subkey_with_flags("SOFTWARE\\Wow6432Node\\Valve\\Steam", KEY_READ).or_else(|_| // 32-bit
			hklm.open_subkey_with_flags("SOFTWARE\\Valve\\Steam", KEY_READ)) // 64-bit
		{
		    Ok(installation_regkey) => installation_regkey,
		    Err(_) => return None
		};
		
		// The InstallPath key will contain the full path to the Steam directory
		let install_path_str: String = match installation_regkey.get_value("InstallPath") {
			Ok(install_path_str) => install_path_str,
			Err(_) => return None
		};

		let install_path = PathBuf::from(install_path_str);

		Some(SteamDir {
			path: install_path,
			..Default::default()
		})
	}

	#[cfg(target_os="macos")]
	pub fn locate() -> Option<SteamDir> {
		// Steam's installation location is pretty easy to find on macOS, as it's always in $USER/Library/Application Support
		let home_dir = match dirs::home_dir() {
		    Some(home_dir) => home_dir,
		    None => return None
		};
		
		// Find Library/Application Support/Steam
		let install_path = home_dir.join("Library/Application Support/Steam");
		return match install_path.is_dir() {
			false => None,
			true => Some(SteamDir {
				path: install_path,
				..Default::default()
			})
		}
	}

	#[cfg(target_os="linux")]
	pub fn locate() -> Option<SteamDir> {
		// Steam's installation location is pretty easy to find on Linux, too, thanks to the symlink in $USER
		let home_dir = match dirs::home_dir() {
		    Some(home_dir) => home_dir,
		    None => return None
		};
		
		// Find .steam/steam
		let install_path = home_dir.join(".steam/steam");
		return match install_path.is_dir() {
			false => None,
			true => Some(SteamDir {
				path: install_path,
				..Default::default()
			})
		}
	}

	#[cfg(not(any(target_os="windows", target_os="macos", target_os="linux")))]
	pub fn locate() -> Option<SteamDir> {
		panic!("Unsupported operating system");
	}

}

#[cfg(test)]
mod tests;