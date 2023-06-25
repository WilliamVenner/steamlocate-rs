pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    FailedLocatingSteamDir,
    Io(std::io::Error),
    Parse {
        kind: ParseErrorKind,
        error: ParseError,
    },
    MissingExpectedApp {
        app_id: u32,
    },
}

impl Error {
    pub(crate) fn parse(kind: ParseErrorKind, error: ParseError) -> Self {
        Self::Parse { kind, error }
    }
}

// TODO: rename to something like target?
#[derive(Copy, Clone, Debug)]
pub enum ParseErrorKind {
    LibaryFolders,
    SteamApp,
    Shortcut,
}

pub struct ParseError {
    // Keep `keyvalues_parser` and `keyvalues_serde` types out of the public API (this includes
    // from traits, so no using `thiserror` with `#[from]`)
    #[allow(dead_code)] // Only used for displaying currently
    inner: Box<ParseErrorInner>,
}

pub enum ParseErrorInner {
    Parse(keyvalues_parser::error::Error),
    Serde(keyvalues_serde::error::Error),
    UnexpectedStructure,
    Missing,
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
