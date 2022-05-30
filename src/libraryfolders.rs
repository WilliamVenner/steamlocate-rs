use std::{
    fs,
    path::{Path, PathBuf},
    slice,
};

use crate::SteamApp;

use keyvalues_parser::{Obj, Vdf};

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
///         "apps"
///         {
///             <app-id>    <size>
///             ... // for all apps in the library
///         }
///     }
///     "1"
///     {
///         "path"    "/path/to/second/library"
///         ...
///         "apps"
///         {
///             <app-id>    <size>
///             ... // for all apps in the library
///         }
///     }
///     ...
/// }
/// ```
pub fn parse_library_folders(path: &Path) -> Option<Vec<Library>> {
    if !path.is_file() {
        return None;
    }

    let contents = fs::read_to_string(path).ok()?;
    let value = Vdf::parse(&contents).ok()?.value;
    let obj = value.get_obj()?;

    // Parse the information from each library object
    let libraries: Vec<_> = obj
        .iter()
        .filter(|(key, values)| key.parse::<u32>().is_ok() && values.len() == 1)
        .filter_map(|(_, values)| {
            let library_obj = values.get(0)?.get_obj()?;
            Library::new(&library_obj)
        })
        .collect();

    Some(libraries)
}

#[derive(Clone, Debug)]
pub struct Library {
    path: PathBuf,
    apps: Vec<u32>,
}

impl Library {
    fn new(obj: &Obj) -> Option<Self> {
        let path = obj.get("path")?.get(0)?.get_str()?;
        let path = PathBuf::from(path);
        let apps = obj
            .get("apps")?
            .get(0)?
            .get_obj()?
            .keys()
            .map(|app_str| app_str.parse().ok())
            .collect::<Option<_>>()?;

        Some(Self { path, apps })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn app_ids(&self) -> &[u32] {
        &self.apps
    }

    pub fn app(&self, app_id: u32) -> Option<SteamApp> {
        self.app_ids()
            .iter()
            .find(|&&id| id == app_id)
            .and_then(|&id| {
                let manifest_path = self
                    .path()
                    .join("steamapps")
                    .join(format!("appmanifest_{}.acf", id));
                SteamApp::new(&manifest_path)
            })
    }

    pub fn apps(&self) -> AppIter {
        AppIter {
            library: self,
            app_ids: self.app_ids().iter(),
        }
    }
}

pub struct AppIter<'library> {
    library: &'library Library,
    app_ids: slice::Iter<'library, u32>,
}

impl<'library> Iterator for AppIter<'library> {
    // TODO: this will make a lot more sense when it becomes a `Result<SteamApp>`
    type Item = Option<SteamApp>;

    fn next(&mut self) -> Option<Self::Item> {
        self.app_ids.next().map(|&id| self.library.app(id))
    }
}
