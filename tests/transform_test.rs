use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use anyhow::Result;

fn setup_test_project() -> Result<(TempDir, PathBuf)> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let project_dir = temp_dir.path().join("test_project");
    
    // Create basic project structure
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("src"))?;
    
    // Create a basic Cargo.toml
    let cargo_content = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_content)?;
    
    // Create a basic main.rs
    let main_content = r#"fn main() {
    println!("Hello, world!");
}
"#;
    fs::write(project_dir.join("src").join("main.rs"), main_content)?;
    
    Ok((temp_dir, project_dir))
}

#[test]
fn test_error_handling_in_utils() -> Result<()> {
    // Test that our error handling in utils module works correctly
    let (_temp_dir, project_dir) = setup_test_project()?;
    
    // Create a completely invalid file instead of just invalid TOML
    fs::remove_file(project_dir.join("Cargo.toml"))?;
    
    // Test reading a non-existent Cargo.toml
    let result = ferrisup::utils::read_cargo_toml(&project_dir);
    
    // Verify the operation fails with an error
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Cargo.toml not found"), 
            "Error should mention 'Cargo.toml not found'");
    
    Ok(())
}

#[test]
fn test_error_handling_with_nonexistent_directory() {
    // Test handling of non-existent directories
    let non_existent_dir = PathBuf::from("/non/existent/path");
    
    // Try to create a directory inside a non-existent path
    let result = ferrisup::utils::create_directory(&non_existent_dir.join("test"));
    
    // Verify the operation fails with an error
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Failed to create directory") || error.contains("No such file or directory"), 
            "Error should mention directory creation failure");
}

#[test]
fn test_error_handling_with_invalid_workspace_members() -> Result<()> {
    // Test handling of invalid workspace members
    let (_temp_dir, project_dir) = setup_test_project()?;
    
    // Create an invalid workspace Cargo.toml
    let invalid_workspace_content = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"

[workspace]
members = [
    "invalid-path",
]
"#;
    fs::write(project_dir.join("Cargo.toml"), invalid_workspace_content)?;
    
    // Try to update workspace members
    let result = ferrisup::utils::update_workspace_members(&project_dir);
    
    // This should succeed even with invalid members, as it just updates the list
    assert!(result.is_ok());
    
    // Check that the Cargo.toml was updated
    let updated_content = fs::read_to_string(project_dir.join("Cargo.toml"))?;
    assert!(updated_content.contains("members"));
    
    Ok(())
}

#[test]
fn test_error_handling_with_file_operations() -> Result<()> {
    // Test handling of file operation errors
    let (_temp_dir, project_dir) = setup_test_project()?;
    
    // Make the src directory read-only to cause write errors
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let src_dir = project_dir.join("src");
        let metadata = fs::metadata(&src_dir)?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o444); // read-only
        fs::set_permissions(&src_dir, perms)?;
        
        // Try to write to a file in the read-only directory
        let result = fs::write(src_dir.join("new_file.rs"), "test content");
        
        // Verify the operation fails with an error
        assert!(result.is_err());
        
        // Reset permissions
        let mut perms = metadata.permissions();
        perms.set_mode(0o755); // rwx for owner, rx for group and others
        fs::set_permissions(&src_dir, perms)?;
    }
    
    Ok(())
}
