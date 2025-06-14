use std::fs;
use tempfile::TempDir;
use anyhow::Result;
use shared::fs::{create_directory, copy_directory};

#[test]
fn test_create_directory_success() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().join("test_dir");
    
    // Test creating a directory
    create_directory(&test_dir)?;
    
    // Verify the directory was created
    assert!(test_dir.exists());
    assert!(test_dir.is_dir());
    
    Ok(())
}

#[test]
fn test_create_directory_nested() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let nested_dir = temp_dir.path().join("parent/child/grandchild");
    
    // Test creating a nested directory structure
    create_directory(&nested_dir)?;
    
    // Verify all directories were created
    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
    assert!(nested_dir.parent().unwrap().exists());
    assert!(nested_dir.parent().unwrap().parent().unwrap().exists());
    
    Ok(())
}

#[test]
fn test_read_cargo_toml_nonexistent() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let non_existent_dir = temp_dir.path().join("non_existent");
    
    // Try to read Cargo.toml from a non-existent directory
    let result = shared::cargo::read_cargo_toml(&non_existent_dir);
    
    // Verify the operation fails with the expected error
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_copy_directory() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let src_dir = temp_dir.path().join("src");
    let dst_dir = temp_dir.path().join("dst");
    
    // Create source directory and a file inside it
    fs::create_dir_all(&src_dir)?;
    fs::write(src_dir.join("test.txt"), "Hello, world!")?;
    
    // Test copying the directory
    copy_directory(&src_dir, &dst_dir)?;
    
    // Verify the destination directory and file were created
    assert!(dst_dir.exists());
    assert!(dst_dir.is_dir());
    assert!(dst_dir.join("test.txt").exists());
    assert_eq!(fs::read_to_string(dst_dir.join("test.txt"))?, "Hello, world!");
    
    Ok(())
}
