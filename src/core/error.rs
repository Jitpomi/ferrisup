// Error types for FerrisUp
use std::fmt;
use std::io;

/// Custom Result type for FerrisUp operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for FerrisUp operations
#[derive(Debug)]
pub enum Error {
    /// I/O error
    Io(io::Error),
    /// Template error
    Template(String),
    /// Project handler error
    Handler(String),
    /// Config error
    Config(String),
    /// Other error
    Other(String),
    /// Anyhow error (for compatibility)
    Anyhow(anyhow::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::Template(msg) => write!(f, "Template error: {}", msg),
            Error::Handler(msg) => write!(f, "Project handler error: {}", msg),
            Error::Config(msg) => write!(f, "Config error: {}", msg),
            Error::Other(msg) => write!(f, "{}", msg),
            Error::Anyhow(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Anyhow(err)
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Other(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Other(msg.to_string())
    }
}
