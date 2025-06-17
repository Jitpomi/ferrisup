
// Shared component functionality is available through the shared crate
// Uncomment if needed: use ferrisup_common::*;
/// FerrisUp - A versatile Rust project bootstrapping tool
/// 
/// This crate provides a CLI tool for bootstrapping and managing Rust projects
/// with various templates and configurations.
/// 
/// # Examples
/// 
/// ```bash
/// # Create a new minimal Rust project
/// ferrisup new my_project --template minimal
/// 
/// # List available templates
/// ferrisup list
/// 
/// # Preview a template
/// ferrisup preview --template full-stack
/// ```
// Core modules
pub mod core;

// Project management modules
pub mod project;
pub mod template_manager;

// CLI command modules
pub mod commands;

// Re-exports of frequently used components
pub use core::{Config, Result, Error};
pub use project::{find_handler, get_handlers, ProjectHandler};
