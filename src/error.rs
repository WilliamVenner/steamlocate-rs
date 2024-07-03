use std::{
    fmt, io,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    FailedLocate(LocateError),
    InvalidSteamDir(ValidationError),
    Io {
        inner: io::Error,
        path: PathBuf,
    },
    Parse {
        kind: ParseErrorKind,
        error: ParseError,
        path: PathBuf,
    },
    MissingExpectedApp {
        app_id: u32,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedLocate(error) => {
                write!(f, "Failed locating the steam dir. Error: {error}")
            }
            Self::InvalidSteamDir(error) => {
                write!(f, "Failed validating steam dir. Error: {error}")
            }
            Self::Io { inner: err, path } => {
                write!(f, "Encountered an I/O error: {} at {}", err, path.display())
            }
            Self::Parse { kind, error, path } => write!(
                f,
                "Failed parsing VDF file. File kind: {:?}, Error: {} at {}",
                kind,
                error,
                path.display(),
            ),
            Self::MissingExpectedApp { app_id } => {
                write!(f, "Missing expected app with id: {}", app_id)
            }
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    #[cfg(feature = "locate")]
    pub(crate) fn locate(locate: LocateError) -> Self {
        Self::FailedLocate(locate)
    }

    pub(crate) fn validation(validation: ValidationError) -> Self {
        Self::InvalidSteamDir(validation)
    }

    pub(crate) fn io(io: io::Error, path: &Path) -> Self {
        Self::Io {
            inner: io,
            path: path.to_owned(),
        }
    }

    pub(crate) fn parse(kind: ParseErrorKind, error: ParseError, path: &Path) -> Self {
        Self::Parse {
            kind,
            error,
            path: path.to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LocateError {
    Backend(BackendError),
    Unsupported,
}

impl LocateError {
    #[cfg(all(feature = "locate", target_os = "windows"))]
    pub(crate) fn winreg(io: io::Error) -> Self {
        Self::Backend(BackendError {
            inner: BackendErrorInner(std::sync::Arc::new(io)),
        })
    }

    #[cfg(all(feature = "locate", not(target_os = "windows")))]
    pub(crate) fn no_home() -> Self {
        Self::Backend(BackendError {
            inner: BackendErrorInner::NoHome,
        })
    }
}

impl fmt::Display for LocateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(error) => error.fmt(f),
            Self::Unsupported => f.write_str("Unsupported platform"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BackendError {
    #[cfg(feature = "locate")]
    #[allow(dead_code)] // Only used for displaying currently
    inner: BackendErrorInner,
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(all(feature = "locate", target_os = "windows"))]
        {
            write!(f, "{}", self.inner.0)
        }
        #[cfg(all(feature = "locate", not(target_os = "windows")))]
        {
            match self.inner {
                BackendErrorInner::NoHome => f.write_str("Unable to locate the user's $HOME"),
            }
        }
        #[cfg(not(feature = "locate"))]
        {
            // "Use" the unused value
            let _ = f;
            unreachable!("This should never be constructed!");
        }
    }
}

// TODO: move all this conditional junk into different modules, so that I don't have to keep
// repeating it everywhere
#[derive(Clone, Debug)]
#[cfg(all(feature = "locate", target_os = "windows"))]
struct BackendErrorInner(std::sync::Arc<io::Error>);
#[derive(Clone, Debug)]
#[cfg(all(feature = "locate", not(target_os = "windows")))]
enum BackendErrorInner {
    NoHome,
}

#[derive(Clone, Debug)]
pub struct ValidationError {
    #[allow(dead_code)] // Only used for displaying currently
    inner: ValidationErrorInner,
}

impl ValidationError {
    pub(crate) fn missing_dir() -> Self {
        Self {
            inner: ValidationErrorInner::MissingDirectory,
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            ValidationErrorInner::MissingDirectory => f.write_str(
                "The Steam installation directory either isn't a directory or doesn't exist",
            ),
        }
    }
}

#[derive(Clone, Debug)]
enum ValidationErrorInner {
    MissingDirectory,
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum ParseErrorKind {
    Config,
    LibraryFolders,
    App,
    Shortcut,
}

#[derive(Debug)]
pub struct ParseError {
    // Keep `keyvalues_parser` and `keyvalues_serde` types out of the public API (this includes
    // from traits, so no using `thiserror` with `#[from]`)
    #[allow(dead_code)] // Only used for displaying currently
    inner: Box<ParseErrorInner>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug)]
pub(crate) enum ParseErrorInner {
    Parse(keyvalues_parser::error::Error),
    Serde(keyvalues_serde::error::Error),
    UnexpectedStructure,
    Missing,
}

impl fmt::Display for ParseErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "{}", err),
            Self::Serde(err) => write!(f, "{}", err),
            Self::UnexpectedStructure => f.write_str("File did not match expected structure"),
            Self::Missing => f.write_str("Expected file was missing"),
        }
    }
}

impl ParseError {
    pub(crate) fn new(inner: ParseErrorInner) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    pub(crate) fn from_parser(err: keyvalues_parser::error::Error) -> Self {
        Self::new(ParseErrorInner::Parse(err))
    }

    pub(crate) fn from_serde(err: keyvalues_serde::error::Error) -> Self {
        Self::new(ParseErrorInner::Serde(err))
    }

    pub(crate) fn unexpected_structure() -> Self {
        Self::new(ParseErrorInner::UnexpectedStructure)
    }

    pub(crate) fn missing() -> Self {
        Self::new(ParseErrorInner::Missing)
    }
}
