use std::fs;
use tempfile::TempDir;
use anyhow::Result;

#[test]
fn test_create_directory_success() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().join("test_dir");
    
    // Test creating a directory
    ferrisup::utils::create_directory(&test_dir)?;
    
    // Verify the directory was created
    assert!(test_dir.exists());
    assert!(test_dir.is_dir());
    
    Ok(())
}

#[test]
fn test_create_directory_nested() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let nested_dir = temp_dir.path().join("parent").join("child").join("grandchild");
    
    // Test creating nested directories
    ferrisup::utils::create_directory(&nested_dir)?;
    
    // Verify all directories were created
    assert!(nested_dir.exists());
    assert!(nested_dir.is_dir());
    
    Ok(())
}

#[test]
fn test_read_cargo_toml_missing_file() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let non_existent_dir = temp_dir.path().join("non_existent");
    
    // Test reading a non-existent Cargo.toml
    let result = ferrisup::utils::read_cargo_toml(&non_existent_dir);
    
    // Verify the operation fails with an error
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Cargo.toml"), "Error should mention Cargo.toml");
}

#[test]
fn test_write_cargo_toml_content() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().join("test_project");
    fs::create_dir_all(&test_dir)?;
    
    // Test content to write
    let content = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#;
    
    // Write the content
    ferrisup::utils::write_cargo_toml_content(&test_dir, content)?;
    
    // Verify the file was created with correct content
    let cargo_path = test_dir.join("Cargo.toml");
    assert!(cargo_path.exists());
    let read_content = fs::read_to_string(cargo_path)?;
    assert_eq!(read_content, content);
    
    Ok(())
}

#[test]
fn test_copy_directory() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let src_dir = temp_dir.path().join("src");
    let dst_dir = temp_dir.path().join("dst");
    
    // Create source directory with some files
    fs::create_dir_all(&src_dir)?;
    fs::write(src_dir.join("file1.txt"), "content1")?;
    fs::create_dir_all(src_dir.join("subdir"))?;
    fs::write(src_dir.join("subdir").join("file2.txt"), "content2")?;
    
    // Copy the directory
    ferrisup::utils::copy_directory(&src_dir, &dst_dir)?;
    
    // Verify the directory was copied with all files
    assert!(dst_dir.exists());
    assert!(dst_dir.join("file1.txt").exists());
    assert!(dst_dir.join("subdir").exists());
    assert!(dst_dir.join("subdir").join("file2.txt").exists());
    
    // Verify file contents
    assert_eq!(fs::read_to_string(dst_dir.join("file1.txt"))?, "content1");
    assert_eq!(fs::read_to_string(dst_dir.join("subdir").join("file2.txt"))?, "content2");
    
    Ok(())
}
