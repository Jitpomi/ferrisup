use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project_name: String,
    pub template: String,
    #[serde(default)]
    pub components: Components,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    #[serde(default)]
    pub client: Option<Client>,
    #[serde(default)]
    pub server: Option<Server>,
    #[serde(default)]
    pub database: Option<Database>,
    #[serde(default)]
    pub libs: Option<Libs>,
    #[serde(default)]
    pub binaries: Option<Binaries>,
    #[serde(default)]
    pub ai: Option<AI>,
    #[serde(default)]
    pub edge: Option<Edge>,
    #[serde(default)]
    pub embedded: Option<Embedded>,
}

impl Default for Components {
    fn default() -> Self {
        Self {
            client: None,
            server: None,
            database: None,
            libs: None,
            binaries: None,
            ai: None,
            edge: None,
            embedded: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    #[serde(default)]
    pub apps: Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            apps: vec![],
            frameworks: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            services: vec![],
            frameworks: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub engines: Vec<String>,
    #[serde(default)]
    pub migration_tool: String,
    #[serde(default)]
    pub cache_engine: Option<String>,
    #[serde(default)]
    pub vector_engine: Option<String>,
    #[serde(default)]
    pub graph_engine: Option<String>,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            enabled: false,
            engines: vec![],
            migration_tool: "".to_string(),
            cache_engine: None,
            vector_engine: None,
            graph_engine: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Libs {
    #[serde(default)]
    pub modules: Vec<String>,
}

impl Default for Libs {
    fn default() -> Self {
        Self {
            modules: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binaries {
    #[serde(default)]
    pub apps: Vec<String>,
}

impl Default for Binaries {
    fn default() -> Self {
        Self {
            apps: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI {
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub frameworks: Vec<String>,
}

impl Default for AI {
    fn default() -> Self {
        Self {
            models: vec![],
            frameworks: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    #[serde(default)]
    pub apps: Vec<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
}

impl Default for Edge {
    fn default() -> Self {
        Self {
            apps: vec![],
            platforms: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedded {
    #[serde(default)]
    pub devices: Vec<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
}

impl Default for Embedded {
    fn default() -> Self {
        Self {
            devices: vec![],
            platforms: vec![],
        }
    }
}

pub fn get_config_path() -> Result<String> {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    Ok(format!("{}/config.json", cargo_manifest_dir))
}

pub fn read_config() -> Result<Config> {
    let config_path = get_config_path()?;
    
    let config_content = fs::read_to_string(&config_path)
        .context(format!("Failed to read config file: {}", config_path))?;
    
    let mut config: Config = serde_json::from_str(&config_content)
        .context("Failed to parse config.json")?;
    
    // Apply compatibility conversions for old template formats
    convert_old_template(&mut config);
    
    Ok(config)
}

pub fn write_config(config: &Config, path: &Path) -> Result<()> {
    let config_str = serde_json::to_string_pretty(config)
        .context("Failed to serialize config to JSON")?;
    
    fs::write(path.join("config.json"), config_str)
        .context("Failed to write config.json")?;
    
    Ok(())
}

pub fn get_default_config() -> Config {
    Config {
        project_name: "rust_workspace".to_string(),
        template: "minimal".to_string(),
        components: Components {
            client: Some(Client {
                apps: vec!["app1".to_string(), "app2".to_string()],
                frameworks: vec!["dioxus".to_string(), "dioxus".to_string()],
            }),
            server: Some(Server {
                services: vec!["api".to_string(), "auth".to_string()],
                frameworks: vec!["axum".to_string(), "axum".to_string()],
            }),
            database: Some(Database {
                enabled: true,
                engines: vec!["postgres".to_string()],
                migration_tool: "sea-orm".to_string(),
                cache_engine: Some("redis".to_string()),
                vector_engine: Some("pinecone".to_string()),
                graph_engine: Some("neo4j".to_string()),
            }),
            libs: Some(Libs {
                modules: vec!["core".to_string(), "models".to_string(), "utils".to_string()],
            }),
            binaries: Some(Binaries {
                apps: vec!["cli".to_string()],
            }),
            ai: Some(AI {
                models: vec!["inference".to_string()],
                frameworks: vec!["tract".to_string()],
            }),
            edge: Some(Edge {
                apps: vec!["worker".to_string()],
                platforms: vec!["cloudflare".to_string()],
            }),
            embedded: Some(Embedded {
                devices: vec!["device".to_string()],
                platforms: vec!["rp2040".to_string()],
            }),
        },
    }
}

pub fn convert_old_template(config: &mut Config) {
    if let Some(ai) = config.components.ai.as_mut() {
        if ai.frameworks.is_empty() {
            ai.frameworks = vec!["tract".to_string()];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_components_default() {
        let components = Components::default();
        assert!(components.client.is_none());
        assert!(components.server.is_none());
        assert!(components.database.is_none());
        assert!(components.libs.is_none());
        assert!(components.binaries.is_none());
        assert!(components.ai.is_none());
        assert!(components.edge.is_none());
        assert!(components.embedded.is_none());
    }
    
    #[test]
    fn test_get_default_config() {
        let config = get_default_config();
        
        // Check basic properties
        assert_eq!(config.project_name, "rust_workspace");
        assert_eq!(config.template, "minimal");
        
        // Check client component
        assert!(config.components.client.is_some(), "Client component should be present");
        let client = config.components.client.as_ref().expect("Client component should be present");
        assert_eq!(client.apps.len(), 2);
        assert_eq!(client.apps[0], "app1");
        assert_eq!(client.frameworks[0], "dioxus");
        
        // Check server component
        assert!(config.components.server.is_some(), "Server component should be present");
        let server = config.components.server.as_ref().expect("Server component should be present");
        assert_eq!(server.services.len(), 2);
        assert_eq!(server.services[0], "api");
        assert_eq!(server.frameworks[0], "axum");
        
        // Check database component
        assert!(config.components.database.is_some(), "Database component should be present");
        let db = config.components.database.as_ref().expect("Database component should be present");
        assert!(db.enabled);
        assert_eq!(db.engines.len(), 1);
        assert_eq!(db.engines[0], "postgres");
        assert_eq!(db.migration_tool, "sea-orm");
        assert!(db.cache_engine.is_some(), "Cache engine should be present");
        assert_eq!(db.cache_engine.as_ref().expect("Cache engine should be present"), "redis");
        assert!(db.vector_engine.is_some(), "Vector engine should be present");
        assert_eq!(db.vector_engine.as_ref().expect("Vector engine should be present"), "pinecone");
        assert!(db.graph_engine.is_some(), "Graph engine should be present");
        assert_eq!(db.graph_engine.as_ref().expect("Graph engine should be present"), "neo4j");
        
        // Check libs component
        assert!(config.components.libs.is_some(), "Libs component should be present");
        let libs = config.components.libs.as_ref().expect("Libs component should be present");
        assert_eq!(libs.modules.len(), 3);
        assert!(libs.modules.contains(&"core".to_string()));
        assert!(libs.modules.contains(&"models".to_string()));
        assert!(libs.modules.contains(&"utils".to_string()));
    }
    
    #[test]
    fn test_write_and_read_config() -> Result<()> {
        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        
        // Create a simple config
        let config = Config {
            project_name: "test_project".to_string(),
            template: "minimal".to_string(),
            components: Components::default(),
        };
        
        // Write the config
        write_config(&config, temp_path)?;
        
        // Check the file exists
        let config_file = temp_path.join("config.json");
        assert!(config_file.exists());
        
        // Read the contents directly and check
        let content = fs::read_to_string(&config_file)?;
        let parsed: serde_json::Value = serde_json::from_str(&content)?;
        
        assert_eq!(parsed["project_name"], "test_project");
        assert_eq!(parsed["template"], "minimal");
        
        // Clean up
        temp_dir.close()?;
        
        Ok(())
    }
    
    #[test]
    fn test_config_serialization() -> Result<()> {
        // Create a config with all component types
        let mut config = get_default_config();
        config.project_name = "serialization_test".to_string();
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&config)?;
        
        // Deserialize back
        let deserialized: Config = serde_json::from_str(&json)?;
        
        // Verify the round trip
        assert_eq!(deserialized.project_name, "serialization_test");
        assert_eq!(deserialized.template, "minimal");
        
        // Verify components survived the round trip
        assert!(deserialized.components.client.is_some());
        assert!(deserialized.components.server.is_some());
        assert!(deserialized.components.database.is_some());
        assert!(deserialized.components.libs.is_some());
        assert!(deserialized.components.binaries.is_some());
        assert!(deserialized.components.ai.is_some());
        assert!(deserialized.components.edge.is_some());
        assert!(deserialized.components.embedded.is_some());
        
        Ok(())
    }
    
    #[test]
    fn test_convert_old_template() {
        let mut config = Config {
            project_name: "old_template".to_string(),
            template: "old".to_string(),
            components: Components {
                ai: Some(AI {
                    models: vec!["inference".to_string()],
                    frameworks: vec![],
                }),
                ..Default::default()
            },
        };
        
        convert_old_template(&mut config);
        
        assert!(config.components.ai.is_some(), "AI component should be present");
        assert_eq!(config.components.ai.as_ref().expect("AI component should be present").frameworks, vec!["tract".to_string()]);
    }
}
