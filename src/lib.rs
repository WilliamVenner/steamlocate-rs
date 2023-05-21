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
//! If you need to get uncached results, simply instantiate a new [SteamDir](https://docs.rs/steamlocate/*/steamlocate/struct.SteamDir.html).
//!
//! # steamid-ng Support
//! This crate supports [steamid-ng](https://docs.rs/steamid-ng) and can automatically convert [SteamApp::last_user](struct.SteamApp.html#structfield.last_user) to a [SteamID](https://docs.rs/steamid-ng/*/steamid_ng/struct.SteamID.html) for you.
//!
//! To enable this support, [use the  `steamid_ng` Cargo.toml feature](#using-steamlocate).
//!
//! # Examples
//!
//! ### Locate the installed Steam directory
//! ```rust
//! extern crate steamlocate;
//! use steamlocate::SteamDir;
//!
//! match SteamDir::locate() {
//!     Some(steamdir) => println!("{:#?}", steamdir),
//!     None => panic!("Couldn't locate Steam on this computer!")
//! }
//! ```
//! ```ignore
//! SteamDir (
//!     path: PathBuf: "C:\\Program Files (x86)\\Steam"
//! )
//! ```
//!
//! ### Locate an installed Steam app by its app ID
//! This will locate Garry's Mod anywhere on the filesystem.
//! ```ignore
//! extern crate steamlocate;
//! use steamlocate::SteamDir;
//!
//! let mut steamdir = SteamDir::locate().unwrap();
//! match steamdir.app(&4000) {
//!     Some(app) => println!("{:#?}", app),
//!     None => panic!("Couldn't locate Garry's Mod on this computer!")
//! }
//! ```
//! ```ignore
//! SteamApp (
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
//!     4000: SteamApp (
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
//!     "C:\\Program Files (x86)\\Steam\\steamapps",
//!     "D:\\Steam\\steamapps",
//!     "E:\\Steam\\steamapps",
//!     "F:\\Steam\\steamapps",
//!     ...
//! }
//! ```

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
compile_error!("Unsupported operating system!");

use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ},
    RegKey,
};
#[cfg(not(target_os = "windows"))]
extern crate dirs;

pub mod steamapp;
pub use steamapp::SteamApp;

#[doc(hidden)]
pub mod libraryfolders;
pub use libraryfolders::{parse_library_folders, Library};

mod shortcut;
pub use shortcut::Shortcut;

/// An instance of a Steam installation.
///
/// All functions of this struct will cache their results.
///
/// If you'd like to dispose of the cache or get uncached results, just instantiate a new `SteamDir`.
///
/// # Example
/// ```rust
/// # use steamlocate::SteamDir;
/// let steamdir = SteamDir::locate();
/// println!("{:#?}", steamdir.unwrap());
/// ```
/// ```ignore
/// SteamDir (
///     path: "C:\\Program Files (x86)\\Steam"
/// )
/// ```
#[derive(Clone, Debug)]
pub struct SteamDir {
    path: PathBuf,
    libraries: Vec<Library>,
    shortcuts: Option<Vec<Shortcut>>,
}

impl SteamDir {
    /// The path to the Steam installation directory on this computer.
    ///
    /// Example: `C:\Program Files (x86)\Steam`
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn libraries(&self) -> &[Library] {
        &self.libraries
    }

    /// Returns a `Some` reference to a `SteamApp` via its app ID.
    ///
    /// If the Steam app is not installed on the system, this will return `None`.
    ///
    /// This function will cache its (either `Some` and `None`) result and will always return a reference to the same `SteamApp`.
    ///
    /// # Example
    /// ```ignore
    /// # use steamlocate::SteamDir;
    /// let mut steamdir = SteamDir::locate().unwrap();
    /// let gmod = steamdir.app(&4000);
    /// println!("{:#?}", gmod.unwrap());
    /// ```
    /// ```ignore
    /// SteamApp (
    ///     appid: u32: 4000,
    ///     path: PathBuf: "C:\\Program Files (x86)\\steamapps\\common\\GarrysMod",
    ///     vdf: <steamy_vdf::Table>,
    ///     name: Some(String: "Garry's Mod"),
    ///     last_user: Some(u64: 76561198040894045) // This will be a steamid_ng::SteamID if the "steamid_ng" feature is enabled
    /// )
    /// ```
    pub fn app(&self, app_id: u32) -> Option<SteamApp> {
        // Search for the `app_id` in each library
        self.libraries().iter().find_map(|lib| lib.app(app_id))
    }

    /// Returns a listing of all added non-Steam games
    pub fn shortcuts(&mut self) -> &[Shortcut] {
        if self.shortcuts.is_none() {
            let shortcuts = shortcut::discover_shortcuts(&self.path);
            self.shortcuts = Some(shortcuts);
        }

        self.shortcuts.as_ref().unwrap()
    }

    /// Locates the Steam installation directory on the filesystem and initializes a `SteamDir` (Windows)
    ///
    /// Returns `None` if no Steam installation can be located.
    pub fn locate() -> Option<SteamDir> {
        let path = Self::locate_steam_dir()?;
        let libraryfolders_vdf = path.join("steamapps").join("libraryfolders.vdf");
        let libraries = parse_library_folders(&libraryfolders_vdf)?;

        Some(Self {
            path,
            libraries,
            shortcuts: None,
        })
    }

    #[cfg(target_os = "windows")]
    fn locate_steam_dir() -> Option<PathBuf> {
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
        install_path.is_dir().then(|| install_path)
    }

    #[cfg(target_os = "macos")]
    fn locate_steam_dir() -> Option<PathBuf> {
        // Steam's installation location is pretty easy to find on macOS, as it's always in $USER/Library/Application Support
        let home_dir = match dirs::home_dir() {
            Some(home_dir) => home_dir,
            None => return None,
        };

        // Find Library/Application Support/Steam
        let install_path = home_dir.join("Library/Application Support/Steam");
        install_path.is_dir().then(|| install_path)
    }

    #[cfg(target_os = "linux")]
    fn locate_steam_dir() -> Option<PathBuf> {
        // Steam's installation location is pretty easy to find on Linux, too, thanks to the symlink in $USER
        let home_dir = dirs::home_dir()?;

        // Check for Flatpak steam install
        let steam_flatpak_path = home_dir.join(".var/app/com.valvesoftware.Steam");
        if steam_flatpak_path.is_dir() {
            let steam_flatpak_install_path = steam_flatpak_path.join(".steam/steam");
            if steam_flatpak_install_path.is_dir() {
                return Some(steam_flatpak_install_path);
            }
        }

        // Check for Standard steam install
        let standard_path = home_dir.join(".steam/steam");
        if standard_path.is_dir() {
            return Some(standard_path);
        }

        None
    }
}

#[cfg(test)]
mod tests;
