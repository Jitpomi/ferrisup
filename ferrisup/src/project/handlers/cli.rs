// CLI project handler implementation
use anyhow::{Result, anyhow};
use std::path::Path;
use serde_json::Value;
use std::process::Command;
use crate::project::handlers::traits::ProjectHandler;

/// Handler for CLI-based project generation
///
/// This handler is responsible for managing projects that are created using external CLI tools,
/// such as Embassy, Dioxus, and Tauri. It handles checking if the tool is installed, installing
/// it if needed, and generating the project using the tool's command-line interface.
pub struct CliProjectHandler {
    /// Name of the CLI tool
    name: String,
    
    /// Description of the CLI tool
    description: String,
    
    /// Templates that this handler can handle
    templates: Vec<String>,
    
    /// Base CLI command (e.g., "cargo embassy")
    cli_command: String,
    
    /// Function to generate CLI arguments based on project variables
    cli_args_fn: fn(&str, &Path, &Value) -> Vec<String>,
    
    /// Function to generate next steps based on project variables
    next_steps_fn: fn(&str, &Value) -> Vec<String>,
    
    /// Command to install the CLI tool (if needed)
    installation_command: Option<String>,
    
    /// Command to check if the CLI tool is installed
    version_check_command: Option<String>,
}

impl CliProjectHandler {
    /// Create a new CLI project handler
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
    
    fn initialize_project(&self, project_name: &str, target_dir: &Path, variables: &Value) -> crate::core::Result<()> {
        // Install CLI tool if needed
        self.install_if_needed().map_err(|e| e.to_string())?;
        
        // Generate the CLI arguments
        let args = (self.cli_args_fn)(project_name, target_dir, variables);
        
        // Split the CLI command
        let parts: Vec<&str> = self.cli_command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Invalid CLI command".into());
        }
        
        let program = parts[0];
        let mut all_args: Vec<String> = parts[1..].iter().map(|&s| s.to_string()).collect();
        all_args.extend(args);
        
        // Run the CLI command
        println!("🔄 Creating new {} project using {}...", project_name, self.name);
        
        let status = Command::new(program)
            .args(&all_args)
            .current_dir(target_dir.parent().unwrap_or(Path::new(".")))
            .status()
            .map_err(|e| format!("Failed to execute CLI command: {}", e))?;
            
        if !status.success() {
            return Err(format!("Failed to create {} project", self.name).into());
        }
        
        println!("✅ {} project created successfully!", project_name);
        Ok(())
    }
    
    fn get_next_steps(&self, project_name: &str, variables: &Value) -> Vec<String> {
        (self.next_steps_fn)(project_name, variables)
    }
}
