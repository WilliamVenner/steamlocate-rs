use std::{
    fs,
    path::{Path, PathBuf},
    slice,
};

use crate::{error::ParseErrorKind, Error, ParseError, Result, SteamApp};

use keyvalues_parser::Vdf;

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
pub fn parse_library_folders(path: &Path) -> Result<LibraryIter> {
    let parse_error = |err| Error::parse(ParseErrorKind::LibaryFolders, err);

    if !path.is_file() {
        return Err(parse_error(ParseError::missing()));
    }

    let contents = fs::read_to_string(path).map_err(Error::Io)?;
    let value = Vdf::parse(&contents)
        .map_err(|err| parse_error(ParseError::from_parser(err)))?
        .value;
    let obj = value
        .get_obj()
        .ok_or_else(|| parse_error(ParseError::unexpected_structure()))?;
    let paths: Vec<_> = obj
        .iter()
        .filter(|(key, _)| key.parse::<u32>().is_ok())
        .map(|(_, values)| {
            values
                .get(0)
                .and_then(|value| value.get_obj())
                .and_then(|obj| obj.get("path"))
                .and_then(|values| values.get(0))
                .and_then(|value| value.get_str())
                .ok_or_else(|| {
                    Error::parse(
                        ParseErrorKind::LibaryFolders,
                        ParseError::unexpected_structure(),
                    )
                })
                .map(PathBuf::from)
        })
        .collect::<Result<_>>()?;

    Ok(LibraryIter {
        paths: paths.into_iter(),
    })
}

pub struct LibraryIter {
    paths: std::vec::IntoIter<PathBuf>,
}

impl Iterator for LibraryIter {
    type Item = Result<Library>;

    fn next(&mut self) -> Option<Self::Item> {
        self.paths.next().map(Library::new)
    }
}

impl ExactSizeIterator for LibraryIter {
    fn len(&self) -> usize {
        self.paths.len()
    }
}

#[derive(Clone, Debug)]
pub struct Library {
    path: PathBuf,
    apps: Vec<u32>,
}

impl Library {
    fn new(path: PathBuf) -> Result<Self> {
        // Read the manifest files at the library to get an up-to-date list of apps since the
        // values in `libraryfolders.vdf` may be stale
        let mut apps = Vec::new();
        for entry in fs::read_dir(path.join("steamapps")).map_err(Error::Io)? {
            let entry = entry.map_err(Error::Io)?;
            if let Some(id) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_prefix("appmanifest_"))
                .and_then(|prefixless_name| prefixless_name.strip_suffix(".acf"))
                .and_then(|app_id_str| app_id_str.parse().ok())
            {
                apps.push(id);
            }
        }

        Ok(Self { path, apps })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // TODO: if this was sorted then we could locate single apps faster
    pub fn app_ids(&self) -> &[u32] {
        &self.apps
    }

    pub fn app(&self, app_id: u32) -> Option<Result<SteamApp>> {
        self.app_ids().iter().find(|&&id| id == app_id).map(|&id| {
            let manifest_path = self
                .path()
                .join("steamapps")
                .join(format!("appmanifest_{}.acf", id));
            SteamApp::new(&self.path, &manifest_path)
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
    type Item = Result<SteamApp>;

    fn next(&mut self) -> Option<Self::Item> {
        let app_id = *self.app_ids.next()?;
        if let some_res @ Some(_) = self.library.app(app_id) {
            some_res
        } else {
            // We use the listing from libraryfolders, so all apps should be accounted for
            Some(Err(Error::MissingExpectedApp { app_id }))
        }
    }
}
