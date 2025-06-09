// Core workspace entry point
pub mod config;
pub mod error;

// Re-exports of core components
pub use config::Config;
pub use error::{Error, Result};
