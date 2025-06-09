use anyhow::{Result, anyhow};
use std::process::Command;
use serde_json::Value;
use std::path::Path;

/// Trait defining the interface for CLI tool handlers
pub trait CliHandler {
    /// Get the name of the CLI tool
    fn get_name(&self) -> &str;
    
    /// Check if the CLI tool is installed
    fn is_installed(&self) -> Result<bool>;
    
    /// Install the CLI tool if needed
    fn install_if_needed(&self) -> Result<bool>;
    
    /// Generate a project using the CLI tool
    fn generate_project(&self, project_name: &str, target_dir: &Path, options: &Value) -> Result<()>;
    
    /// Get next steps for this CLI-generated project
    fn get_next_steps(&self, project_name: &str, options: &Value) -> Vec<String>;
}

/// Embassy CLI handler implementation
pub struct EmbassyCliHandler;

impl EmbassyCliHandler {
    pub fn new() -> Self {
        Self {}
    }
    
    // Map MCU target to chip name
    fn map_target_to_chip<'a>(&self, target: &'a str) -> &'a str {
        match target {
            "esp32" => "esp32c3",
            chip => chip // Use as-is for other chips
        }
    }
}

impl CliHandler for EmbassyCliHandler {
    fn get_name(&self) -> &str {
        "cargo-embassy"
    }
    
    fn is_installed(&self) -> Result<bool> {
        let output = Command::new("cargo")
            .args(["embassy", "--version"])
            .output();
            
        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false)
        }
    }
    
    fn install_if_needed(&self) -> Result<bool> {
        println!("🔍 Checking for cargo-embassy...");
        
        if !self.is_installed()? {
            println!("⚠️ cargo-embassy not found. Installing...");
            
            let status = Command::new("cargo")
                .args(["install", "cargo-embassy"])
                .status()?;
                
            if !status.success() {
                println!("Please install it manually with: cargo install cargo-embassy");
                return Err(anyhow!("Failed to install cargo-embassy"));
            }
            
            println!("✅ cargo-embassy installed successfully");
            Ok(true)
        } else {
            println!("✅ cargo-embassy is already installed");
            Ok(false)
        }
    }
    
    fn generate_project(&self, project_name: &str, target_dir: &Path, options: &Value) -> Result<()> {
        // Get MCU target from options
        let mcu_target = if let Some(target) = options.get("mcu_target").and_then(|t| t.as_str()) {
            self.map_target_to_chip(target)
        } else {
            return Err(anyhow!("No microcontroller target specified for Embassy project"));
        };
        
        println!("Using {} as the Embassy chip", mcu_target);
        println!("🔄 Creating new Embassy project...");
        
        // Create a temporary directory for the Embassy project
        let temp_dir = target_dir.join("embassy_temp");
        std::fs::create_dir_all(&temp_dir)?;
        
        // Run cargo-embassy init
        let status = Command::new("cargo")
            .args(["embassy", "init", "--chip", mcu_target, project_name])
            .current_dir(&temp_dir)
            .status()?;
            
        if !status.success() {
            return Err(anyhow!("Failed to create Embassy project"));
        }
        
        // Move the generated project to the target directory
        let project_dir = temp_dir.join(project_name);
        if project_dir.exists() {
            // Copy all files from the generated project to the target directory
            let target_dir = Path::new(project_name);
            if !target_dir.exists() {
                std::fs::create_dir_all(target_dir)?;
            }
            
            // Copy the contents
            copy_dir_all(&project_dir, target_dir)?;
            
            // Clean up the temp directory
            std::fs::remove_dir_all(temp_dir)?;
            
            Ok(())
        } else {
            Err(anyhow!("Embassy project directory not found"))
        }
    }
    
    fn get_next_steps(&self, project_name: &str, _options: &Value) -> Vec<String> {
        vec![
            format!("🚀 Navigate to your project: cd {}", project_name),
            "📝 Review the generated code".to_string(),
            "🔧 Build the project: cargo build --release".to_string(),
            "▶️ Flash the project: cargo run --release".to_string(),
            "📚 Read the Embassy documentation: https://embassy.dev".to_string()
        ]
    }
}

/// Helper function to recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_dir_all(&path, &target)?;
        } else {
            std::fs::copy(&path, &target)?;
        }
    }
    
    Ok(())
}

/// Function to get a CLI handler by name
pub fn get_handler(name: &str) -> Option<Box<dyn CliHandler>> {
    match name {
        "embassy" => Some(Box::new(EmbassyCliHandler::new())),
        _ => None
    }
}
