// Configuration management for FerrisUp
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::error::{Error, Result};

/// Configuration for FerrisUp
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Path to templates directory
    pub templates_dir: PathBuf,
    /// User preferences
    pub preferences: Preferences,
}

/// User preferences for FerrisUp
#[derive(Debug, Serialize, Deserialize)]
pub struct Preferences {
    /// Whether to initialize git repositories for new projects
    #[serde(default = "default_git")]
    pub git: bool,
    /// Whether to build projects after creation
    #[serde(default = "default_build")]
    pub build: bool,
    /// Whether to use interactive mode
    #[serde(default = "default_interactive")]
    pub interactive: bool,
}

fn default_git() -> bool {
    true
}

fn default_build() -> bool {
    false
}

fn default_interactive() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        let mut templates_dir = dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".ferrisup")
            .join("templates");

        // If running from a development environment, use the local templates directory
        if Path::new("templates").exists() {
            templates_dir = PathBuf::from("templates");
        }

        Self {
            templates_dir,
            preferences: Preferences::default(),
        }
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            git: default_git(),
            build: default_build(),
            interactive: default_interactive(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;
        let config: Config = serde_json::from_str(&content).map_err(|e| Error::Config(format!("Invalid config format: {}", e)))?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content).map_err(|e| Error::Config(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }

    /// Get the default configuration path
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .expect("Failed to get config directory")
            .join("ferrisup")
            .join("config.json")
    }
    
    /// Get the config path to use for operations
    pub fn get_config_path() -> PathBuf {
        // First check if there's a local config file
        let local_config = Path::new("ferrisup.json");
        if local_config.exists() {
            return local_config.to_path_buf();
        }
        
        // Then check the default config path
        Self::default_path()
    }
    
    /// Get the default configuration
    pub fn get_default_config() -> Self {
        Self::default()
    }
}
