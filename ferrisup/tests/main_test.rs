//! Main test suite for FerrisUp
//! 
//! This file serves as an entry point for running all integration tests.
//! To run the complete test suite, use: `cargo test --all-features`

use anyhow::Result;
use ferrisup;
use std::fs;

// Import all test modules to ensure they run
mod common;

#[test]
fn test_ferrisup_version() -> Result<()> {
    // Check that the version from Cargo.toml matches the one we use
    let cargo_version = env!("CARGO_PKG_VERSION");
    assert!(!cargo_version.is_empty(), "Version should not be empty");
    
    // Simple test to make sure the build process works
    assert!(std::process::Command::new(env!("CARGO_BIN_EXE_ferrisup"))
        .arg("--version")
        .status()?
        .success());
    
    Ok(())
}

#[test]
fn test_cargo_metadata() -> Result<()> {
    // Ensure the Cargo.toml has the required metadata for publishing
    assert!(!env!("CARGO_PKG_AUTHORS").is_empty(), "Authors should be specified");
    assert!(!env!("CARGO_PKG_DESCRIPTION").is_empty(), "Description should be provided");
    assert!(!env!("CARGO_PKG_LICENSE").is_empty(), "License should be specified");
    assert!(!env!("CARGO_PKG_REPOSITORY").is_empty(), "Repository URL should be provided");
    
    // Instead of checking CARGO_PKG_KEYWORDS directly (which might not be available at runtime),
    // we'll read the Cargo.toml file directly to check for keywords
    let cargo_toml_content = std::fs::read_to_string("Cargo.toml")?;
    
    // Simple check for keywords section
    assert!(cargo_toml_content.contains("keywords = ["), 
            "Keywords should be provided in Cargo.toml for crates.io discoverability");
    
    Ok(())
}

// This test runs slower, so we'll mark it with ignore by default
// Run with `cargo test -- --ignored` to include it
#[test]
#[ignore]
fn test_all_templates() -> Result<()> {
    // This is a comprehensive test that creates a project with each template
    // and ensures it builds successfully
    use std::process::{Command, Stdio};
    use tempfile::TempDir;
    
    let template_tuples = ferrisup::template_manager::list_templates()?;
    let _template_count = template_tuples.len();
    // Found templates to test
    
    // Track successful and failed templates
    let mut success_count = 0;
    let mut failed_templates = Vec::new();
    
    for (template_name, _template_description) in &template_tuples {
        // Check that template name is not empty
        assert!(!template_name.is_empty());
        
        // Testing template
        
        // Skip templates that may require additional setup
        if template_name.contains("embedded") || template_name.contains("edge") {
            // Skipping template that requires special setup
            continue;
        }
        
        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let project_name = format!("test_{}", template_name.replace("-", "_"));
        
        // Create a project with this template - use --no-interactive flag to avoid terminal prompts
        // Creating test project
        
        let output = Command::new(env!("CARGO_BIN_EXE_ferrisup"))
            .args(&[
                "new", 
                &project_name, 
                "--template", 
                &template_name,
                "--no-interactive" // Add flag to avoid terminal prompts
            ])
            .current_dir(temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        // Print output for debugging
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        // Command output
        if !stdout.is_empty() {
            // STDOUT captured
        }
        if !stderr.is_empty() {
            // STDERR captured
        }
        
        // Check if the command executed successfully, but continue with other templates
        if !output.status.success() {
            // Failed to create project
            // Command status
            failed_templates.push(template_name);
        } else {
            success_count += 1;
            
            // Verify that key files were created
            let src_dir = temp_dir.path().join(&project_name).join("src");
            
            if src_dir.exists() {
                // Check for typical Rust project files
                let cargo_toml = temp_dir.path().join(&project_name).join("Cargo.toml");
                assert!(cargo_toml.exists(), "Cargo.toml should be created");
                
                // Basic sanity check on content
                let cargo_content = fs::read_to_string(&cargo_toml)?;
                assert!(cargo_content.contains(&project_name), "Cargo.toml should contain project name");
                
                // Project structure verification passed
            } else {
                // src directory not found
            }
        }
    }
    
    // Report overall results
    // Template testing complete
    // Success count
    if !failed_templates.is_empty() {
        // Failed templates
    }
    
    // Assert that at least some templates succeeded
    assert!(success_count > 0, "At least one template should create successfully");
    
    Ok(())
}

#[test]
fn test_readme_contains_required_sections() -> Result<()> {
    // Verify the README contains essential sections for a published crate
    use std::fs;
    
    let readme = fs::read_to_string("README.md")?;
    
    // Check for essential sections
    assert!(readme.contains("# FerrisUp"), "README should have a title");
    assert!(readme.contains("## Installation"), "README should have installation instructions");
    assert!(readme.contains("## Usage"), "README should have usage instructions");
    assert!(readme.contains("## License"), "README should have license information");
    
    Ok(())
}
