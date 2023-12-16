//! A crate which efficiently locates any Steam application on the filesystem, and/or the Steam installation itself.
//!
//! # Using steamlocate
//!
//! Simply add `steamlocate` using
//! [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html)
//!
//! ```console
//! $ cargo add steamlocate
//! ```
//!
//! ## Feature flags
//!
//! Default: `locate`
//!
//! | Feature flag | Description |
//! | :---: | :--- |
//! | `locate` | Enables automatically detecting the Steam installation on supported platforms (currently Windows, MacOS, and Linux). Unsupported platforms will return a runtime error. |
//!
//! # Examples
//!
//! ## Locate the Steam installation and a specific game
//!
//! The [`SteamDir`] is going to be your entrypoint into _most_ parts of the API. After you locate
//! it you can access related information.
//!
//! ```rust
//! # /*
//! let steam_dir = steamlocate::SteamDir::locate()?;
//! # */
//! # let temp_steam_dir = steamlocate::tests::helpers::expect_test_env();
//! # let steam_dir = temp_steam_dir.steam_dir();
//! println!("Steam installation - {}", steam_dir.path().display());
//! // ^^ prints something like `Steam installation - C:\Program Files (x86)\Steam`
//!
//! const GMOD_APP_ID: u32 = 4_000;
//! let garrys_mod = steam_dir.app(GMOD_APP_ID)?.expect("Of course we have G Mod");
//! assert_eq!(garrys_mod.name.as_ref().unwrap(), "Garry's Mod");
//! println!("{garrys_mod:#?}");
//! // ^^ prints something like vv
//! # Ok::<_, steamlocate::tests::TestError>(())
//! ```
//! ```ignore
//! App {
//!     app_id: 4_000,
//!     install_dir: "GarrysMod",
//!     name: Some("Garry's Mod"),
//!     universe: Some(Public),
//!     // much much more data
//! }
//! ```
//!
//! ## Get an overview of all libraries and apps on the system
//!
//! You can iterate over all of Steam's libraries from the steam dir. Then from each library you
//! can iterate over all of its apps.
//!
//! ```
//! # /*
//! let steam_dir = steamlocate::SteamDir::locate()?;
//! # */
//! # let temp_steam_dir = steamlocate::tests::helpers::expect_test_env();
//! # let steam_dir = temp_steam_dir.steam_dir();
//!
//! for library in steam_dir.libraries()? {
//!     let library = library?;
//!     println!("Library - {}", library.path().display());
//!
//!     for app in library.apps() {
//!         let app = app?;
//!         println!(
//!             "    App {} - {}",
//!             app.app_id,
//!             app.name.as_deref().unwrap_or("<no_name>")
//!         );
//!     }
//! }
//! # Ok::<_, steamlocate::tests::TestError>(())
//! ```
//!
//! On my laptop this prints
//!
//! ```text
//! Steam Dir - "/home/wintermute/.steam/steam"
//!    Library - /home/wintermute/.local/share/Steam
//!        App 1628350 - Steam Linux Runtime 3.0 (sniper)
//!        App 1493710 - Proton Experimental
//!        App 4000 - Garry's Mod
//!    Library - /home/wintermute/temp steam lib
//!        App 391540 - Undertale
//!        App 1714040 - Super Auto Pets
//!        App 2348590 - Proton 8.0
//! ```

#![warn(
	// We're a library after all
	clippy::print_stderr, clippy::print_stdout,
	// Honestly just good in general
	clippy::todo,
)]

pub mod app;
pub mod config;
pub mod error;
pub mod library;
#[cfg(feature = "locate")]
mod locate;
pub mod shortcut;
// NOTE: exposed publicly, so that we can use them in doctests
/// Not part of the public API
#[doc(hidden)]
pub mod tests; // TODO: rename this if it's gonna be part of the public API

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use error::ValidationError;

use crate::error::{ParseError, ParseErrorKind};

pub use crate::app::App;
pub use crate::config::CompatTool;
pub use crate::error::{Error, Result};
pub use crate::library::Library;
pub use crate::shortcut::Shortcut;

/// An instance of a Steam installation.
///
/// All functions of this struct will cache their results.
///
/// If you'd like to dispose of the cache or get uncached results, just instantiate a new `SteamDir`.
///
/// # Example
/// ```rust,ignore
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
}

impl SteamDir {
    /// The path to the Steam installation directory on this computer.
    ///
    /// Example: `C:\Program Files (x86)\Steam`
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn library_paths(&self) -> Result<Vec<PathBuf>> {
        let libraryfolders_vdf = self.path.join("steamapps").join("libraryfolders.vdf");
        library::parse_library_paths(&libraryfolders_vdf)
    }

    pub fn libraries(&self) -> Result<library::Iter> {
        let paths = self.library_paths()?;
        Ok(library::Iter::new(paths))
    }

    /// Returns a `Some` reference to a `App` via its app ID.
    ///
    /// If the Steam app is not installed on the system, this will return `None`.
    ///
    /// This function will cache its (either `Some` and `None`) result and will always return a reference to the same `App`.
    ///
    /// # Example
    /// ```ignore
    /// # use steamlocate::SteamDir;
    /// let mut steamdir = SteamDir::locate().unwrap();
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

    pub fn compat_tool_mapping(&self) -> Result<HashMap<u32, CompatTool>> {
        let config_path = self.path.join("config").join("config.vdf");
        let vdf_text =
            fs::read_to_string(&config_path).map_err(|io| Error::io(io, &config_path))?;
        let store: config::Store = keyvalues_serde::from_str(&vdf_text).map_err(|de| {
            Error::parse(
                ParseErrorKind::Config,
                ParseError::from_serde(de),
                &config_path,
            )
        })?;

        Ok(store.software.valve.steam.mapping)
    }

    /// Returns a listing of all added non-Steam games
    pub fn shortcuts(&mut self) -> Result<shortcut::Iter> {
        shortcut::Iter::new(&self.path)
    }

    pub fn from_steam_dir(path: &Path) -> Result<SteamDir> {
        if !path.is_dir() {
            return Err(Error::validation(ValidationError::missing_dir()));
        }

        // TODO(cosmic): should we do some kind of extra validation here? Could also use validation
        // to determine if a steam dir has been uninstalled. Should fix all the flatpack/snap issues
        Ok(Self {
            path: path.to_owned(),
        })
    }

    /// Locates the Steam installation directory on the filesystem and initializes a `SteamDir` (Windows)
    ///
    /// Returns `None` if no Steam installation can be located.
    #[cfg(feature = "locate")]
    pub fn locate() -> Result<SteamDir> {
        let path = locate::locate_steam_dir()?;

        Self::from_steam_dir(&path)
    }
}
