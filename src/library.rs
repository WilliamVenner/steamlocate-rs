use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    app,
    error::{ParseError, ParseErrorKind},
    App, Error, Result,
};

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
pub(crate) fn parse_library_paths(path: &Path) -> Result<Vec<PathBuf>> {
    let parse_error = |err| Error::parse(ParseErrorKind::LibraryFolders, err, path);

    if !path.is_file() {
        return Err(parse_error(ParseError::missing()));
    }

    let contents = fs::read_to_string(path).map_err(|io| Error::io(io, path))?;
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
                .first()
                .and_then(|value| value.get_obj())
                .and_then(|obj| obj.get("path"))
                .and_then(|values| values.first())
                .and_then(|value| value.get_str())
                .ok_or_else(|| parse_error(ParseError::unexpected_structure()))
                .map(PathBuf::from)
        })
        .collect::<Result<_>>()?;

    Ok(paths)
}

pub struct Iter {
    paths: std::vec::IntoIter<PathBuf>,
}

impl Iter {
    pub(crate) fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            paths: paths.into_iter(),
        }
    }
}

impl Iterator for Iter {
    type Item = Result<Library>;

    fn next(&mut self) -> Option<Self::Item> {
        self.paths.next().map(|path| Library::from_dir(&path))
    }
}

impl ExactSizeIterator for Iter {
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
    pub fn from_dir(path: &Path) -> Result<Self> {
        // Read the manifest files at the library to get an up-to-date list of apps since the
        // values in `libraryfolders.vdf` may be stale
        let mut apps = Vec::new();
        let steamapps = path.join("steamapps");
        for entry in fs::read_dir(&steamapps).map_err(|io| Error::io(io, &steamapps))? {
            let entry = entry.map_err(|io| Error::io(io, &steamapps))?;
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

        Ok(Self {
            path: path.to_owned(),
            apps,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // TODO: if this was sorted then we could locate single apps faster
    pub fn app_ids(&self) -> &[u32] {
        &self.apps
    }

    pub fn app(&self, app_id: u32) -> Option<Result<App>> {
        self.app_ids().iter().find(|&&id| id == app_id).map(|&id| {
            let manifest_path = self
                .path()
                .join("steamapps")
                .join(format!("appmanifest_{}.acf", id));
            App::new(&manifest_path)
        })
    }

    pub fn apps(&self) -> app::Iter {
        app::Iter::new(self)
    }

    /// Resolves the theoretical installation directory for the given `app`
    ///
    /// This is an unvalidated path, so it's up to you to call this with an `app` that's in this
    /// library
    ///
    /// # Example
    ///
    /// ```
    /// # use std::path::Path;
    /// # let temp_steam_dir = steamlocate::tests::helpers::expect_test_env();
    /// # let steam_dir = temp_steam_dir.steam_dir();
    /// const GRAVEYARD_KEEPER: u32 = 599_140;
    /// let (graveyard_keeper, library) = steam_dir.find_app(GRAVEYARD_KEEPER)?.unwrap();
    /// let app_dir = library.resolve_app_dir(&graveyard_keeper);
    /// let expected_rel_path = Path::new("steamapps").join("common").join("Graveyard Keeper");
    /// assert!(app_dir.ends_with(expected_rel_path));
    /// # Ok::<_, steamlocate::tests::TestError>(())
    /// ```
    pub fn resolve_app_dir(&self, app: &App) -> PathBuf {
        self.path
            .join("steamapps")
            .join("common")
            .join(&app.install_dir)
    }
}
