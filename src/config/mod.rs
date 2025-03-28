use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project_name: String,
    pub template: String,
    pub components: Components,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<Client>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<Database>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub libs: Option<Libs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binaries: Option<Binaries>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai: Option<AI>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge: Option<Edge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded: Option<Embedded>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub apps: Vec<String>,
    pub frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub services: Vec<String>,
    pub frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub db_type: String,
    pub orm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Libs {
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binaries {
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI {
    pub models: Vec<String>,
    pub frameworks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub apps: Vec<String>,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedded {
    pub devices: Vec<String>,
    pub platforms: Vec<String>,
}

pub fn get_config_path() -> Result<String> {
    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    Ok(format!("{}/config.json", cargo_manifest_dir))
}

pub fn read_config() -> Result<Config> {
    let config_path = get_config_path()?;
    let config_str = fs::read_to_string(&config_path)
        .context(format!("Failed to read config from {}", config_path))?;
    
    let config: Config = serde_json::from_str(&config_str)
        .context("Failed to parse config.json")?;
    
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
                db_type: "postgres".to_string(),
                orm: "sea-orm".to_string(),
            }),
            libs: Some(Libs {
                packages: vec!["core".to_string(), "models".to_string(), "utils".to_string()],
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
