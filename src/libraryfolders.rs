use std::{
    fs,
    path::{Path, PathBuf},
};

use keyvalues_parser::Vdf;

/// An instance which contains all the Steam library folders installed on the file system.
/// Example:
/// ```rust
/// # use std::{vec, path::PathBuf};
/// # use steamlocate::{SteamDir, LibraryFolders};
/// let mut steamdir: SteamDir = SteamDir::locate().unwrap();
/// let libraryfolders: &LibraryFolders = steamdir.libraryfolders();
/// let paths: &Vec<PathBuf> = &libraryfolders.paths;
/// println!("{:#?}", paths);
/// ```
/// ```ignore
/// {
///		"C:\\Program Files (x86)\\Steam\\steamapps",
///		"D:\\Steam\\steamapps",
///		"E:\\Steam\\steamapps",
///		"F:\\Steam\\steamapps",
///		...
///	}
/// ```
#[derive(Default, Clone, Debug)]
pub struct LibraryFolders {
    /// A `Vec<PathBuf>` of Steam library folder paths.
    ///
    /// This will always include the Steam installation directory's `SteamApps` folder.
    pub paths: Vec<PathBuf>,
    pub(crate) discovered: bool,
}

impl LibraryFolders {
    /// Discovers all the steam libraries from `libraryfolders.vdf`
    ///
    /// We want all the library paths from `libraryfolders.vdf` which has the following structure
    ///
    /// ```vdf
    /// "libraryfolders"
    /// {
    ///     ...
    ///     "0"
    ///     {
    ///         "path"    "/path/to/first/library"
    ///         ...
    ///     }
    ///     "1"
    ///     {
    ///         "path"    "/path/to/second/library"
    ///         ...
    ///     }
    ///     ...
    /// }
    /// ```
    pub(crate) fn discover(&mut self, path: &Path) {
        let _ = self._discover(path);
    }

    fn _discover(&mut self, path: &Path) -> Option<()> {
        let steamapps = path.join("steamapps");
        self.paths.push(steamapps.clone());

        let libraryfolders_vdf_path = steamapps.join("libraryfolders.vdf");
        if libraryfolders_vdf_path.is_file() {
            let vdf_text = fs::read_to_string(&libraryfolders_vdf_path).ok()?;
            let value = Vdf::parse(&vdf_text).ok()?.value;
            let obj = value.get_obj()?;

            let library_folders: Vec<_> = obj
                .iter()
                .filter(|(key, values)| key.parse::<u32>().is_ok() && values.len() == 1)
                .filter_map(|(_, values)| {
                    let library_folder_string = values
                        .get(0)?
                        .get_obj()?
                        .get("path")?
                        .get(0)?
                        .get_str()?
                        .to_string();
                    let library_folder = PathBuf::from(library_folder_string).join("steamapps");
                    Some(library_folder)
                })
                .collect();

            self.paths = library_folders;
        }

        self.discovered = true;

        Some(())
    }
}
