//! Tests for the config module

use std::fs;
use anyhow::Result;
use serde_json::json;

use ferrisup::config::{Config, Components, Client, Server, Database};

mod common;

#[test]
fn test_default_config() -> Result<()> {
    // Test the default configuration
    let config = ferrisup::config::get_default_config();
    
    // Verify the default project name and template
    assert_eq!(config.project_name, "rust_workspace");
    assert_eq!(config.template, "minimal");
    
    // Verify components exist
    assert!(config.components.client.is_some());
    assert!(config.components.server.is_some());
    assert!(config.components.database.is_some());
    
    Ok(())
}

#[test]
fn test_config_serialization() -> Result<()> {
    // Create a test config
    let config = Config {
        project_name: "test_project".to_string(),
        template: "minimal".to_string(),
        components: Components {
            client: Some(Client {
                apps: vec!["web".to_string()],
                frameworks: vec!["dioxus".to_string()],
            }),
            server: Some(Server {
                services: vec!["api".to_string()],
                frameworks: vec!["axum".to_string()],
            }),
            database: Some(Database {
                enabled: true,
                engines: vec!["sqlite".to_string()],
                migration_tool: "sqlx".to_string(),
            }),
            libs: None,
            binaries: None,
            ai: None,
            edge: None,
            embedded: None,
        },
    };
    
    // Serialize the config
    let json_str = serde_json::to_string_pretty(&config)?;
    
    // Deserialize and verify it matches
    let deserialized: Config = serde_json::from_str(&json_str)?;
    assert_eq!(deserialized.project_name, "test_project");
    assert_eq!(deserialized.template, "minimal");
    
    // Check component details
    if let Some(client) = &deserialized.components.client {
        assert_eq!(client.apps[0], "web");
        assert_eq!(client.frameworks[0], "dioxus");
    } else {
        panic!("Client component missing");
    }
    
    if let Some(server) = &deserialized.components.server {
        assert_eq!(server.services[0], "api");
        assert_eq!(server.frameworks[0], "axum");
    } else {
        panic!("Server component missing");
    }
    
    if let Some(db) = &deserialized.components.database {
        assert!(db.enabled);
        assert_eq!(db.engines[0], "sqlite");
        assert_eq!(db.migration_tool, "sqlx");
    } else {
        panic!("Database component missing");
    }
    
    Ok(())
}

#[test]
fn test_config_write_read() -> Result<()> {
    let temp_dir = common::create_test_dir()?;
    let config_path = temp_dir.path();
    
    // Create a test config
    let config = Config {
        project_name: "test_project".to_string(),
        template: "minimal".to_string(),
        components: Components::default(),
    };
    
    // Write config to disk
    ferrisup::config::write_config(&config, config_path)?;
    
    // Check that the file exists
    let config_file_path = config_path.join("config.json");
    assert!(config_file_path.exists());
    
    // Read the config file directly and verify content
    let config_content = fs::read_to_string(&config_file_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&config_content)?;
    
    assert_eq!(parsed["project_name"], "test_project");
    assert_eq!(parsed["template"], "minimal");
    
    common::cleanup_test_dir(temp_dir)?;
    Ok(())
}

#[test]
fn test_convert_old_template() -> Result<()> {
    use ferrisup::config::{Config, Components, AI, convert_old_template};
    
    // Create a config with old format (missing frameworks in AI)
    let mut config = Config {
        project_name: "old_template_test".to_string(),
        template: "gen-ai".to_string(),
        components: Components {
            client: None,
            server: None,
            database: None,
            libs: None,
            binaries: None,
            ai: Some(AI {
                models: vec!["gpt2".to_string(), "llama".to_string()],
                frameworks: vec![], // Empty frameworks, should be populated by convert_old_template
            }),
            edge: None,
            embedded: None,
        },
    };
    
    // Apply the compatibility conversion
    convert_old_template(&mut config);
    
    // Verify that frameworks is now populated for AI component
    if let Some(ai) = &config.components.ai {
        assert!(!ai.frameworks.is_empty(), "AI frameworks should be populated");
        assert!(ai.frameworks.contains(&"tract".to_string()), "Should contain tract framework");
    } else {
        panic!("AI component should exist");
    }
    
    Ok(())
}

#[test]
fn test_config_with_missing_fields() -> Result<()> {
    use ferrisup::config::Config;
    use serde_json::json;
    
    // Test parsing a JSON with missing fields
    let incomplete_json = json!({
        "project_name": "incomplete_test",
        "template": "minimal",
        "components": {
            "client": {
                // Missing 'apps' and 'frameworks'
            },
            "ai": {
                "models": ["gpt2"]
                // Missing 'frameworks'
            }
        }
    });
    
    // Parse the incomplete JSON into our Config struct
    let config_str = serde_json::to_string(&incomplete_json)?;
    let config: Config = serde_json::from_str(&config_str)?;
    
    // Verify default values were applied
    if let Some(client) = &config.components.client {
        assert!(client.apps.is_empty(), "Client apps should be empty by default");
        assert!(client.frameworks.is_empty(), "Client frameworks should be empty by default");
    } else {
        panic!("Client component should exist");
    }
    
    if let Some(ai) = &config.components.ai {
        assert_eq!(ai.models, vec!["gpt2"], "AI models should match the input");
        assert!(ai.frameworks.is_empty(), "AI frameworks should be empty by default");
    } else {
        panic!("AI component should exist");
    }
    
    Ok(())
}
