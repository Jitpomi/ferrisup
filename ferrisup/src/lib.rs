/// FerrisUp creates and evolves coherent, inspectable Rust project foundations.
///
/// This crate provides a CLI tool for bootstrapping and managing Rust projects
/// with various templates and configurations.
///
/// # Examples
///
/// ```bash
/// # Create a new minimal Rust project
/// ferrisup new my_project --component-type minimal --no-interactive
///
/// # List available templates
/// ferrisup list
///
/// # Preview a template
/// ferrisup preview --component-type server --framework axum
/// ```
// Core modules
pub mod core;

// Project management modules
pub mod project;

// CLI command modules
pub mod commands;

// Re-exports of frequently used components
pub use core::{Config, Error, Result};
pub use project::{ProjectHandler, find_handler, get_handlers};
