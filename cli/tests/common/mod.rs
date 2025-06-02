//! Common test utilities for FerrisUp

use std::string::String;
use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;
use anyhow::Result;

/// Creates a temporary directory for testing
/// Used in config_test.rs, commands_test.rs, and template_compatibility_test.rs
pub fn create_test_dir() -> Result<TempDir> {
    Ok(tempfile::tempdir()?)
}

/// Helper to clean up test directories
/// Used in config_test.rs, commands_test.rs, and template_compatibility_test.rs
pub fn cleanup_test_dir(dir: TempDir) -> Result<()> {
    dir.close()?;
    Ok(())
}

/// Helper to capture CLI output for verification in tests
/// Used in main_test.rs and commands_test.rs
pub struct OutputCapture {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl OutputCapture {
    pub fn new() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
    
    pub fn get_stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }
    
    pub fn get_stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).to_string()
    }
}
