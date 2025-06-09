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
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Check if the output contains information about pruning features
        if stderr.contains("Prune") {
            println!("{}", "Found unused features:".yellow());
            
            // Parse and display the unused features from the log output
            let log_lines: Vec<&str> = stderr.lines().collect();
            let mut current_dependency = "";
            let mut unused_features = Vec::new();
            
            for line in log_lines {
                if line.contains("==== Dependency") {
                    // Extract dependency name
                    if let Some(dep_name) = line.split("'")
                        .nth(1) {
                        current_dependency = dep_name;
                        println!("{}", format!("\nDependency: {}", current_dependency).yellow().bold());
                    }
                } else if line.contains("Prune") && line.contains("feature flag from") {
                    // Extract feature name
                    if let Some(feature_name) = line.split("'")
                        .nth(1) {
                        println!("  - {}", feature_name);
                        unused_features.push((current_dependency, feature_name));
                    }
                }
            }
            
            println!("\n{}", "Recommendations:".green());
            println!("- Review the unused features and consider removing them from your Cargo.toml");
            println!("- For each dependency with unused features, update it like this:");
            
            // Group by dependency
            let mut deps = std::collections::HashMap::new();
            for (dep, feature) in unused_features {
                deps.entry(dep).or_insert_with(Vec::new).push(feature);
            }
            
            // Print example for each dependency
            for (dep, features) in deps {
                println!("  {} = {{ version = \"x.y\", features = [] }} # Removed: {}", 
                         dep.italic(), 
                         features.join(", "));
            }
            
            println!("- Run 'ferrisup unused-features' again after making changes to verify");
            
            // Clean up any report file if it exists
            let report_path = target_path.join("report.json");
            if report_path.exists() {
                let _ = std::fs::remove_file(&report_path);
            }
        } else {
            println!("{}", "✅ No unused features found!".green());
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
