//! Common test utilities for FerrisUp
//! 
//! These utilities are maintained for test infrastructure even if not currently used
//! by all tests. They provide a consistent testing approach for current and future tests.

use std::string::String;
use tempfile::TempDir;
use anyhow::Result;

/// Creates a temporary directory for testing
/// 
/// This function is maintained as part of the test infrastructure for current
/// and future test development.
#[allow(dead_code)]
pub fn create_test_dir() -> Result<TempDir> {
    Ok(tempfile::tempdir()?)
}

/// Helper to clean up test directories
/// 
/// This function is maintained as part of the test infrastructure for current
/// and future test development.
#[allow(dead_code)]
pub fn cleanup_test_dir(dir: TempDir) -> Result<()> {
    dir.close()?;
    Ok(())
}

/// Helper to capture CLI output for verification in tests
/// 
/// This struct is maintained as part of the test infrastructure for current
/// and future test development.
#[allow(dead_code)]
pub struct OutputCapture {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl OutputCapture {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
    
    #[allow(dead_code)]
    pub fn get_stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }
    
    #[allow(dead_code)]
    pub fn get_stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).to_string()
    }
}
