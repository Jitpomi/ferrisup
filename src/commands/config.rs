use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use std::fs;
use std::path::Path;

use crate::config::{self, Config, read_config, write_config};

/// Execute the config command for managing configurations
pub fn execute(export: bool, import: Option<&str>, path: Option<&str>) -> Result<()> {
    if export {
        export_config(path)?;
    } else if let Some(import_path) = import {
        import_config(import_path, path)?;
    } else {
        // Interactive mode
        run_interactive()?;
    }
    
    Ok(())
}

/// Run interactive configuration management
fn run_interactive() -> Result<()> {
    println!("{}", "FerrisUp Configuration Manager".bold().green());
    println!("Manage and customize your FerrisUp configurations\n");
    
    let options = vec!["Export current config", "Import config", "View current config", "Cancel"];
    
    let selection = Select::new()
        .with_prompt("Select an operation")
        .items(&options)
        .default(0)
        .interact()?;
    
    match selection {
        0 => {
            let path = Input::<String>::new()
                .with_prompt("Export config to file")
                .default("ferrisup-config.json".to_string())
                .interact()?;
            
            export_config(Some(&path))?;
        },
        1 => {
            let import_path = Input::<String>::new()
                .with_prompt("Import config from file")
                .interact()?;
            
            if !Path::new(&import_path).exists() {
                println!("{} File not found", "Error:".red().bold());
                return Ok(());
            }
            
            let export_path = Input::<String>::new()
                .with_prompt("Save imported config to (leave empty to apply directly)")
                .allow_empty(true)
                .interact()?;
            
            if export_path.is_empty() {
                import_config(&import_path, None)?;
            } else {
                import_config(&import_path, Some(&export_path))?;
            }
        },
        2 => {
            view_current_config()?;
        },
        _ => {
            println!("Operation cancelled");
        }
    }
    
    Ok(())
}

/// Export the current configuration to a file
fn export_config(path: Option<&str>) -> Result<()> {
    let config_path = match path {
        Some(p) => p.to_string(),
        None => "ferrisup-config.json".to_string(),
    };
    
    // Read current config or create default
    let config = match read_config() {
        Ok(cfg) => cfg,
        Err(_) => {
            println!("{} No existing config found, using default", "Warning:".yellow().bold());
            config::get_default_config()
        }
    };
    
    // Write to the specified path
    let path = Path::new(&config_path);
    write_config(&config, path)?;
    
    println!("{} {}", "Configuration exported to:".green(), config_path);
    
    Ok(())
}

/// Import a configuration from a file
fn import_config(import_path: &str, export_path: Option<&str>) -> Result<()> {
    // Read the config to import
    let content = fs::read_to_string(import_path)
        .context(format!("Failed to read config from {}", import_path))?;
    
    let config: Config = serde_json::from_str(&content)
        .context("Failed to parse config as JSON")?;
    
    if let Some(path) = export_path {
        // Write to the specified path
        write_config(&config, Path::new(path))?;
        println!("{} {}", "Configuration saved to:".green(), path);
    } else {
        // Apply as current config
        write_config(&config, Path::new("config.json"))?;
        println!("{}", "Configuration applied as current".green());
    }
    
    Ok(())
}

/// View the current configuration
fn view_current_config() -> Result<()> {
    // Read current config or use default
    let config = match read_config() {
        Ok(cfg) => cfg,
        Err(_) => {
            println!("{} No existing config found, showing default", "Warning:".yellow().bold());
            config::get_default_config()
        }
    };
    
    // Convert to pretty JSON for display
    let json = serde_json::to_string_pretty(&config)
        .context("Failed to serialize config to JSON")?;
    
    println!("\n{}", "Current Configuration:".bold());
    println!("{}", json);
    
    // Ask if user wants to export this config
    if Confirm::new()
        .with_prompt("Export this configuration?")
        .default(false)
        .interact()?
    {
        let path = Input::<String>::new()
            .with_prompt("Export config to file")
            .default("ferrisup-config.json".to_string())
            .interact()?;
        
        write_config(&config, Path::new(&path))?;
        println!("{} {}", "Configuration exported to:".green(), path);
    }
    
    Ok(())
}
