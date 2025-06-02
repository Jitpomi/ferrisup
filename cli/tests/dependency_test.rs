use anyhow::Result;
use std::fs;
use tempfile::TempDir;

mod common;

// Test that the dependency command properly handles errors with missing Cargo.toml
#[test]
fn test_missing_cargo_toml() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Test with a non-existent directory
    let non_existent_dir = temp_path.join("non_existent");
    fs::create_dir_all(&non_existent_dir)?;
    
    // This should fail because there's no Cargo.toml
    let result = ferrisup::commands::dependency::add_dependencies(
        ferrisup::commands::dependency::AddArgs {
            dependencies: vec!["anyhow".to_string()],
            dev: false,
            features: None,
            version: None,
            path: Some(non_existent_dir.clone()),
        }
    );
    
    assert!(result.is_err(), "Expected an error when adding dependencies to a directory without Cargo.toml");
    
    // Clean up
    temp_dir.close()?;
    
    Ok(())
}

// Test that the dependency command properly handles invalid TOML
#[test]
fn test_invalid_cargo_toml() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Create a project directory with an invalid Cargo.toml
    let project_dir = temp_path.join("invalid_toml");
    fs::create_dir_all(&project_dir)?;
    
    let invalid_cargo_toml = r#"
[package]
name = "invalid_toml"
version = "0.1.0"
edition = 2021  # Missing quotes around string

[dependencies
"#;  // Missing closing bracket
    
    fs::write(project_dir.join("Cargo.toml"), invalid_cargo_toml)?;
    
    // Test removing a dependency from an invalid Cargo.toml
    let result = ferrisup::commands::dependency::remove_dependencies(
        ferrisup::commands::dependency::RemoveArgs {
            dependencies: vec!["some_dependency".to_string()],
            path: Some(project_dir.clone()),
        }
    );
    
    // This should fail because the TOML is invalid
    assert!(result.is_err(), "Expected an error when parsing invalid TOML");
    
    // Clean up
    temp_dir.close()?;
    
    Ok(())
}
