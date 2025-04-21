use anyhow::{Result, anyhow};
use std::path::Path;
use serde_json::Value;
use std::process::Command;
use crate::project_handlers::traits::ProjectHandler;

/// Handler for CLI-based project generation
pub struct CliProjectHandler {
    name: String,
    description: String,
    templates: Vec<String>,
    cli_command: String,
    cli_args_fn: fn(&str, &Path, &Value) -> Vec<String>,
    next_steps_fn: fn(&str, &Value) -> Vec<String>,
    installation_command: Option<String>,
    version_check_command: Option<String>,
}

impl CliProjectHandler {
    pub fn new(
        name: &str, 
        description: &str,
        templates: Vec<String>,
        cli_command: &str,
        cli_args_fn: fn(&str, &Path, &Value) -> Vec<String>,
        next_steps_fn: fn(&str, &Value) -> Vec<String>,
        installation_command: Option<String>,
        version_check_command: Option<String>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            templates,
            cli_command: cli_command.to_string(),
            cli_args_fn,
            next_steps_fn,
            installation_command,
            version_check_command,
        }
    }
    
    /// Check if the CLI tool is installed
    fn is_installed(&self) -> Result<bool> {
        if let Some(check_cmd) = &self.version_check_command {
            let parts: Vec<&str> = check_cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Ok(false);
            }
            
            let program = parts[0];
            let args = &parts[1..];
            
            let output = Command::new(program)
                .args(args)
                .output();
                
            match output {
                Ok(output) => Ok(output.status.success()),
                Err(_) => Ok(false)
            }
        } else {
            // No check command, assume it's installed
            Ok(true)
        }
    }
    
    /// Install the CLI tool if needed
    fn install_if_needed(&self) -> Result<bool> {
        if !self.is_installed()? {
            println!("⚠️ {} not found. Installing...", self.name);
            
            if let Some(install_cmd) = &self.installation_command {
                let parts: Vec<&str> = install_cmd.split_whitespace().collect();
                if parts.is_empty() {
                    return Err(anyhow!("Invalid installation command"));
                }
                
                let program = parts[0];
                let args = &parts[1..];
                
                let status = Command::new(program)
                    .args(args)
                    .status()?;
                    
                if !status.success() {
                    return Err(anyhow!("Failed to install {}", self.name));
                }
                
                println!("✅ {} installed successfully", self.name);
                Ok(true)
            } else {
                Err(anyhow!("No installation command provided for {}", self.name))
            }
        } else {
            println!("✅ {} is already installed", self.name);
            Ok(false)
        }
    }
}

impl ProjectHandler for CliProjectHandler {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn can_handle(&self, template_name: &str, _variables: &Value) -> bool {
        self.templates.contains(&template_name.to_string())
    }
    
    fn initialize_project(&self, project_name: &str, target_dir: &Path, variables: &Value) -> Result<()> {
        // Install CLI tool if needed
        self.install_if_needed()?;
        
        // Generate the CLI arguments
        let args = (self.cli_args_fn)(project_name, target_dir, variables);
        
        // Split the CLI command
        let parts: Vec<&str> = self.cli_command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Invalid CLI command"));
        }
        
        let program = parts[0];
        let mut all_args: Vec<String> = parts[1..].iter().map(|&s| s.to_string()).collect();
        all_args.extend(args);
        
        // Run the CLI command
        println!("🔄 Creating new {} project using {}...", project_name, self.name);
        
        // Special handling for cargo subcommands
        let command_result = if program == "cargo" && !all_args.is_empty() {
            // For cargo subcommands like "cargo embassy", we need to find the actual binary
            // The binary is likely installed as cargo-embassy, not as embassy
            if all_args[0] == "embassy" {
                let cargo_binary = format!("cargo-{}", all_args[0]);
                // Remove the first argument (the subcommand name)
                let embassy_args = all_args[1..].to_vec();
                
                // Try to run the cargo-embassy binary directly
                Command::new(&cargo_binary)
                    .args(&embassy_args)
                    .current_dir(target_dir.parent().unwrap_or(Path::new(".")))
                    .status()
            } else {
                // Regular cargo subcommand
                Command::new(program)
                    .args(&all_args)
                    .current_dir(target_dir.parent().unwrap_or(Path::new(".")))
                    .status()
            }
        } else {
            // Standard command (not a cargo subcommand)
            Command::new(program)
                .args(&all_args)
                .current_dir(target_dir.parent().unwrap_or(Path::new(".")))
                .status()
        };
        
        match command_result {
            Ok(status) => {
                if !status.success() {
                    return Err(anyhow!("Failed to create {} project, command exited with non-zero status", self.name));
                }
            },
            Err(e) => {
                return Err(anyhow!("Failed to execute CLI command: {}", e));
            }
        }
        
        println!("✅ {} project created successfully!", project_name);
        Ok(())
    }
    
    fn get_next_steps(&self, project_name: &str, variables: &Value) -> Vec<String> {
        (self.next_steps_fn)(project_name, variables)
    }
}
