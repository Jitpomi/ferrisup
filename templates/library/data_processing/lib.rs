//! A data processing library crate.
//!
//! This crate provides functionality for processing and transforming data.

mod processor;

pub use processor::*;

/// The main error type for data processing operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error that occurred during data deserialization.
    #[error("Failed to deserialize data: {0}")]
    Deserialization(String),
    
    /// An error that occurred during data processing.
    #[error("Failed to process data: {0}")]
    Processing(String),
    
    /// An error that occurred during data serialization.
    #[error("Failed to serialize data: {0}")]
    Serialization(String),
}

/// A specialized Result type for data processing operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        // Add your tests here
    }
}
