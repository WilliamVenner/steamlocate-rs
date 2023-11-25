//! A crate which efficiently locates any Steam application on the filesystem, and/or the Steam installation itself.
//!
//! **This crate supports Windows, macOS and Linux.**
//!
//! # Using steamlocate
//! Simply add to your [Cargo.toml](https://doc.rust-lang.org/cargo/reference/manifest.html) file:
//! ```toml
//! [dependencies]
//! steamlocate = "0.*"
//! ```
//!
//! To use [steamid-ng](#steamid-ng-support) with steamlocate, add this to your [Cargo.toml](https://doc.rust-lang.org/cargo/reference/manifest.html) file:
//! ```toml
//! [dependencies]
//! steamid-ng = "1.*"
//!
//! [dependencies.steamlocate]
//! version = "0.*"
//! features = ["steamid_ng"]
//! ```
//!
//! # Caching
//! All functions in this crate cache their results, meaning you can call them as many times as you like and they will always return the same reference.
//!
//! If you need to get uncached results, simply instantiate a new [InstallDir](https://docs.rs/steamlocate/*/steamlocate/struct.InstallDir.html).
//!
//! # steamid-ng Support
//! This crate supports [steamid-ng](https://docs.rs/steamid-ng) and can automatically convert [App::last_user](struct.App.html#structfield.last_user) to a [SteamID](https://docs.rs/steamid-ng/*/steamid_ng/struct.SteamID.html) for you.
//!
//! To enable this support, [use the  `steamid_ng` Cargo.toml feature](#using-steamlocate).
//!
//! # Examples
//!
//! ### Locate the installed Steam directory
//! ```rust,ignore
//! extern crate steamlocate;
//! use steamlocate::InstallDir;
//!
//! match InstallDir::locate() {
//!     Ok(steamdir) => println!("{:#?}", steamdir),
//!     Err(_) => panic!("Couldn't locate Steam on this computer!")
//! }
//! ```
//! ```ignore
//! InstallDir (
//!     path: PathBuf: "C:\\Program Files (x86)\\Steam"
//! )
//! ```
//!
//! ### Locate an installed Steam app by its app ID
//! This will locate Garry's Mod anywhere on the filesystem.
//! ```ignore
//! extern crate steamlocate;
//! use steamlocate::InstallDir;
//!
//! let mut steamdir = InstallDir::locate().unwrap();
//! match steamdir.app(&4000) {
//!     Some(app) => println!("{:#?}", app),
//!     None => panic!("Couldn't locate Garry's Mod on this computer!")
//! }
//! ```
//! ```ignore
//! App (
//!     appid: u32: 4000,
//!     path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
//!     vdf: <steamy_vdf::Table>,
//!     name: Some(String: "Garry's Mod"),
//!     last_user: Some(u64: 76561198040894045)
//! )
//! ```
//!
//! ### Locate all Steam apps on this filesystem
//! ```ignore
//! extern crate steamlocate;
//! use steamlocate::{InstallDir, App};
//! use std::collections::HashMap;
//!
//! let mut steamdir = InstallDir::locate().unwrap();
//! let apps: &HashMap<u32, Option<App>> = steamdir.apps();
//!
//! println!("{:#?}", apps);
//! ```
//! ```ignore
//! {
//!     4000: App (
//!         appid: u32: 4000,
//!         path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
//!         vdf: <steamy_vdf::Table>,
//!         name: Some(String: "Garry's Mod"),
//!         last_user: Some(u64: 76561198040894045)
//!     )
//!     ...
//! }
//! ```
//!
//! ### Locate all Steam library folders
//! ```ignore
//! extern crate steamlocate;
//! use steamlocate::{InstallDir, LibraryFolders};
//! use std::{vec, path::PathBuf};
//!
//! let mut steamdir: InstallDir = InstallDir::locate().unwrap();
//! let libraryfolders: &LibraryFolders = steamdir.libraryfolders();
//! let paths: &Vec<PathBuf> = &libraryfolders.paths;
//!
//! println!("{:#?}", paths);
//! ```
//! ```ignore
//! {
//!     "C:\\Program Files (x86)\\Steam\\steamapps",
//!     "D:\\Steam\\steamapps",
//!     "E:\\Steam\\steamapps",
//!     "F:\\Steam\\steamapps",
//!     ...
//! }
//! ```

#![warn(
	// We're a library after all
	clippy::print_stderr, clippy::print_stdout
)]

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
compile_error!("Unsupported operating system!");

pub mod app;
pub mod error;
pub mod library;
pub mod shortcut;
#[cfg(any(test, doctest))]
pub mod tests;

use std::path::{Path, PathBuf};

pub use crate::app::App;
pub use crate::error::{Error, Result};
pub use crate::library::Library;
pub use crate::shortcut::Shortcut;

/// An instance of a Steam installation.
///
/// All functions of this struct will cache their results.
///
/// If you'd like to dispose of the cache or get uncached results, just instantiate a new `InstallDir`.
///
/// # Example
/// ```rust,ignore
/// # use steamlocate::InstallDir;
/// let steamdir = InstallDir::locate();
/// println!("{:#?}", steamdir.unwrap());
/// ```
/// ```ignore
/// InstallDir (
///     path: "C:\\Program Files (x86)\\Steam"
/// )
/// ```
#[derive(Clone, Debug)]
pub struct InstallDir {
    path: PathBuf,
}

impl InstallDir {
    /// The path to the Steam installation directory on this computer.
    ///
    /// Example: `C:\Program Files (x86)\Steam`
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn libraries(&self) -> Result<library::Iter> {
        let libraryfolders_vdf = self.path.join("steamapps").join("libraryfolders.vdf");
        library::parse_library_folders(&libraryfolders_vdf)
    }

    /// Returns a `Some` reference to a `App` via its app ID.
    ///
    /// If the Steam app is not installed on the system, this will return `None`.
    ///
    /// This function will cache its (either `Some` and `None`) result and will always return a reference to the same `App`.
    ///
    /// # Example
    /// ```ignore
    /// # use steamlocate::InstallDir;
    /// let mut steamdir = InstallDir::locate().unwrap();
    /// let gmod = steamdir.app(&4000);
    /// println!("{:#?}", gmod.unwrap());
    /// ```
    /// ```ignore
    /// App (
    ///     appid: u32: 4000,
    ///     path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
    ///     vdf: <steamy_vdf::Table>,
    ///     name: Some(String: "Garry's Mod"),
    ///     last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
    /// )
    /// ```
    pub fn app(&self, app_id: u32) -> Result<Option<App>> {
        // Search for the `app_id` in each library
        match self.libraries() {
            Err(e) => Err(e),
            Ok(libraries) => libraries
                .filter_map(|library| library.ok())
                .find_map(|lib| lib.app(app_id))
                .transpose(),
        }
    }

    /// Returns a listing of all added non-Steam games
    pub fn shortcuts(&mut self) -> Result<shortcut::Iter> {
        shortcut::Iter::new(&self.path)
    }

    pub fn from_steam_dir(path: &Path) -> Result<InstallDir> {
        if !path.is_dir() {
            return Err(Error::FailedLocatingInstallDir);
        }

        // TODO(cosmic): should we do some kind of extra validation here? Could also use validation
        // to determine if a steam dir has been uninstalled. Should fix all the flatpack/snap issues
        Ok(Self {
            path: path.to_owned(),
        })
    }

    /// Locates the Steam installation directory on the filesystem and initializes a `InstallDir` (Windows)
    ///
    /// Returns `None` if no Steam installation can be located.
    pub fn locate() -> Result<InstallDir> {
        let path = Self::locate_steam_dir().ok_or(Error::FailedLocatingInstallDir)?;

        Ok(Self { path })
    }

    #[cfg(target_os = "windows")]
    fn locate_steam_dir() -> Option<PathBuf> {
        use winreg::{
            enums::{HKEY_LOCAL_MACHINE, KEY_READ},
            RegKey,
        };

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
            .ok()?;

        // The InstallPath key will contain the full path to the Steam directory
        let install_path_str: String = installation_regkey.get_value("InstallPath").ok()?;

        let install_path = PathBuf::from(install_path_str);
        install_path.is_dir().then_some(install_path)
    }

    #[cfg(target_os = "macos")]
    fn locate_steam_dir() -> Option<PathBuf> {
        // Steam's installation location is pretty easy to find on macOS, as it's always in $USER/Library/Application Support
        let home_dir = dirs::home_dir()?;

        // Find Library/Application Support/Steam
        let install_path = home_dir.join("Library/Application Support/Steam");
        install_path.is_dir().then_some(install_path)
    }

    #[cfg(target_os = "linux")]
    fn locate_steam_dir() -> Option<PathBuf> {
        use std::env;

        // Steam's installation location is pretty easy to find on Linux, too, thanks to the symlink in $USER
        let home_dir = dirs::home_dir()?;
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

        steam_paths.into_iter().find(|x| x.is_dir())
    }
}
