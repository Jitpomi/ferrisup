use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use colored::Colorize;

/// Execute the unused-features command to find unused features in Cargo dependencies
pub fn execute(path: Option<&str>) -> Result<()> {
    // Determine the target path
    let target_path = match path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir()?,
    };

    // Verify that the path exists and is a directory
    if !target_path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", target_path.display()));
    }
    
    if !target_path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", target_path.display()));
    }

    // Check if Cargo.toml exists in the target directory
    let cargo_toml_path = target_path.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!("No Cargo.toml found in {}", target_path.display()));
    }

    // Check if cargo-unused-features is installed
    if !is_cargo_unused_features_installed() {
        println!("{}", "cargo-unused-features is not installed.".yellow());
        println!("{}", "Installing cargo-unused-features...".yellow());
        
        // Install cargo-unused-features
        let install_result = Command::new("cargo")
            .args(["install", "cargo-unused-features"])
            .status();
            
        match install_result {
            Ok(status) if status.success() => {
                println!("{}", "Successfully installed cargo-unused-features.".green());
            },
            Ok(_) => {
                return Err(anyhow::anyhow!("Failed to install cargo-unused-features. Please install it manually with 'cargo install cargo-unused-features'"));
            },
            Err(e) => {
                return Err(anyhow::anyhow!("Error installing cargo-unused-features: {}", e));
            }
        }
    }

    println!("{}", "Analyzing unused features in your project...".blue());
    
    // Run unused-features binary with the analyze subcommand
    let output = Command::new("unused-features")
        .arg("analyze")
        .current_dir(&target_path)
        .output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        if stdout.trim().is_empty() {
            println!("{}", "✅ No unused features found!".green());
        } else {
            println!("{}", "Found unused features:".yellow());
            println!("{}", stdout);
            
            println!("\n{}", "Recommendations:".green());
            println!("- Review the unused features and consider removing them from your Cargo.toml");
            println!("- For each dependency with unused features, update it like this:");
            println!("  {} = {{ version = \"0.1\", features = [\"needed-feature\"] }} # Remove unused features", "dependency".italic());
            println!("- Run 'cargo unused-features' again after making changes to verify");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Error running cargo-unused-features: {}", stderr));
    }

    Ok(())
}

/// Check if unused-features binary is installed
fn is_cargo_unused_features_installed() -> bool {
    let output = Command::new("unused-features")
        .arg("--help")
        .output();
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
