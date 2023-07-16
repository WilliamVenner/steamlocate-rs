use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FailedLocatingSteamDir,
    // TODO: make more specific and associate with a path?
    Io(std::io::Error),
    // TODO: associate with a path
    Parse {
        kind: ParseErrorKind,
        error: ParseError,
    },
    MissingExpectedApp {
        app_id: u32,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedLocatingSteamDir => f.write_str("Failed locating the steam dir"),
            Self::Io(err) => write!(f, "Encountered an I/O error: {}", err),
            Self::Parse { kind, error } => write!(
                f,
                "Failed parsing VDF file. File kind: {:?}, Error: {}",
                kind, error
            ),
            Self::MissingExpectedApp { app_id } => {
                write!(f, "Missing expected app with id: {}", app_id)
            }
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub(crate) fn parse(kind: ParseErrorKind, error: ParseError) -> Self {
        Self::Parse { kind, error }
    }
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum ParseErrorKind {
    LibaryFolders,
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
pub enum ParseErrorInner {
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
