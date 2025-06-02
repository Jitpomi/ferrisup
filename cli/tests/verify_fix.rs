use anyhow::Result;
use tempfile::TempDir;

// Import the necessary modules from ferrisup
use ferrisup::config::{self, Components, Client};

#[test]
fn test_client_framework_mismatch() -> Result<()> {
    // This test specifically tests the scenario that was causing the "index out of bounds" panic
    // Create a test config with fewer frameworks than apps
    let mut config = config::get_default_config();
    
    // Set up client with more apps than frameworks (the problematic case)
    let mut components = Components::default();
    let client = Client {
        apps: vec!["app1".to_string(), "app2".to_string()],
        frameworks: vec!["dioxus".to_string()], // Only one framework for two apps
    };
    
    components.client = Some(client);
    config.components = components;
    
    // Write the config to a temporary directory
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path();
    
    config::write_config(&config, config_path)?;
    
    // Read it back to verify it was written correctly
    let config_file_path = config_path.join("config.json");
    assert!(config_file_path.exists(), "Config file was not created");
    
    // Verify the config contains the expected client frameworks and apps
    let content = std::fs::read_to_string(config_file_path)?;
    assert!(content.contains("dioxus"), "Config doesn't contain dioxus framework");
    assert!(content.contains("app1"), "Config doesn't contain app1");
    assert!(content.contains("app2"), "Config doesn't contain app2");
    
    // If we get here without panicking, the test passes
    println!("Successfully created config with more apps than frameworks");
    
    Ok(())
}
