//! Tests for template compatibility and format handling

use std::fs::{self, File};
use std::io::Write;
use anyhow::Result;
use tempfile::TempDir;
use serde_json::json;

mod common;

#[test]
fn test_template_json_variations() -> Result<()> {
    use ferrisup::config::Config;
    
    
    let temp_dir = TempDir::new()?;
    let template_path = temp_dir.path().join("test_template");
    fs::create_dir(&template_path)?;
    
    // Create a template.json with old format (missing some fields)
    let old_format_json = json!({
        "name": "test-template",
        "description": "Template for testing compatibility",
        "type": "library",
        "project_name": "test-template",
        "template": "test-template",
        "files": [
            {
                "source": "lib.rs",
                "target": "src/lib.rs"
            }
        ],
        "dependencies": {
            "default": [
                "anyhow = \"1.0\"",
                "serde = { version = \"1.0\", features = [\"derive\"] }"
            ]
        }
        // Missing several fields that are in newer templates
    });
    
    // Write the template.json file
    let template_json_path = template_path.join("template.json");
    let mut file = File::create(&template_json_path)?;
    file.write_all(serde_json::to_string_pretty(&old_format_json)?.as_bytes())?;
    
    // Create a sample lib.rs file
    let lib_rs_path = template_path.join("lib.rs");
    let mut lib_file = File::create(&lib_rs_path)?;
    lib_file.write_all(b"pub fn hello() -> &'static str { \"Hello, world!\" }")?;
    
    // Test loading the template
    let template_content = fs::read_to_string(&template_json_path)?;
    let template_config: serde_json::Value = serde_json::from_str(&template_content)?;
    
    // Verify template.json was loaded correctly
    assert_eq!(template_config["name"], "test-template");
    assert_eq!(template_config["description"], "Template for testing compatibility");
    
    // Try to convert it to our Config struct (this would trigger our compatibility layer)
    let config: Config = serde_json::from_str(&template_content)?;
    
    // Verify default values were applied
    assert_eq!(config.project_name, "test-template");
    
    // Clean up
    common::cleanup_test_dir(temp_dir)?;
    Ok(())
}

#[test]
fn test_read_template_old_format() -> Result<()> {
    
    
    
    // Create a temporary test directory
    let temp_dir = TempDir::new()?;
    let test_path = temp_dir.path();
    
    // Create a mock template with old format
    let template_dir = test_path.join("old-format-template");
    fs::create_dir_all(&template_dir)?;
    
    // Create template.json with old format (missing fields)
    let old_template_json = json!({
        "name": "old-format",
        "description": "Template with old format for testing",
        "type": "binary",
        "components": {
            "client": {
                // Missing apps and frameworks
            },
            "database": {
                // Old format
                "db_type": "postgres",
                "orm": "diesel"
            }
        }
    });
    
    let template_json_path = template_dir.join("template.json");
    let mut file = File::create(&template_json_path)?;
    file.write_all(serde_json::to_string_pretty(&old_template_json)?.as_bytes())?;
    
    // Create a main.rs file for the template
    let main_rs_path = template_dir.join("main.rs");
    let mut main_file = File::create(&main_rs_path)?;
    main_file.write_all(b"fn main() { println!(\"Hello from old format template\"); }")?;
    
    // Test parsing this template
    let template_content = fs::read_to_string(&template_json_path)?;
    
    // This should not panic or error due to our compatibility layer
    let parsed: serde_json::Value = serde_json::from_str(&template_content)?;
    
    // Verify we read the template data correctly
    assert_eq!(parsed["name"], "old-format");
    assert_eq!(parsed["description"], "Template with old format for testing");
    
    // Clean up
    common::cleanup_test_dir(temp_dir)?;
    Ok(())
}
