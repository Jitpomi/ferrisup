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
pub mod utils;
pub mod config;
pub mod commands;
pub mod template_manager;
