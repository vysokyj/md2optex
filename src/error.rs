use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    TomlParse(toml::de::Error),
    MissingChaptersDir(PathBuf),
    HyphenationDict(PathBuf, io::Error),
    #[allow(dead_code)] // reserved for future style-system implementation
    StyleNotFound(String),
    OptexNotFound,
    OptexFailed(i32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::TomlParse(e) => write!(f, "Error in metadata.toml: {e}"),
            Error::MissingChaptersDir(p) => {
                write!(f, "Chapters directory not found: {}", p.display())
            }
            Error::HyphenationDict(p, e) => {
                write!(
                    f,
                    "Cannot read hyphenation dictionary '{}': {e}",
                    p.display()
                )
            }
            Error::StyleNotFound(s) => write!(f, "Style '{s}' not found"),
            Error::OptexNotFound => write!(
                f,
                "`optex` not found in PATH — install OpTeX: https://petr.olsak.net/optex/"
            ),
            Error::OptexFailed(code) => write!(f, "optex failed with exit code {code}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::TomlParse(e)
    }
}
