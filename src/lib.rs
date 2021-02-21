//! A crate which efficiently locates any Steam application on the filesystem, and/or the Steam installation itself.
//!
//! **This crate supports Windows, macOS and Linux.**
//!
//! # Caching
//! All functions in this crate cache their results, meaning you can call them as many times as you like and they will always return the same reference.
//!
//! If you need to get uncached results, simply instantiate a new [SteamDir](struct.SteamDir.html).
//!
//! # steamid-ng Support
//! This crate supports [steamid-ng](/steamid-ng) and can automatically convert [SteamApp::last_user](struct.SteamApp.html#structfield.last_user) to a [SteamID](/steamid-ng/*/steamid-ng/struct.SteamID.html) for you.
//!
//! To enable this feature, build with `cargo build --features steamid_ng`
//!
//! # Examples
//!
//! ## Locate the installed Steam directory
//! ```rust
//! extern crate steamlocate;
//! use steamlocate::SteamDir;
//!
//! match SteamDir::locate() {
//! 	Some(steamdir) => println!("{:#?}", steamdir),
//! 	None => panic!("Couldn't locate Steam on this computer!")
//! }
//! ```
//! ```ignore
//! SteamDir (
//! 	path: PathBuf: "C:\\Program Files (x86)\\Steam"
//! )
//! ```
//!
//! ## Locate an installed Steam app by its app ID
//! This will locate Garry's Mod anywhere on the filesystem.
//! ```rust
//! extern crate steamlocate;
//! use steamlocate::SteamDir;
//!
//! let mut steamdir = SteamDir::locate().unwrap();
//! match steamdir.app(&4000) {
//! 	Some(app) => println!("{:#?}", app),
//! 	None => panic!("Couldn't locate Garry's Mod on this computer!")
//! }
//! ```
//! ```ignore
//! SteamApp (
//! 	appid: u32: 4000,
//! 	path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
//! 	vdf: <steamy_vdf::Table>,
//! 	name: Some(String: "Garry's Mod"),
//! 	last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
//! )
//! ```
//!
//! ## Locate all Steam apps on this filesystem
//! ```rust
//! extern crate steamlocate;
//! use steamlocate::{SteamDir, SteamApp};
//! use std::collections::HashMap;
//!
//! let mut steamdir = SteamDir::locate().unwrap();
//! let apps: &HashMap<u32, Option<SteamApp>> = steamdir.apps();
//!
//! println!("{:#?}", apps);
//! ```
//! ```ignore
//! {
//! 	4000: SteamApp (
//! 		appid: u32: 4000,
//! 		path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
//! 		vdf: <steamy_vdf::Table>,
//! 		name: Some(String: "Garry's Mod"),
//! 		last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
//! 	)
//! 	...
//! }
//! ```
//!
//! ## Locate all Steam library folders
//! ```rust
//! extern crate steamlocate;
//! use steamlocate::{SteamDir, LibraryFolders};
//! use std::{vec, path::PathBuf};
//!
//! let mut steamdir: SteamDir = SteamDir::locate().unwrap();
//! let libraryfolders: &LibraryFolders = steamdir.libraryfolders();
//! let paths: &Vec<PathBuf> = &libraryfolders.paths;
//!
//! println!("{:#?}", paths);
//! ```
//! ```ignore
//! {
//!		"C:\\Program Files (x86)\\Steam\\steamapps",
//!		"D:\\Steam\\steamapps",
//!		"E:\\Steam\\steamapps",
//!		"F:\\Steam\\steamapps",
//!		...
//!	}
//! ```

#[macro_use] extern crate lazy_static;

use std::{collections::HashMap, path::PathBuf};

#[cfg(target_os="windows")]
use winreg::{RegKey, enums::{HKEY_LOCAL_MACHINE, KEY_READ}};
#[cfg(not(target_os="windows"))]
extern crate dirs;

#[doc(hidden)]
pub mod steamapp;
pub use steamapp::SteamApp;

#[doc(hidden)]
pub mod libraryfolders;
pub use libraryfolders::LibraryFolders;

mod steamapps;
use steamapps::SteamApps;

/// An instance of a Steam installation.
///
/// All functions of this struct will cache their results.
///
/// If you'd like to dispose of the cache or get uncached results, just instantiate a new `SteamDir`.
///
/// # Example
/// ```rust
/// # use steamlocate::SteamDir;
///	let steamdir = SteamDir::locate();
/// println!("{:#?}", steamdir.unwrap());
/// ```
/// ```ignore
/// SteamDir (
/// 	path: "C:\\Program Files (x86)\\Steam"
/// )
/// ```
#[derive(Default, Clone, Debug)]
pub struct SteamDir {
	/// The path to the Steam installation directory on this computer.
	///
	/// Example: `C:\Program Files (x86)\Steam`
	pub path: PathBuf,
	pub(crate) steam_apps: SteamApps,
	pub(crate) libraryfolders: LibraryFolders,
}

impl SteamDir {
	/// Returns a reference to a `LibraryFolders` instance.
	///
	/// You can then index `LibraryFolders.paths` to get a reference to a `Vec<PathBuf>` of every library folder installed on the file system.
	///
	/// This function will cache its result.
	pub fn libraryfolders(&mut self) -> &LibraryFolders {
		let libraryfolders = &mut self.libraryfolders;
		if !libraryfolders.discovered { libraryfolders.discover(&self.path); }
		&*libraryfolders
	}

	/// Returns a reference to `HashMap<u32, Option<SteamApp>>` of all `SteamApp`s located on this computer.
	///
	/// All `Option<SteamApp>` in this context will be `Some`, so you can safely `unwrap()` them without panicking.
	///
	/// This function will cache its results and will always return a reference to the same `HashMap`.
	/// # Example
	/// ```rust
	/// # use steamlocate::{SteamDir, SteamApp};
	/// # use std::collections::HashMap;
	/// let mut steamdir = SteamDir::locate().unwrap();
	/// let apps: &HashMap<u32, Option<SteamApp>> = steamdir.apps();
	/// println!("{:#?}", apps);
	/// ```
	/// ```ignore
	/// {
	/// 	4000: SteamApp (
	/// 		appid: u32: 4000,
	/// 		path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
	/// 		vdf: <steamy_vdf::Table>,
	/// 		name: Some(String: "Garry's Mod"),
	/// 		last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
	/// 	)
	/// 	...
	/// }
	/// ```
	pub fn apps(&mut self) -> &HashMap<u32, Option<SteamApp>> {
		let steam_apps = &mut self.steam_apps;
		if !steam_apps.discovered {
			let libraryfolders = &mut self.libraryfolders;
			if !libraryfolders.discovered { libraryfolders.discover(&self.path); }
			steam_apps.discover_apps(libraryfolders);
		}
		&steam_apps.apps
	}

	/// Returns a `Some` reference to a `SteamApp` via its app ID.
	///
	/// If the Steam app is not installed on the system, this will return `None`.
	///
	/// This function will cache its (either Some and None) result and will always return a reference to the same `SteamApp`.
	///
	/// # Example
	/// ```rust
	/// # use steamlocate::SteamDir;
	/// let mut steamdir = SteamDir::locate().unwrap();
	/// let gmod = steamdir.app(&4000);
	/// println!("{:#?}", gmod.unwrap());
	/// ```
	/// ```ignore
	/// SteamApp (
	/// 	appid: u32: 4000,
	/// 	path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
	/// 	vdf: <steamy_vdf::Table>,
	/// 	name: Some(String: "Garry's Mod"),
	/// 	last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
	/// )
	/// ```
	pub fn app(&mut self, app_id: &u32) -> Option<&SteamApp> {
		let steam_apps = &mut self.steam_apps;

		if !steam_apps.apps.contains_key(app_id) {
			let libraryfolders = &mut self.libraryfolders;
			if !libraryfolders.discovered { libraryfolders.discover(&self.path); }
			steam_apps.discover_app(libraryfolders, app_id);
		}

		steam_apps.apps.get(app_id).unwrap().as_ref()
	}

	/// Locates the Steam installation directory on the filesystem and initializes a `SteamDir` (Windows)
	///
	/// Returns `None` if no Steam installation can be located.
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

	/// Locates the Steam installation directory on the filesystem and initializes a `SteamDir` (macOS)
	///
	/// Returns `None` if no Steam installation can be located.
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

	/// Locates the Steam installation directory on the filesystem and initializes a `SteamDir` (Linux)
	///
	/// Returns `None` if no Steam installation can be located.
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

	/// UNSUPPORTED OPERATING SYSTEM! This will panic!
	///
	/// target_os must be `any(target_os="windows", target_os="macos", target_os="linux")`
	#[cfg(not(any(target_os="windows", target_os="macos", target_os="linux")))]
	pub fn locate() -> Option<SteamDir> {
		panic!("Unsupported operating system");
	}

}

#[cfg(test)]
mod tests;