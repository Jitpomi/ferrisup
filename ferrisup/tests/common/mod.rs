//! Common test utilities for FerrisUp
//!
//! Shared temporary-directory helpers for CLI integration tests.

use anyhow::Result;
use tempfile::TempDir;

/// Creates a temporary directory for testing
pub fn create_test_dir() -> Result<TempDir> {
    Ok(tempfile::tempdir()?)
}

/// Helper to clean up test directories
pub fn cleanup_test_dir(dir: TempDir) -> Result<()> {
    dir.close()?;
    Ok(())
}
