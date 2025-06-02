use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect};
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use crate::utils::update_cargo_with_dependencies;

#[derive(Debug, Args)]
pub struct DependencyArgs {
    #[command(subcommand)]
    command: DependencyCommands,
}

#[derive(Debug, Subcommand)]
pub enum DependencyCommands {
    /// Add dependencies to your project
    Add(AddArgs),
    
    /// Remove dependencies from your project
    Remove(RemoveArgs),
    
    /// Update dependencies in your project
    Update(UpdateArgs),
    
    /// Analyze dependencies in your project
    Analyze(AnalyzeArgs),
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Dependencies to add (e.g., serde, tokio, etc.)
    #[arg(required = false)]
    pub dependencies: Vec<String>,
    
    /// Add as development dependency
    #[arg(short, long)]
    pub dev: bool,
    
    /// Add with specific features (comma separated)
    #[arg(short, long)]
    pub features: Option<String>,
    
    /// Add with specific version
    #[arg(short, long)]
    pub version: Option<String>,
    
    /// Path to the project (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
    
    /// Disable interactive prompts
    #[arg(long)]
    pub no_interactive: bool,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Dependencies to remove
    #[arg(required = false)]
    pub dependencies: Vec<String>,
    
    /// Path to the project (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Dependencies to update (if empty, updates all)
    #[arg(required = false)]
    pub dependencies: Vec<String>,
    
    /// Path to the project (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct AnalyzeArgs {
    /// Path to the project (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

/// Execute the dependency command
pub fn execute(args: DependencyArgs) -> Result<()> {
    match args.command {
        DependencyCommands::Add(args) => add_dependencies(args),
        DependencyCommands::Remove(args) => remove_dependencies(args),
        DependencyCommands::Update(args) => update_dependencies(args),
        DependencyCommands::Analyze(args) => analyze_dependencies(args),
    }
}

/// Add dependencies to a project
pub fn add_dependencies(args: AddArgs) -> Result<()> {
    // Get project directory
    let project_dir = match &args.path {
        Some(path) => path.clone(),
        None => PathBuf::from(".")
    };
    
    // Verify this is a Rust project
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!("No Cargo.toml found in the specified directory. Are you sure this is a Rust project?"));
    }
    
    // If no dependencies were provided, prompt for them
    let dependencies = if args.dependencies.is_empty() {
        let input: String = Input::new()
            .with_prompt("Enter dependencies to add (comma separated)")
            .interact_text()?;
        
        input.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        args.dependencies.clone()
    };
    
    if dependencies.is_empty() {
        return Err(anyhow::anyhow!("No dependencies specified"));
    }

    // Check if we need to handle dependencies that exist in the wrong section
    let cargo_content = fs::read_to_string(&cargo_toml_path)
        .context("Failed to read Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_content)
        .context("Failed to parse Cargo.toml as valid TOML")?;
    
    // Check for dependencies in the wrong section
    let section_to_check = if args.dev { "dependencies" } else { "dev-dependencies" };
    let target_section = if args.dev { "dev-dependencies" } else { "dependencies" };
    let section_flag = if section_to_check == "dev-dependencies" { "--dev" } else { "" };
    
    if let Some(deps_table) = cargo_toml.get(section_to_check) {
        if let Some(deps_table) = deps_table.as_table() {
            for dependency in &dependencies {
                if deps_table.contains_key(dependency) {
                    println!("{} {} {} {}", 
                        "Moving".yellow(), 
                        dependency.bold(), 
                        format!("from {} to", section_to_check).yellow(),
                        target_section.yellow());
                    
                    // First remove the dependency from the wrong section
                    let mut remove_cmd = std::process::Command::new("cargo");
                    remove_cmd.current_dir(&project_dir);
                    
                    // Add the appropriate section flag if needed
                    if !section_flag.is_empty() {
                        remove_cmd.args(["remove", dependency, section_flag]);
                    } else {
                        remove_cmd.args(["remove", dependency]);
                    }
                    
                    let output = remove_cmd.output()
                        .context(format!("Failed to remove dependency {} from {}", dependency, section_to_check))?;
                    
                    if !output.status.success() {
                        let error = String::from_utf8_lossy(&output.stderr);
                        println!("{} {}", 
                            "Warning:".yellow().bold(), 
                            format!("Failed to remove dependency {} from {}: {}", dependency, section_to_check, error).yellow());
                    }
                }
            }
        }
    }
    
    // Process each dependency
    let mut dependencies_to_add = Vec::new();
    
    for dependency in dependencies {
        let version = args.version.clone().unwrap_or_else(|| "*".to_string());
        let mut features_option: Option<Vec<String>> = None;
        
        // Handle features if provided via command line
        if let Some(features_str) = &args.features {
            let features: Vec<String> = features_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
                
            if !features.is_empty() {
                features_option = Some(features);
            }
        } 
        // If no features provided via command line, suggest popular features
        else if !args.no_interactive {
            // Get suggested features for this dependency
            let features = suggest_features_interactive(&dependency)?;
            if !features.is_empty() {
                features_option = Some(features);
            }
        }
        
        dependencies_to_add.push((dependency, version, features_option));
    }
    
    // Use our enhanced utility function
    update_cargo_with_dependencies(&cargo_toml_path, dependencies_to_add, args.dev)?;
    
    Ok(())
}

/// Get a map of popular features for common crates
fn get_popular_features(dependency: &str) -> Option<Vec<&'static str>> {
    match dependency {
        "tokio" => Some(vec!["full", "rt", "rt-multi-thread", "macros", "io-util", "time"]),
        "serde" => Some(vec!["derive"]),
        "reqwest" => Some(vec!["json", "blocking", "rustls-tls", "cookies", "gzip"]),
        "axum" => Some(vec!["headers", "http2", "macros", "multipart", "ws"]),
        "diesel" => Some(vec!["postgres", "mysql", "sqlite", "r2d2", "chrono"]),
        "sqlx" => Some(vec!["runtime-tokio-rustls", "postgres", "mysql", "sqlite", "macros"]),
        "clap" => Some(vec!["derive", "cargo", "env", "wrap_help"]),
        _ => None,
    }
}

/// Interactive feature selection for a dependency
fn suggest_features_interactive(dependency: &str) -> Result<Vec<String>> {
    let mut selected_features = Vec::new();
    
    if let Some(suggested_features) = get_popular_features(dependency) {
        println!("Suggested features for {}:", dependency.green());
        
        let selections = MultiSelect::new()
            .items(&suggested_features)
            .interact()?;
        
        if !selections.is_empty() {
            selected_features = selections.iter()
                .map(|&i| suggested_features[i].to_string())
                .collect();
        }
    }
    
    Ok(selected_features)
}

/// Remove dependencies from a project
pub fn remove_dependencies(args: RemoveArgs) -> Result<()> {
    let project_dir = args.path.unwrap_or_else(|| PathBuf::from("."));
    
    // Verify this is a Rust project
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!("No Cargo.toml found in the specified directory. Are you sure this is a Rust project?"));
    }
    
    // If no dependencies were provided, list current dependencies and prompt
    let dependencies = if args.dependencies.is_empty() {
        // Parse Cargo.toml to get current dependencies
        let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
            .context("Failed to read Cargo.toml")?;
        
        let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)
            .context("Failed to parse Cargo.toml")?;
        
        let mut all_deps = Vec::new();
        
        // Get regular dependencies
        if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
            all_deps.extend(deps.keys().map(|k| k.to_string()));
        }
        
        // Get dev-dependencies
        if let Some(deps) = cargo_toml.get("dev-dependencies").and_then(|d| d.as_table()) {
            all_deps.extend(deps.keys().map(|k| k.to_string()));
        }
        
        // Get build-dependencies
        if let Some(deps) = cargo_toml.get("build-dependencies").and_then(|d| d.as_table()) {
            all_deps.extend(deps.keys().map(|k| k.to_string()));
        }
        
        if all_deps.is_empty() {
            return Err(anyhow::anyhow!("No dependencies found in the project"));
        }
        
        // Prompt user to select dependencies to remove
        let selections = MultiSelect::new()
            .with_prompt("Select dependencies to remove")
            .items(&all_deps)
            .interact()?;
        
        if selections.is_empty() {
            return Ok(());
        }
        
        selections.into_iter()
            .map(|i| all_deps[i].clone())
            .collect()
    } else {
        args.dependencies
    };
    
    // Remove each dependency
    for dependency in &dependencies {
        println!("{} {}", "Removing dependency:".yellow(), dependency);
        
        let output = Command::new("cargo")
            .args(["remove", dependency])
            .current_dir(&project_dir)
            .output()
            .context(format!("Failed to remove dependency {}", dependency))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to remove dependency {}: {}", dependency, error));
        }
        
        println!("{} {}", "Successfully removed:".green(), dependency);
    }
    
    Ok(())
}

/// Update dependencies in a project
pub fn update_dependencies(args: UpdateArgs) -> Result<()> {
    let project_dir = args.path.unwrap_or_else(|| PathBuf::from("."));
    
    // Verify this is a Rust project
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!("No Cargo.toml found in the specified directory. Are you sure this is a Rust project?"));
    }
    
    // If specific dependencies were provided, update only those
    if !args.dependencies.is_empty() {
        for dependency in &args.dependencies {
            println!("{} {}", "Updating dependency:".blue(), dependency);
            
            let output = Command::new("cargo")
                .args(["update", dependency])
                .current_dir(&project_dir)
                .output()
                .context(format!("Failed to update dependency {}", dependency))?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to update dependency {}: {}", dependency, error));
            }
            
            println!("{} {}", "Successfully updated:".green(), dependency);
        }
    } else {
        // Update all dependencies
        println!("{}", "Updating all dependencies...".blue());
        
        let output = Command::new("cargo")
            .args(["update"])
            .current_dir(&project_dir)
            .output()
            .context("Failed to update dependencies")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to update dependencies: {}", error));
        }
        
        println!("{}", "Successfully updated all dependencies".green());
    }
    
    Ok(())
}

/// Analyze dependencies in a project
pub fn analyze_dependencies(args: AnalyzeArgs) -> Result<()> {
    let project_dir = args.path.unwrap_or_else(|| PathBuf::from("."));
    
    // Verify this is a Rust project
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(anyhow::anyhow!("No Cargo.toml found in the specified directory. Are you sure this is a Rust project?"));
    }
    
    println!("{}", "Analyzing dependencies...".blue());
    
    // Check if cargo-audit is installed
    let audit_installed = Command::new("cargo")
        .args(["audit", "--version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    
    if !audit_installed {
        println!("{}", "cargo-audit is not installed. It's recommended for security analysis.".yellow());
        if Confirm::new()
            .with_prompt("Would you like to install cargo-audit?")
            .interact()?
        {
            println!("{}", "Installing cargo-audit...".blue());
            
            let output = Command::new("cargo")
                .args(["install", "cargo-audit"])
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .context("Failed to install cargo-audit")?;
            
            if !output.success() {
                println!("{} Check your internet connection and try again", "Failed to install cargo-audit:".red());
            } else {
                println!("{}", "Successfully installed cargo-audit".green());
            }
        }
    }
    
    // Run cargo tree
    println!("\n{}", "Dependency tree:".blue());
    let tree_output = Command::new("cargo")
        .args(["tree"])
        .current_dir(&project_dir)
        .output()
        .context("Failed to run cargo tree")?;
    
    println!("{}", String::from_utf8_lossy(&tree_output.stdout));
    
    // Run cargo audit if available
    if audit_installed || Command::new("cargo")
        .args(["audit", "--version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        println!("\n{}", "Security audit:".blue());
        let audit_output = Command::new("cargo")
            .args(["audit"])
            .current_dir(&project_dir)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .context("Failed to run cargo audit")?;
        
        if !audit_output.success() {
            println!("{}", "Security vulnerabilities found in your dependencies. Please review and update.".yellow());
        }
    }
    
    Ok(())
}
