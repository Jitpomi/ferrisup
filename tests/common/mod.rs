//! Common test utilities for FerrisUp

use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;
use anyhow::Result;

/// Creates a temporary directory for testing
pub fn create_test_dir() -> Result<TempDir> {
    Ok(tempfile::tempdir()?)
}

/// Creates a mock project structure for testing
pub fn create_mock_project(test_dir: &Path, template: &str) -> Result<PathBuf> {
    let project_path = test_dir.join("test_project");
    fs::create_dir_all(&project_path)?;
    
    // Create a minimal structure based on template
    fs::create_dir_all(project_path.join("src"))?;
    
    match template {
        "minimal" => {
            fs::write(
                project_path.join("src").join("main.rs"),
                r#"fn main() {
    println!("Hello, world!");
}"#,
            )?;
        },
        "library" => {
            fs::write(
                project_path.join("src").join("lib.rs"),
                r#"pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}"#,
            )?;
        },
        _ => {
            // For other templates just create an empty main.rs
            fs::write(
                project_path.join("src").join("main.rs"),
                r#"fn main() {
    println!("Hello from test project!");
}"#,
            )?;
        }
    }
    
    // Create a basic Cargo.toml
    fs::write(
        project_path.join("Cargo.toml"),
        format!(r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#),
    )?;
    
    Ok(project_path)
}

/// Helper to clean up test directories
pub fn cleanup_test_dir(dir: TempDir) -> Result<()> {
    dir.close()?;
    Ok(())
}

/// Captures CLI output for verification in tests
#[macro_export]
macro_rules! assert_cmd_output {
    ($cmd:expr, $expected:expr) => {
        let output = $cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        assert!(stdout.contains($expected), "Expected '{}' in command output: '{}'", $expected, stdout);
    };
}
