//! A crate which efficiently locates any Steam application on the filesystem, and/or the Steam installation itself.
//!
//! # Using steamlocate
//!
//! Simply add `steamlocate` using
//! [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html).
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
//! let (garrys_mod, _lib) = steam_dir.find_app(GMOD_APP_ID)?.expect("Of course we have G Mod");
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
//!         println!("    App {} - {:?}", app.app_id, app.name);
//!     }
//! }
//! # Ok::<_, steamlocate::tests::TestError>(())
//! ```
//!
//! On my laptop this prints
//!
//! ```text
//! Library - /home/wintermute/.local/share/Steam
//!     App 1628350 - Steam Linux Runtime 3.0 (sniper)
//!     App 1493710 - Proton Experimental
//!     App 4000 - Garry's Mod
//! Library - /home/wintermute/temp steam lib
//!     App 391540 - Undertale
//!     App 1714040 - Super Auto Pets
//!     App 2348590 - Proton 8.0
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
/// Not part of the public API >:V
#[doc(hidden)]
pub mod tests; // TODO: rename this since it may leak out in compiler error messages

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

/// The entrypoint into most of the rest of the API
///
/// Use either [`SteamDir::locate()`] or [`SteamDir::from_dir()`] to create a new instance.
/// From there you have access to:
///
/// - The Steam installation directory
///   - [`steam_dir.path()`][SteamDir::path]
/// - Library info
///   - [`steam_dir.library_paths()`][SteamDir::library_paths]
///   - [`steam_dir.libraries()`][SteamDir::libraries]
/// - Convenient access to find a specific app by id
///   - [`steam_dir.find_app(app_id)`][SteamDir::find_app]
/// - Compatibility tool mapping (aka Proton to game mapping)
///   - [`steam_dir.compat_tool_mapping()`][SteamDir::compat_tool_mapping]
/// - Shortcuts info (aka the listing of non-Steam games)
///   - [`steam_dir.shortcuts()`][SteamDir::shortcuts]
///
/// # Example
/// ```
/// # /*
/// let steam_dir = SteamDir::locate()?;
/// # */
/// # let temp_steam_dir = steamlocate::tests::helpers::expect_test_env();
/// # let steam_dir = temp_steam_dir.steam_dir();
/// assert!(steam_dir.path().ends_with("Steam"));
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

    /// Convenient helper to look through all the libraries for a specific app
    ///
    /// # Example
    /// ```
    /// # let temp_steam_dir = steamlocate::tests::helpers::expect_test_env();
    /// # let steam_dir = temp_steam_dir.steam_dir();
    /// # /*
    /// let steam_dir = SteamDir::locate()?;
    /// # */
    /// const WARFRAME: u32 = 230_410;
    /// let (warframe, library) = steam_dir.find_app(WARFRAME)?.unwrap();
    /// assert_eq!(warframe.app_id, WARFRAME);
    /// assert!(library.app_ids().contains(&warframe.app_id));
    /// # Ok::<_, steamlocate::tests::TestError>(())
    /// ```
    pub fn find_app(&self, app_id: u32) -> Result<Option<(App, Library)>> {
        // Search for the `app_id` in each library
        match self.libraries() {
            Err(e) => Err(e),
            Ok(libraries) => libraries
                .filter_map(|library| library.ok())
                .find_map(|lib| {
                    lib.app(app_id)
                        .map(|maybe_app| maybe_app.map(|app| (app, lib)))
                })
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

    /// Returns an iterator of all non-Steam games that were added to steam
    pub fn shortcuts(&mut self) -> Result<shortcut::Iter> {
        shortcut::Iter::new(&self.path)
    }

    // TODO: rename to `from_dir()` and make consitent with similar constructors on other structs
    pub fn from_dir(path: &Path) -> Result<SteamDir> {
        if !path.is_dir() {
            return Err(Error::validation(ValidationError::missing_dir()));
        }

        // TODO(cosmic): should we do some kind of extra validation here? Could also use validation
        // to determine if a steam dir has been uninstalled. Should fix all the flatpack/snap issues
        Ok(Self {
            path: path.to_owned(),
        })
    }

    /// Attempts to locate the Steam installation directory on the system
    #[cfg(feature = "locate")]
    pub fn locate() -> Result<SteamDir> {
        let path = locate::locate_steam_dir()?;

        Self::from_dir(&path)
    }
}
