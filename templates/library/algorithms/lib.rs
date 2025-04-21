//! A collection of common algorithms implemented in Rust.
//!
//! This crate provides efficient implementations of various algorithms
//! for sorting, searching, and other computational tasks.

pub mod sorting;
pub mod search;

/// The main error type for algorithm operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error that occurred during algorithm execution.
    #[error("Algorithm error: {0}")]
    Algorithm(String),
    
    /// An error that occurred due to invalid input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// A specialized Result type for algorithm operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Creates a new error indicating invalid input.
pub fn invalid_input<T, S: Into<String>>(msg: S) -> Result<T> {
    Err(Error::InvalidInput(msg.into()))
}

/// Creates a new error during algorithm execution.
pub fn algorithm_error<T, S: Into<String>>(msg: S) -> Result<T> {
    Err(Error::Algorithm(msg.into()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Basic test to verify everything compiles
        assert_eq!(2 + 2, 4);
    }
}
