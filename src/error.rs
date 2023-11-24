use std::{
    fmt,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    FailedLocatingSteamDir,
    Io {
        inner: std::io::Error,
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
            Self::FailedLocatingSteamDir => f.write_str("Failed locating the steam dir"),
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
    pub(crate) fn io(io: std::io::Error, path: &Path) -> Self {
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

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum ParseErrorKind {
    LibraryFolders,
    SteamApp,
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
