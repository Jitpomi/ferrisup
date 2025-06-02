use anyhow::Result;
use tempfile::TempDir;

// Import the necessary modules from ferrisup
use ferrisup::config::{self, Components, Client, Database};

#[test]
fn test_config_with_client_frameworks() -> Result<()> {
    // Create a test config with client frameworks
    let mut config = config::get_default_config();
    
    // Set up client with frameworks
    let mut components = Components::default();
    let client = Client {
        apps: vec!["app1".to_string()],
        frameworks: vec!["dioxus".to_string(), "tauri".to_string()],
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
    
    // Verify the config contains the expected client frameworks
    let content = std::fs::read_to_string(config_file_path)?;
    assert!(content.contains("dioxus"), "Config doesn't contain dioxus framework");
    assert!(content.contains("tauri"), "Config doesn't contain tauri framework");
    
    Ok(())
}

#[test]
fn test_config_with_multiple_database_types() -> Result<()> {
    // Create a test config with database settings
    let mut config = config::get_default_config();
    
    // Set up database configuration
    let mut components = Components::default();
    let database = Database {
        enabled: true,
        engines: vec!["postgresql".to_string()],
        migration_tool: "diesel".to_string(),
        cache_engine: Some("redis".to_string()),
        vector_engine: Some("pinecone".to_string()),
        graph_engine: Some("neo4j".to_string()),
    };
    
    components.database = Some(database);
    config.components = components;
    
    // Write the config to a temporary directory
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path();
    
    config::write_config(&config, config_path)?;
    
    // Read it back to verify it was written correctly
    let config_file_path = config_path.join("config.json");
    assert!(config_file_path.exists(), "Config file was not created");
    
    // Verify the config contains the expected database engines
    let content = std::fs::read_to_string(config_file_path)?;
    assert!(content.contains("postgresql"), "Config doesn't contain PostgreSQL");
    assert!(content.contains("redis"), "Config doesn't contain Redis");
    assert!(content.contains("pinecone"), "Config doesn't contain Pinecone");
    assert!(content.contains("neo4j"), "Config doesn't contain Neo4j");
    
    Ok(())
}
