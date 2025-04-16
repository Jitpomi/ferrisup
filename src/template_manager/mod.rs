use anyhow::{Result, anyhow};
use std::fs;
use std::fs::File;
use std::io::{self, Write, BufRead};
use std::path::{Path, PathBuf};
use serde_json::{Value, json, Map};
use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;
use handlebars::Handlebars;
use colored::Colorize;
use dialoguer::Select;
use std::process::Command;
use walkdir::WalkDir;
use regex::Regex;
use std::os::unix::fs::PermissionsExt;

lazy_static! {
    static ref CURRENT_VARIABLES: Arc<RwLock<Map<String, Value>>> = Arc::new(RwLock::new(Map::new()));
}

pub fn get_template(name: &str) -> Result<String> {
    let templates = get_all_templates()?;
    
    if templates.contains(&name.to_string()) {
        // Check if the template has a valid template.json file
        let template_dir = format!("{}/templates/{}", env!("CARGO_MANIFEST_DIR"), name);
        let template_json = Path::new(&template_dir).join("template.json");
        
        if template_json.exists() {
            Ok(name.to_string())
        } else {
            println!("⚠️ Template '{}' does not have a valid configuration. Using minimal template instead.", name);
            Ok("minimal".to_string())
        }
    } else {
        // Fall back to minimal if template not found
        println!("⚠️ Template '{}' not found. Using minimal template instead.", name);
        Ok("minimal".to_string())
    }
}

pub fn get_all_templates() -> Result<Vec<String>> {
    // List all built-in templates
    let templates = vec![
        "minimal".to_string(),
        "library".to_string(),
        "embedded".to_string(),
        "server".to_string(),
        "serverless".to_string(),
        "client".to_string(),
        "data-science".to_string(),
    ];
    
    // Check for custom templates in the templates directory
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
    if let Ok(entries) = fs::read_dir(&templates_dir) {
        let mut all_templates = templates;
        
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    // Skip templates that are known to be incomplete
                    if dir_name == "web" || !entry.path().join("template.json").exists() {
                        continue;
                    }
                    
                    // Only add if not already in the list
                    if !all_templates.contains(&dir_name.to_string()) {
                        all_templates.push(dir_name.to_string());
                    }
                }
            }
        }
        
        return Ok(all_templates);
    }
    
    Ok(templates)
}

/// Returns a list of templates with their descriptions
/// Format: Vec<(name, description)>
pub fn list_templates() -> Result<Vec<(String, String)>> {
    // Define core templates with descriptions
    let mut templates = vec![
        ("minimal".to_string(), "Simple binary with a single main.rs file".to_string()),
        ("library".to_string(), "Rust library crate with a lib.rs file".to_string()),
        ("embedded".to_string(), "Embedded systems firmware for microcontrollers".to_string()),
        ("server".to_string(), "Web server with API endpoints (Axum, Actix, or Poem)".to_string()),
        ("serverless".to_string(), "Serverless functions for cloud deployment".to_string()),
        ("client".to_string(), "Frontend client application".to_string()),
        ("data-science".to_string(), "Data science and machine learning projects".to_string()),
    ];
    
    // Track template names we've already added to avoid duplicates
    let template_names: Vec<String> = templates.iter().map(|(name, _)| name.clone()).collect();
    
    // Check for custom templates in the templates directory
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
    if let Ok(entries) = fs::read_dir(&templates_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    // Skip templates we've already added or those that are known to be incomplete
                    if template_names.contains(&dir_name.to_string()) || 
                       dir_name == "web" || 
                       !entry.path().join("template.json").exists() {
                        continue;
                    }
                    
                    // Skip data-science subdirectories in the main list
                    if dir_name.starts_with("data-science/") {
                        continue;
                    }
                    
                    // Try to read description from template.json if it exists
                    let template_json = entry.path().join("template.json");
                    let description = if template_json.exists() {
                        if let Ok(content) = fs::read_to_string(&template_json) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(desc) = json.get("description").and_then(|d| d.as_str()) {
                                    desc.to_string()
                                } else {
                                    format!("Custom template: {}", dir_name)
                                }
                            } else {
                                format!("Custom template: {}", dir_name)
                            }
                        } else {
                            format!("Custom template: {}", dir_name)
                        }
                    } else {
                        format!("Custom template: {}", dir_name)
                    };
                    
                    templates.push((dir_name.to_string(), description));
                }
            }
        }
    }
    
    Ok(templates)
}

/// Get data science templates with descriptions
pub fn list_data_science_templates() -> Result<Vec<(String, String)>> {
    Ok(vec![
        // Data Analysis
        ("data-science/polars-cli".to_string(), "Data Analysis: Process and analyze data with Polars (similar to pandas)".to_string()),
        
        // Machine Learning
        ("data-science/linfa-examples".to_string(), "Machine Learning: Working examples with Linfa 0.7.1 (classification, regression, clustering)".to_string()),
        
        // Deep Learning with Burn Framework - Image Processing
        ("data-science/burn-image-recognition".to_string(), "Burn - Image Recognition: Identify handwritten numbers with MNIST dataset".to_string()),
        ("data-science/burn-custom-image".to_string(), "Burn - Custom Image Classifier: Train a model on your own image dataset".to_string()),
        ("data-science/burn-image-classifier".to_string(), "Burn - Image Classifier: Customizable CNN for multi-class image classification".to_string()),
        
        // Deep Learning with Burn Framework - Text Processing
        ("data-science/burn-text-classifier".to_string(), "Burn - Text Classifier: Categorize text into predefined classes".to_string()),
        ("data-science/burn-text-analyzer".to_string(), "Burn - Text Analyzer: Analyze text sentiment with customizable LSTM model".to_string()),
        
        // Deep Learning with Burn Framework - Numerical Data
        ("data-science/burn-value-prediction".to_string(), "Burn - Value Prediction: Forecast numerical values with regression models".to_string()),
        ("data-science/burn-data-predictor".to_string(), "Burn - Data Predictor: Advanced regression with customizable architecture".to_string()),
        
        // Deep Learning with Burn Framework - Advanced & Experimental
        ("data-science/burn-net".to_string(), "Burn - Neural Network Playground: Experiment with custom network architectures".to_string()),
    ])
}

/// Apply a template to a target directory
/// 
/// # Arguments
/// 
/// * `template_name` - The name of the template to apply
/// * `target_dir` - The directory to apply the template to
/// * `project_name` - The name of the project
/// * `variables` - Additional variables to use in template substitution
/// 
/// # Returns
/// 
/// * `Result<()>` - Ok if successful, Err otherwise
pub fn apply_template(
    template_name: &str,
    target_dir: &Path,
    project_name: &str,
    variables: Option<Value>,
) -> Result<()> {
    let template_dir = get_template_dir(template_name)?;
    
    // Create target directory if it doesn't exist
    fs::create_dir_all(target_dir)?;
    
    // Get template configuration
    let template_config = get_template_config(template_name)?;
    
    // Prepare template variables
    let mut template_vars = json!({
        "project_name": project_name,
        "project_name_pascal_case": to_pascal_case(project_name),
    });
    
    // Add user-provided variables
    if let Some(vars) = variables {
        if let Some(obj) = vars.as_object() {
            if let Some(obj_mut) = template_vars.as_object_mut() {
                for (_key, value) in obj {
                    obj_mut.insert(_key.clone(), value.clone());
                }
            }
        }
    }
    
    // Handle data science template-specific prompts
    if template_name.starts_with("data-science/") {
        let mut additional_vars = Map::new();
        
        // Common data science questions
        println!("\n{}", "📊 Data Science Project Configuration".bold().green());
        
        if template_name == "data-science/polars-cli" {
            println!("\n{}", "Polars DataFrame Analysis Configuration:".bold());
            
            let data_source = prompt_with_options(
                "What type of data will you be working with?",
                &["CSV files", "JSON data"]
            )?;
            additional_vars.insert("data_source".to_string(), json!(data_source));
            
            let analysis_type = prompt_with_options(
                "What type of analysis do you plan to perform?",
                &["Exploratory data analysis", "Data cleaning & transformation", "Statistical analysis", "Time series analysis", "Custom analysis"]
            )?;
            additional_vars.insert("analysis_type".to_string(), json!(analysis_type));
            
            let visualization = prompt_with_default(
                "Do you need data visualization capabilities?",
                "yes"
            )?;
            additional_vars.insert("visualization".to_string(), json!(visualization.to_lowercase()));
            
            println!("\n✅ Polars DataFrame project configured successfully!");
        } else if template_name == "data-science/linfa-examples" {
            println!("\n{}", "Linfa Machine Learning Examples Configuration:".bold());
            
            // Focus only on data source, which is the meaningful distinction
            let data_source = prompt_with_options(
                "What type of data source would you like to use for the examples?",
                &["Synthetic data (generated)", "Custom data files", "Both (examples will show both options)"]
            )?;
            additional_vars.insert("data_source".to_string(), json!(data_source));
            
            // If user wants to use custom data files, ask for the format
            if data_source == "Custom data files" || data_source == "Both (examples will show both options)" {
                let data_format = prompt_with_options(
                    "What data format would you like to use?",
                    &["CSV files", "JSON files", "All formats (CSV, JSON)"]
                )?;
                additional_vars.insert("data_format".to_string(), json!(data_format));
            }
            
            println!("\n✅ Linfa machine learning examples configured successfully!");
        } else if template_name == "data-science/burn-value-prediction" || 
                  template_name == "data-science/burn-text-classifier" || 
                  template_name == "data-science/burn-custom-image" || 
                  template_name == "data-science/burn-data-predictor" ||
                  template_name == "data-science/burn-net" {
            
            // Map our template names to the corresponding Burn examples
            let burn_example = match template_name {
                "data-science/burn-value-prediction" => "simple-regression",
                "data-science/burn-text-classifier" => "text-classification",
                "data-science/burn-custom-image" => "custom-image-dataset",
                "data-science/burn-data-predictor" => "simple-regression",
                "data-science/burn-net" => "custom-training-loop", // Changed from "net" to "custom-training-loop"
                _ => "mnist", // Default fallback
            };
            
            println!("\n{}", format!("Setting up {} project...", template_name.replace("data-science/burn-", "")).bold());
            
            // Use direct clone and copy approach which is more reliable
            println!("Generating project from Burn example: {}", burn_example);
            
            // Create a temporary directory for the Burn repository
            let burn_repo_dir = std::env::temp_dir().join("burn-repo");
            if !burn_repo_dir.exists() {
                // Clone the Burn repository if it doesn't exist
                println!("Cloning Burn repository (this may take a moment)...");
                let clone_result = std::process::Command::new("git")
                    .args([
                        "clone",
                        "--depth=1",
                        "https://github.com/tracel-ai/burn.git",
                        burn_repo_dir.to_str().unwrap()
                    ])
                    .status()?;
                    
                if !clone_result.success() {
                    return Err(anyhow!("Failed to clone the Burn repository"));
                }
            } else {
                // Pull the latest changes if the repo already exists
                println!("Updating Burn repository...");
                let pull_result = std::process::Command::new("git")
                    .args([
                        "pull"
                    ])
                    .current_dir(&burn_repo_dir)
                    .status()?;
                
                if !pull_result.success() {
                    println!("Warning: Failed to update the Burn repository, using existing version");
                }
            }
            
            // Check if the example exists
            let example_dir = burn_repo_dir.join("examples").join(burn_example);
            if !example_dir.exists() {
                // Provide a more helpful error message with alternatives
                let available_examples = get_available_burn_examples(&burn_repo_dir)?;
                let available_examples_str = available_examples.join(", ");
                
                return Err(anyhow!(
                    "Burn example '{}' not found in the repository. Available examples are: {}",
                    burn_example,
                    available_examples_str
                ));
            }
            
            // Create the target directory if it doesn't exist
            if !target_dir.exists() {
                std::fs::create_dir_all(target_dir)?;
            }
            
            // Copy the example to the target directory
            copy_dir_all(&example_dir, target_dir)?;
            
            // We'll keep the original Cargo.toml from the Burn example
            // Just update the project name
            let cargo_toml_path = target_dir.join("Cargo.toml");
            if cargo_toml_path.exists() {
                let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path)?;
                
                // Replace workspace references with explicit values
                let updated_content = cargo_toml_content
                    // Replace project name
                    .replace("name = \"mnist\"", &format!("name = \"{}\"", project_name))
                    .replace("name = \"example\"", &format!("name = \"{}\"", project_name))
                    .replace("name = \"simple-regression\"", &format!("name = \"{}\"", project_name))
                    .replace("name = \"text-classification\"", &format!("name = \"{}\"", project_name))
                    .replace("name = \"custom-image-dataset\"", &format!("name = \"{}\"", project_name))
                    .replace("name = \"custom-training-loop\"", &format!("name = \"{}\"", project_name))
                    // Replace workspace settings with explicit values
                    .replace("edition.workspace = true", "edition = \"2021\"")
                    .replace("version.workspace = true", "version = \"0.1.0\"")
                    .replace("license.workspace = true", "license = \"MIT\"")
                    // Replace workspace dependencies with explicit versions
                    .replace("burn = { path = \"../../crates/burn\"", "burn = { version = \"0.16.0\"")
                    .replace("log = { workspace = true }", "log = \"0.4\"")
                    .replace("serde = { workspace = true", "serde = { version = \"1.0\"")
                    .replace("clap = { workspace = true", "clap = { version = \"4.5\"")
                    .replace("rand = { workspace = true }", "rand = \"0.8\"")
                    .replace("anyhow = { workspace = true }", "anyhow = \"1.0\"")
                    .replace("thiserror = { workspace = true }", "thiserror = \"1.0\"")
                    .replace("bincode = { workspace = true }", "bincode = \"1.3.3\"") // Fixed the missing closing quote
                    .replace("workspace = true", "version = \"1.0\"");
                
                std::fs::write(&cargo_toml_path, updated_content)?;
            }
            
            // Apply Burn 0.16.0 compatibility fixes to the generated code
            apply_burn_compatibility_fixes(target_dir)?;
            
            // Check if wasm32-unknown-unknown target is installed for web examples
            check_wasm_target(burn_example)?;
            
            // After copying the Burn example, we need to create a README.md with instructions
            // since the original examples might not work outside their workspace
            let readme_path = target_dir.join("README.md");
            let readme_content = format!(r#"# {} - Burn Image Recognition Project

This project is based on the official Burn MNIST example for image recognition.

## Important Note

This project is set up to demonstrate the structure and code of a Burn machine learning project. 
However, to run the project successfully, you'll need to set up a proper Burn environment.

## Next Steps

1. Visit the [Burn repository](https://github.com/tracel-ai/burn) to learn more about setting up Burn projects
2. Read the [Burn documentation](https://burn.dev/book/) for detailed guides
3. Explore the code in this project to understand the structure of a Burn ML application

## Project Structure

- `src/main.rs` - The main entry point with commands for training and testing
- `src/data.rs` - Data loading and preprocessing
- `src/model.rs` - Neural network model definition
- `src/train.rs` - Training loop implementation
- `src/args.rs` - Command-line argument parsing

This project was generated using FerrisUp.
"#, project_name);
            std::fs::write(&readme_path, readme_content)?;
            
            println!("\n✅ Project generated successfully from the Burn {} example!", burn_example);
            println!("📝 Check the README.md file in your project directory for more information.");
            
            // Display custom next steps for the Burn example
            let next_steps = get_burn_example_next_steps(burn_example, project_name);
            println!("\nNext steps:");
            for step in next_steps {
                println!("  {}", step);
            }
            
            // Skip the regular template application since we used direct copy
            return Ok(());
        }
        
        // Add the additional variables to the template variables
        if let Some(obj_mut) = template_vars.as_object_mut() {
            for (key, value) in additional_vars {
                obj_mut.insert(key, value);
            }
        }
    }
    
    // Handle edge template-specific setup
    if template_name == "edge" {
        // Process edge template options first
        println!("\n{}", "WebAssembly Edge App Configuration:".bold().green());
        
        let mut additional_vars = Map::new();
        
        if let Some(options) = template_config.get("options") {
            if let Some(obj) = options.as_object() {
                // Process testing_approach option
                if let Some(testing_approach_option) = obj.get("testing_approach") {
                    if let Some(testing_approach_obj) = testing_approach_option.as_object() {
                        let prompt = testing_approach_obj.get("prompt")
                            .and_then(|p| p.as_str())
                            .unwrap_or("How would you like to test your WebAssembly code?");
                        
                        let values = testing_approach_obj.get("values")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                            .unwrap_or_else(|| vec!["browser_example", "headless_tests", "both"]);
                        
                        // Display help text if available
                        if let Some(help_obj) = testing_approach_obj.get("help").and_then(|h| h.as_object()) {
                            println!("\nTesting approach options:");
                            for value in &values {
                                if let Some(help_text) = help_obj.get(*value).and_then(|h| h.as_str()) {
                                    println!("  {} - {}", value, help_text);
                                }
                            }
                            println!("");
                        }
                        
                        let testing_approach = prompt_with_options(prompt, &values)?;
                        additional_vars.insert("testing_approach".to_string(), json!(testing_approach));
                    }
                }
                
                // Process static_server option
                if let Some(static_server_option) = obj.get("static_server") {
                    if let Some(static_server_obj) = static_server_option.as_object() {
                        let prompt = static_server_obj.get("prompt")
                            .and_then(|p| p.as_str())
                            .unwrap_or("Which static file server would you like to use for browser testing?");
                        
                        let values = static_server_obj.get("values")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                            .unwrap_or_else(|| vec!["miniserve", "static-web-server", "none"]);
                        
                        // Display help text if available
                        if let Some(help_obj) = static_server_obj.get("help").and_then(|h| h.as_object()) {
                            println!("\nStatic file server options:");
                            for value in &values {
                                if let Some(help_text) = help_obj.get(*value).and_then(|h| h.as_str()) {
                                    println!("  {} - {}", value, help_text);
                                }
                            }
                            println!("");
                        }
                        
                        let static_server = prompt_with_options(prompt, &values)?;
                        additional_vars.insert("static_server".to_string(), json!(static_server));
                    }
                }
            }
        }
        
        // Add the additional variables to the template variables
        if let Some(obj_mut) = template_vars.as_object_mut() {
            for (key, value) in additional_vars {
                obj_mut.insert(key, value);
            }
        }
        
        // Now check for and install dependencies
        println!("🔍 Checking for wasm32-unknown-unknown target...");
        let output = Command::new("rustup")
            .args([
                "target",
                "list",
                "--installed"
            ])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        if !output_str.contains("wasm32-unknown-unknown") {
            println!("⚠️ wasm32-unknown-unknown target not found, installing...");
            let install_output = Command::new("rustup")
                .args([
                    "target",
                    "add",
                    "wasm32-unknown-unknown"
                ])
                .output()?;
            
            if install_output.status.success() {
                println!("✅ wasm32-unknown-unknown target installed successfully");
            } else {
                println!("❌ Failed to install wasm32-unknown-unknown target");
                println!("{}", String::from_utf8_lossy(&install_output.stderr));
            }
        } else {
            println!("✅ wasm32-unknown-unknown target is already installed");
        }
        
        // Check if wasm-pack is installed
        println!("🔍 Checking for wasm-pack...");
        let wasm_pack_output = Command::new("which")
            .arg("wasm-pack")
            .output()?;
        
        if !wasm_pack_output.status.success() {
            println!("⚠️ wasm-pack not found, installing...");
            let install_output = Command::new("cargo")
                .args(&["install", "wasm-pack"])
                .output()?;
            
            if install_output.status.success() {
                println!("✅ wasm-pack installed successfully");
            } else {
                println!("❌ Failed to install wasm-pack");
                println!("{}", String::from_utf8_lossy(&install_output.stderr));
            }
        } else {
            println!("✅ wasm-pack is already installed");
        }
        
        // Check if static file server is installed (if selected)
        if let Some(static_server) = template_vars.get("static_server").and_then(|s| s.as_str()) {
            if static_server != "none" {
                println!("🔍 Checking for {}...", static_server);
                let server_output = Command::new("which")
                    .arg(static_server)
                    .output()?;
                
                if !server_output.status.success() {
                    println!("⚠️ {} not found, installing...", static_server);
                    let install_output = Command::new("cargo")
                        .args(&["install", static_server])
                        .output()?;
                    
                    if install_output.status.success() {
                        println!("✅ {} installed successfully", static_server);
                    } else {
                        println!("❌ Failed to install {}", static_server);
                        println!("{}", String::from_utf8_lossy(&install_output.stderr));
                    }
                } else {
                    println!("✅ {} is already installed", static_server);
                }
            }
        }
        
        // Check if cargo-generate is installed and up-to-date
        println!("Checking for cargo-generate...");
        
        // First check if cargo-generate is installed
        let cargo_generate_check = std::process::Command::new("cargo")
            .args(["generate", "--version"])
            .output();
            
        let needs_install = if let Ok(output) = cargo_generate_check {
            if !output.status.success() {
                // Not installed
                true
            } else {
                // Installed, check version
                let version_str = String::from_utf8_lossy(&output.stdout);
                println!("Found cargo-generate: {}", version_str.trim());
                
                // Parse the version string to check if it's outdated
                // Format is typically "cargo-generate 0.X.Y"
                if let Some(version_part) = version_str.split_whitespace().nth(1) {
                    // For simplicity, we'll consider anything below 0.10.0 as outdated
                    // This can be adjusted as needed
                    let is_outdated = version_part.starts_with("0.") && 
                                     (version_part.starts_with("0.1.") || 
                                      version_part.starts_with("0.2.") || 
                                      version_part.starts_with("0.3.") || 
                                      version_part.starts_with("0.4.") || 
                                      version_part.starts_with("0.5.") || 
                                      version_part.starts_with("0.6.") || 
                                      version_part.starts_with("0.7.") || 
                                      version_part.starts_with("0.8.") || 
                                      version_part.starts_with("0.9."));
                    
                    if is_outdated {
                        println!("Detected outdated cargo-generate version. Updating...");
                        true
                    } else {
                        false
                    }
                } else {
                    // Couldn't parse version, assume it needs updating
                    true
                }
            }
        } else {
            // Command failed, assume not installed
            true
        };
        
        if needs_install {
            println!("Installing/updating cargo-generate...");
            let install_result = std::process::Command::new("cargo")
                .args(&["install", "cargo-generate", "--force"])
                .status()?;
                
            if !install_result.success() {
                return Err(anyhow!("Failed to install cargo-generate. Please install it manually with 'cargo install cargo-generate'"));
            }
            
            println!("Successfully installed/updated cargo-generate.");
        }
    }
    
    // Register handlebars helpers
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    
    // Copy template files to target directory
    if let Some(files) = template_config.get("files").and_then(|f| f.as_array()) {
        for file_entry in files {
            process_file(file_entry, &template_dir, target_dir, &template_vars, &mut handlebars)?;
        }
    }
    
    // Process dependencies if specified
    if let Some(dependencies) = template_config.get("dependencies") {
        process_dependencies(dependencies, target_dir, "dependencies")?;
    }
    
    // Display next steps if available
    if let Some(next_steps) = get_template_next_steps(template_name, project_name, Some(template_vars.clone())) {
        println!("\nNext steps:");
        for step in next_steps {
            println!("  {}", step);
        }
    }
    
    println!("✅ Successfully applied template: {}", template_name);
    
    Ok(())
}

fn process_file(
    file_entry: &Value,
    template_dir: &Path,
    target_dir: &Path,
    template_vars: &Value,
    handlebars: &mut Handlebars,
) -> Result<()> {
    if let (Some(source), Some(target)) = (
        file_entry.get("source").and_then(|s| s.as_str()),
        file_entry.get("target").and_then(|t| t.as_str()),
    ) {
        // Check if there's a condition and evaluate it
        if let Some(condition) = file_entry.get("condition").and_then(|c| c.as_str()) {
            // Parse and evaluate the condition
            let vars = template_vars.as_object().unwrap();
            
            // Simple condition evaluation for now - just check equality
            // Format: "variable_name == 'value'"
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim();
                let expected_value = parts[1].trim().trim_matches('\'').trim_matches('"');
                
                if let Some(_var_value) = vars.get(var_name) {
                    if let Some(value_str) = _var_value.as_str() {
                        if value_str != expected_value {
                            // Condition not met, skip this file
                            return Ok(());
                        }
                    }
                } else {
                    // Variable not found, skip this file
                    return Ok(());
                }
            }
        }
        
        let source_path = template_dir.join(source);
        let target_path = target_dir.join(target);
        
        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Process the file based on its extension
        if source.ends_with(".template") || source.ends_with(".rs") || source.ends_with(".md") || 
           source.ends_with(".toml") || source.ends_with(".html") || source.ends_with(".css") || 
           source.ends_with(".json") || target.ends_with("Cargo.toml") {
            // Read the template content
            let template_content = fs::read_to_string(&source_path)?;
            
            // Process conditional blocks manually before rendering with Handlebars
            let processed_content = process_conditional_blocks(&template_content, template_vars)?;
            
            // Render the template with variables using Handlebars
            let rendered = handlebars.render_template(&processed_content, template_vars)?;
            
            // Write the rendered content to the target file
            let mut file = File::create(&target_path)?;
            file.write_all(rendered.as_bytes())?;
        } else {
            // Just copy the file
            fs::copy(&source_path, &target_path)?;
            // Set executable bit for .sh files
            if let Some(ext) = target_path.extension() {
                if ext == "sh" {
                    let mut perms = fs::metadata(&target_path)?.permissions();
                    perms.set_mode(perms.mode() | 0o111); // Add execute bit
                    fs::set_permissions(&target_path, perms)?;
                }
            }
        }
    }
    
    Ok(())
}

/// Process conditional blocks in template content
fn process_conditional_blocks(content: &str, variables: &Value) -> Result<String> {
    let mut result = content.to_string();
    
    // Get the cloud provider from variables
    let cloud_provider = if let Some(provider) = variables.get("cloud_provider").and_then(|p| p.as_str()) {
        provider
    } else {
        return Ok(result);
    };
    
    // Process {{#if (eq cloud_provider "aws")}} blocks
    let providers = ["aws", "gcp", "azure", "vercel", "netlify"];
    
    for provider in providers {
        let start_tag = format!("{{{{#if (eq cloud_provider \"{}\")}}}}", provider);
        let end_tag = "{{/if}}";
        
        // Find all blocks for this provider
        let mut start_idx = 0;
        while let Some(block_start) = result[start_idx..].find(&start_tag) {
            let block_start = start_idx + block_start;
            
            // Find the matching end tag
            if let Some(block_end) = result[block_start..].find(end_tag) {
                let block_end = block_start + block_end + end_tag.len();
                
                // If this is the selected provider, keep the content but remove the tags
                if provider == cloud_provider {
                    let content_start = block_start + start_tag.len();
                    let content_end = block_end - end_tag.len();
                    
                    // Create a new string with the content but without the tags
                    let new_result = format!(
                        "{}{}{}",
                        &result[0..block_start],
                        &result[content_start..content_end],
                        &result[block_end..]
                    );
                    
                    result = new_result;
                    
                    // Adjust the start index for the next search
                    start_idx = block_start + (content_end - content_start);
                } else {
                    // This is not the selected provider, remove the entire block
                    let new_result = format!(
                        "{}{}",
                        &result[0..block_start],
                        &result[block_end..]
                    );
                    
                    result = new_result;
                    
                    // Adjust the start index for the next search
                    start_idx = block_start;
                }
            } else {
                // No matching end tag found, move past this start tag
                start_idx = block_start + start_tag.len();
            }
        }
    }
    
    Ok(result)
}

/// Apply transformations to content based on the selected variable value
#[allow(dead_code)]
fn apply_transformations(content: &str, transformations: &[Value], variables: &Value) -> Result<String> {
    let mut result = content.to_string();
    
    for transformation in transformations {
        if let Some(pattern) = transformation.get("pattern").and_then(|p| p.as_str()) {
            if let Some(replacement_value) = transformation.get("replacement") {
                // If replacement is an object, it may contain variable references
                if let Some(replacement_obj) = replacement_value.as_object() {
                    // Check for variable matches in the replacement object
                    if let Some(vars) = variables.as_object() {
                        for (var_key, _var_value) in vars {
                            if let Some(replacement) = replacement_obj.get(var_key) {
                                if let Some(replacement_str) = replacement.as_str() {
                                    result = result.replace(pattern, replacement_str);
                                }
                            }
                        }
                    }
                } else if let Some(replacement_str) = replacement_value.as_str() {
                    // Direct string replacement
                    result = result.replace(pattern, replacement_str);
                }
            }
        }
    }
    
    Ok(result)
}

/// Find the directory containing a template
pub fn find_template_directory(template_name: &str) -> Result<PathBuf> {
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
    
    // Check if it's a direct template
    let direct_path = Path::new(&templates_dir).join(template_name);
    if direct_path.exists() && direct_path.is_dir() {
        return Ok(direct_path);
    }
    
    // Check if it's a nested template (e.g., client/leptos/counter)
    let parts: Vec<&str> = template_name.split('/').collect();
    if parts.len() > 1 {
        let nested_path = Path::new(&templates_dir).join(parts.join("/"));
        if nested_path.exists() && nested_path.is_dir() {
            return Ok(nested_path);
        }
    }
    
    // Search for the template in subdirectories
    for entry in fs::read_dir(&templates_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Check if this directory contains our template
            let potential_template = path.join(template_name);
            if potential_template.exists() && potential_template.is_dir() {
                return Ok(potential_template);
            }
            
            // Check one level deeper
            if let Ok(subentries) = fs::read_dir(&path) {
                for subentry in subentries {
                    let subentry = subentry?;
                    let subpath = subentry.path();
                    
                    if subpath.is_dir() {
                        let subdir_name = subpath.file_name().unwrap().to_string_lossy();
                        if subdir_name == template_name {
                            return Ok(subpath);
                        }
                    }
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Template '{}' not found", template_name))
}

/// Get the template configuration
pub fn get_template_config(template_name: &str) -> Result<Value> {
    let template_dir = get_template_dir(template_name)?;
    
    // Read the template configuration
    let template_config_path = template_dir.join("template.json");
    let template_config_str = fs::read_to_string(&template_config_path)?;
    let template_config: Value = serde_json::from_str(&template_config_str)?;
    
    Ok(template_config)
}

/// Replace template variables in a string
#[allow(dead_code)]
fn replace_variables(content: &str, variables: &Value) -> String {
    let mut result = content.to_string();
    
    if let Some(obj) = variables.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{{{}}}}}",key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            
            result = result.replace(&placeholder, &replacement);
        }
    }
    
    result
}

/// Process dependencies from template.json
fn process_dependencies(dependencies: &Value, _target_dir: &Path, section: &str) -> Result<()> {
    if let Some(deps) = dependencies.as_object() {
        for (_key, value) in deps {
            if let Some(dep_name) = value.get("name").and_then(|n| n.as_str()) {
                let mut version = "latest".to_string();
                if let Some(ver) = value.get("version").and_then(|v| v.as_str()) {
                    version = ver.to_string();
                }
                
                println!("📦 Adding {} dependency: {} ({})", section, dep_name, version);
            }
        }
    }
    
    Ok(())
}

fn get_template_dir(template_name: &str) -> Result<PathBuf> {
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
    
    // Check if it's a direct template
    let direct_path = Path::new(&templates_dir).join(template_name);
    if direct_path.exists() && direct_path.is_dir() {
        return Ok(direct_path);
    }
    
    // Check if it's a nested template (e.g., client/leptos/counter)
    let parts: Vec<&str> = template_name.split('/').collect();
    if parts.len() > 1 {
        let nested_path = Path::new(&templates_dir).join(parts.join("/"));
        if nested_path.exists() && nested_path.is_dir() {
            return Ok(nested_path);
        }
    }
    
    // Search for the template in subdirectories
    for entry in fs::read_dir(&templates_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Check if this directory contains our template
            let potential_template = path.join(template_name);
            if potential_template.exists() && potential_template.is_dir() {
                return Ok(potential_template);
            }
            
            // Check one level deeper
            if let Ok(subentries) = fs::read_dir(&path) {
                for subentry in subentries {
                    let subentry = subentry?;
                    let subpath = subentry.path();
                    
                    if subpath.is_dir() {
                        let subdir_name = subpath.file_name().unwrap().to_string_lossy();
                        if subdir_name == template_name {
                            return Ok(subpath);
                        }
                    }
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Template '{}' not found", template_name))
}

/// Get the template's custom next steps if available
pub fn get_template_next_steps(template_name: &str, project_name: &str, variables: Option<Value>) -> Option<Vec<String>> {
    // First, check if there's a .ferrisup_next_steps.json file in the project directory
    let project_dir = PathBuf::from(project_name);
    let next_steps_file = project_dir.join(".ferrisup_next_steps.json");
    
    if next_steps_file.exists() {
        // Read the next_steps from the .ferrisup_next_steps.json file
        let next_steps_json = match fs::read_to_string(&next_steps_file) {
            Ok(content) => content,
            Err(_) => return None,
        };
        
        let next_steps_config: Value = match serde_json::from_str(&next_steps_json) {
            Ok(config) => config,
            Err(_) => return None,
        };
        
        if let Some(next_steps) = next_steps_config.get("next_steps") {
            if let Some(steps_array) = next_steps.as_array() {
                let mut result = Vec::new();
                
                // Get the variables for substitution
                let current_vars = match variables {
                    Some(vars) => {
                        if let Some(obj) = vars.as_object() {
                            obj.clone()
                        } else {
                            Map::new()
                        }
                    },
                    None => Map::new(),
                };
                
                // Process each step
                for step in steps_array {
                    if let Some(step_str) = step.as_str() {
                        let mut processed_step = step_str.to_string();
                        
                        // Replace {{project_name}} with the actual project name
                        if processed_step.contains("{{project_name}}") {
                            processed_step = processed_step.replace("{{project_name}}", project_name);
                        }
                        
                        // Replace {{static_server_command}} with the actual command if applicable
                        if processed_step.contains("{{static_server_command}}") {
                            if let Some(static_server) = current_vars.get("static_server") {
                                if let Some(server) = static_server.as_str() {
                                    if server != "none" {
                                        processed_step = processed_step.replace(
                                            "{{static_server_command}}", 
                                            &format!("{} . --port 8080", server)
                                        );
                                    } else {
                                        // Skip this step if no static server was selected
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            } else {
                                // Skip this step if static_server is not in variables
                                continue;
                            }
                        }
                        
                        result.push(processed_step);
                    }
                }
                
                // Remove the .ferrisup_next_steps.json file after reading it
                let _ = fs::remove_file(next_steps_file);
                
                return Some(result);
            }
        }
    }
    
    // If no .ferrisup_next_steps.json file exists, fall back to the template.json file
    let template_dir = match get_template_dir(template_name) {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    let template_json_path = template_dir.join("template.json");
    
    if !template_json_path.exists() {
        return None;
    }
    
    let template_json = match fs::read_to_string(&template_json_path) {
        Ok(content) => content,
        Err(_) => return None,
    };
    
    let template_config: Value = match serde_json::from_str(&template_json) {
        Ok(config) => config,
        Err(_) => return None,
    };
    
    // Get the next_steps from the template.json
    if let Some(next_steps) = template_config.get("next_steps") {
        if let Some(steps_array) = next_steps.as_array() {
            let mut result = Vec::new();
            
            // Get the variables for substitution
            let current_vars = match variables {
                Some(vars) => {
                    if let Some(obj) = vars.as_object() {
                        obj.clone()
                    } else {
                        Map::new()
                    }
                },
                None => Map::new(),
            };
            
            // Process each step
            for step in steps_array {
                if let Some(step_str) = step.as_str() {
                    let mut processed_step = step_str.to_string();
                    
                    // Replace {{project_name}} with the actual project name
                    if processed_step.contains("{{project_name}}") {
                        processed_step = processed_step.replace("{{project_name}}", project_name);
                    }
                    
                    // Replace {{static_server_command}} with the actual command if applicable
                    if processed_step.contains("{{static_server_command}}") {
                        if let Some(static_server) = current_vars.get("static_server") {
                            if let Some(server) = static_server.as_str() {
                                if server != "none" {
                                    processed_step = processed_step.replace(
                                        "{{static_server_command}}", 
                                        &format!("{} . --port 8080", server)
                                    );
                                } else {
                                    // Skip this step if no static server was selected
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        } else {
                            // Skip this step if static_server is not in variables
                            continue;
                        }
                    }
                    
                    result.push(processed_step);
                }
            }
            
            return Some(result);
        }
    }
    
    None
}

/// Prompt the user with a question and return their answer
#[allow(dead_code)]
fn prompt(question: &str) -> Result<String> {
    print!("{} ", question);
    io::stdout().flush()?;
    
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    
    Ok(line.trim().to_string())
}

/// Prompt the user with a question and a set of options
fn prompt_with_options(question: &str, options: &[&str]) -> Result<String> {
    let selection = Select::new()
        .with_prompt(question)
        .items(options)
        .default(0)
        .interact()?;
    
    Ok(options[selection].to_string())
}

/// Prompt the user with a question and a default value
fn prompt_with_default(question: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", question, default);
    io::stdout().flush()?;
    
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    
    let input = line.trim();
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c == '-' || c == '_' || c == ' ' {
            capitalize_next = true;
        } else {
            if capitalize_next {
                result.push(c.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
    }
    
    result
}

/// Recursively copy a directory to a target directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            fs::copy(&path, dst.join(entry.file_name()))?;
        } else if path.is_dir() {
            copy_dir_all(&path, &dst.join(entry.file_name()))?;
        }
    }
    
    Ok(())
}

/// Apply Burn compatibility fixes
fn apply_burn_compatibility_fixes(target_dir: &Path) -> Result<()> {
    // Fix Batcher implementations
    fix_batcher_implementations(target_dir)?;
    // Fix early stopping implementations
    fix_early_stopping_implementations(target_dir)?;
    // Fix device creation
    fix_device_creation(target_dir)?;
    // Fix module imports
    fix_module_imports(target_dir)?;
    // Fix web-specific dependencies and features
    fix_web_dependencies(target_dir)?;
    // Fix optimizer API changes
    fix_optimizer_api(target_dir)?;
    // Fix loss API changes
    fix_loss_api(target_dir)?;
    // Fix training loop API changes
    fix_training_loop_api(target_dir)?;
    // Fix model API changes
    fix_model_api(target_dir)?;
    // Apply specific patches for known problematic files
    apply_specific_patches(target_dir)?;
    
    Ok(())
}

/// Fix Batcher implementations to match Burn 0.16.0 API
fn fix_batcher_implementations(target_dir: &Path) -> Result<()> {
    // Find all Rust files in the project
    let entries = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));
    
    for entry in entries {
        let file_path = entry.path();
        let content = std::fs::read_to_string(file_path)?;
        
        // Check if the file contains Batcher implementation
        if content.contains("impl") && content.contains("Batcher") {
            // Fix Batcher trait implementation by removing the Backend type parameter
            let mut updated_content = content;
            
            // 1. Fix the Batcher trait implementation
            // Old: impl<B: Backend> Batcher<B, ItemType, BatchType<B>>
            // New: impl<B: Backend> Batcher<ItemType, BatchType<B>>
            updated_content = updated_content.replace("impl<B: Backend> Batcher<B,", "impl<B: Backend> Batcher<");
            
            // 2. Fix various batch method signature patterns
            // Pattern 1: Missing closing parenthesis after Vec<ItemType>
            updated_content = updated_content.replace("fn batch(&self, items: Vec<ItemType>", "fn batch(&self, items: Vec<ItemType>)");
            
            // Pattern 2: Extra device parameter that's no longer needed
            updated_content = updated_content.replace(">, device: &B::Device)", ")");
            updated_content = updated_content.replace(", device: &B::Device)", ")");
            
            // Pattern 3: Handle other variations of the batch method signature
            updated_content = updated_content.replace("fn batch(&self, items: Vec<", "fn batch(&self, items: Vec<");
            
            // Write the updated content back to the file
            std::fs::write(file_path, updated_content)?;
        }
    }
    
    Ok(())
}

/// Fix early stopping implementations to match Burn 0.16.0 API
fn fix_early_stopping_implementations(target_dir: &Path) -> Result<()> {
    // Find all Rust files in the project
    let entries = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));
    
    for entry in entries {
        let file_path = entry.path();
        let content = std::fs::read_to_string(file_path)?;
        
        // Check if the file contains early stopping code
        if content.contains("early_stopping") && content.contains("MetricEarlyStoppingStrategy") {
            // Fix early stopping implementation
            let mut updated_content = content.clone();
            
            // Pattern 1: Fix the metric parameter format
            // Old: .early_stopping(MetricEarlyStoppingStrategy::new(&LossMetric::new(), ...
            // New: .early_stopping(MetricEarlyStoppingStrategy::new::<LossMetric<B>>( ...
            updated_content = Regex::new(r"\.early_stopping\(MetricEarlyStoppingStrategy::new\(\s*&([a-zA-Z0-9_::<>]+)::new\(\),")
                .unwrap()
                .replace_all(&updated_content, |caps: &regex::Captures| {
                    let metric_type = &caps[1];
                    format!(".early_stopping(MetricEarlyStoppingStrategy::new::<{}>(",
                            metric_type)
                })
                .to_string();
            
            // Pattern 2: Fix the metric parameter format with backend type
            // Old: .early_stopping(MetricEarlyStoppingStrategy::new(&LossMetric::<B>::new(), ...
            // New: .early_stopping(MetricEarlyStoppingStrategy::new::<LossMetric<B>>( ...
            updated_content = Regex::new(r"\.early_stopping\(MetricEarlyStoppingStrategy::new\(\s*&([a-zA-Z0-9_]+)::<([a-zA-Z0-9_]+)>::new\(\),")
                .unwrap()
                .replace_all(&updated_content, |caps: &regex::Captures| {
                    let metric_type = &caps[1];
                    let backend_type = &caps[2];
                    format!(".early_stopping(MetricEarlyStoppingStrategy::new::<{}::<{}>>(",
                            metric_type, backend_type)
                })
                .to_string();
            
            // Pattern 3: Fix parameters order
            // Make sure Aggregate, Direction, Split are in the correct order
            updated_content = updated_content.replace(
                "StoppingCondition::NoImprovementSince { n_epochs: 1 }, Aggregate::Mean, Direction::Lowest, Split::Valid",
                "Aggregate::Mean, Direction::Lowest, Split::Valid, StoppingCondition::NoImprovementSince { n_epochs: 1 }"
            );
            
            // Write the updated content back to the file
            if updated_content != content {
                std::fs::write(file_path, updated_content)?;
            }
        }
    }
    
    Ok(())
}

/// Fix device creation to match Burn 0.16.0 API
fn fix_device_creation(target_dir: &Path) -> Result<()> {
    // Find all Rust files in the project
    let entries = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));
    
    for entry in entries {
        let file_path = entry.path();
        let content = std::fs::read_to_string(file_path)?;
        
        // Check if the file contains device creation code
        if content.contains("device") || content.contains("Device") {
            // Fix device creation
            let mut updated_content = content.clone();
            
            // Pattern 1: B::default_device() -> B::Device::default()
            updated_content = updated_content.replace(
                "B::default_device()", 
                "B::Device::default()"
            );
            
            // Pattern 2: Cpu::default_device() -> Cpu::Device::default()
            updated_content = updated_content.replace(
                "Cpu::default_device()", 
                "Cpu::Device::default()"
            );
            
            // Pattern 3: Cuda::default_device() -> Cuda::Device::default()
            updated_content = updated_content.replace(
                "Cuda::default_device()", 
                "Cuda::Device::default()"
            );
            
            // Pattern 4: Wgpu::default_device() -> Wgpu::Device::default()
            updated_content = updated_content.replace(
                "Wgpu::default_device()", 
                "Wgpu::Device::default()"
            );
            
            // Pattern 5: Candle::default_device() -> Candle::Device::default()
            updated_content = updated_content.replace(
                "Candle::default_device()", 
                "Candle::Device::default()"
            );
            
            // Pattern 6: let device = default_device() -> let device = Device::default()
            updated_content = updated_content.replace(
                "let device = default_device()", 
                "let device = Device::default()"
            );
            
            // Pattern 7: Fix device parameter in tensor creation
            updated_content = Regex::new(r"Tensor::from_data\(([^,]+),\s*device\)")
                .unwrap()
                .replace_all(&updated_content, "Tensor::from_data($1)")
                .to_string();
            
            // Pattern 8: Fix device parameter in tensor creation with ampersand
            updated_content = Regex::new(r"Tensor::from_data\(([^,]+),\s*&device\)")
                .unwrap()
                .replace_all(&updated_content, "Tensor::from_data($1)")
                .to_string();
            
            // Write the updated content back to the file if changes were made
            if updated_content != content {
                std::fs::write(file_path, updated_content)?;
            }
        }
    }
    
    Ok(())
}

/// Fix module imports to match the project name
fn fix_module_imports(target_dir: &Path) -> Result<()> {
    // Get the project name from Cargo.toml
    let cargo_toml_path = target_dir.join("Cargo.toml");
    let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path)?;
    
    // Extract the project name
    let re = Regex::new(r#"name\s*=\s*"([^"]+)""#).unwrap();
    let project_name = match re.captures(&cargo_toml_content) {
        Some(caps) => caps.get(1).unwrap().as_str().to_string(),
        None => return Err(anyhow!("Could not find project name in Cargo.toml")),
    };
    
    // Find all Rust files in the examples directory
    let examples_dir = target_dir.join("examples");
    if examples_dir.exists() {
        let entries = WalkDir::new(&examples_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));
        
        for entry in entries {
            let file_path = entry.path();
            let content = std::fs::read_to_string(file_path)?;
            
            // Replace imports of original module names with the project name
            let updated_content = content
                .replace("use mnist::", &format!("use {}::", project_name))
                .replace("use simple_regression::", &format!("use {}::", project_name))
                .replace("use text_classification::", &format!("use {}::", project_name))
                .replace("use custom_image_dataset::", &format!("use {}::", project_name))
                .replace("use custom_csv_dataset::", &format!("use {}::", project_name))
                .replace("use custom_training_loop::", &format!("use {}::", project_name))
                .replace("use image_classification_web::", &format!("use {}::", project_name));
            
            // Write the updated content back to the file
            std::fs::write(file_path, updated_content)?;
        }
    }
    
    Ok(())
}

/// Fix web-specific dependencies and features for web-based Burn examples
fn fix_web_dependencies(target_dir: &Path) -> Result<()> {
    // Path to Cargo.toml
    let cargo_toml_path = target_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(());
    }
    
    // Read Cargo.toml
    let content = std::fs::read_to_string(&cargo_toml_path)?;
    
    // Check if this is a web-based example
    if content.contains("image-classification-web") || content.contains("wasm") {
        // Add wasm-specific dependencies and features
        let mut updated_content = content.clone();
        
        // Add wasm-bindgen dependency if not present
        if !updated_content.contains("wasm-bindgen") {
            updated_content = updated_content.replace(
                "[dependencies]",
                "[dependencies]\nwasm-bindgen = \"0.2\""
            );
        }
        
        // Add web-sys dependency if not present
        if !updated_content.contains("web-sys") {
            updated_content = updated_content.replace(
                "[dependencies]",
                "[dependencies]\nweb-sys = { version = \"0.3\", features = [\"console\", \"Document\", \"Element\", \"HtmlElement\", \"Node\", \"Window\"] }"
            );
        }
        
        // Add js-sys dependency if not present
        if !updated_content.contains("js-sys") {
            updated_content = updated_content.replace(
                "[dependencies]",
                "[dependencies]\njs-sys = \"0.3\""
            );
        }
        
        // Add wasm feature to burn if not present
        if !updated_content.contains("\"wasm\"") {
            updated_content = updated_content.replace(
                "burn = { version = \"0.16.0\", features = [\"train\", \"ndarray\"",
                "burn = { version = \"0.16.0\", features = [\"train\", \"ndarray\", \"wasm\""
            );
        }
        
        // Add [lib] section for cdylib if not present
        if !updated_content.contains("[lib]") {
            updated_content = updated_content + "\n\n[lib]\ncrate-type = [\"cdylib\", \"rlib\"]\n";
        }
        
        // Write the updated content back to the file if changes were made
        if updated_content != content {
            std::fs::write(cargo_toml_path, updated_content)?;
        }
        
        // Create a .cargo/config.toml file to configure wasm-bindgen
        let cargo_config_dir = target_dir.join(".cargo");
        if !cargo_config_dir.exists() {
            std::fs::create_dir_all(&cargo_config_dir)?;
        }
        
        let cargo_config_path = cargo_config_dir.join("config.toml");
        if !cargo_config_path.exists() {
            let config_content = r#"[build]
target = "wasm32-unknown-unknown"

[target.wasm32-unknown-unknown]
runner = "wasm-bindgen-test-runner"
"#;
            std::fs::write(cargo_config_path, config_content)?;
        }
    }
    
    Ok(())
}

/// Check if the wasm32-unknown-unknown target is installed
fn check_wasm_target(burn_example: &str) -> Result<()> {
    // Only check for web-based examples
    if burn_example == "image-classification-web" {
        // Check if wasm32-unknown-unknown target is installed
        let output = Command::new("rustup")
            .args([
                "target",
                "list",
                "--installed"
            ])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // If wasm32-unknown-unknown is not installed, print instructions
        if !output_str.contains("wasm32-unknown-unknown") {
            println!("⚠️ The wasm32-unknown-unknown target is not installed, which is required for web-based examples.");
            println!("To install it, run: rustup target add wasm32-unknown-unknown");
            println!("You'll also need wasm-bindgen-cli: cargo install -f wasm-bindgen-cli");
        }
    }
    
    Ok(())
}

/// Get custom next steps for Burn examples
fn get_burn_example_next_steps(burn_example: &str, project_name: &str) -> Vec<String> {
    match burn_example {
        "mnist" => vec![
            format!("📥 Download the MNIST dataset: cd {} && cargo run --example mnist --features ndarray -- download", project_name),
            "🧠 Train the model: cargo run --example mnist --features ndarray -- train".to_string(),
            "🔍 Test with sample images: cargo run --example mnist --features ndarray -- test".to_string(),
            "🖼️ Try with your own images: cargo run --example mnist --features ndarray -- predict path/to/your/image.png".to_string(),
        ],
        "simple-regression" => vec![
            format!("📥 Generate synthetic data: cd {} && cargo run --example simple-regression --features ndarray -- generate", project_name),
            "🧠 Train the model: cargo run --example simple-regression --features ndarray -- train".to_string(),
            "🔍 Test the model: cargo run --example simple-regression --features ndarray -- test".to_string(),
        ],
        "text-classification" => vec![
            format!("📥 Download the dataset: cd {} && cargo run --example text-classification --features ndarray -- download", project_name),
            "🧠 Train the model: cargo run --example text-classification --features ndarray -- train".to_string(),
            "🔍 Test the model: cargo run --example text-classification --features ndarray -- test".to_string(),
            "📝 Classify your own text: cargo run --example text-classification --features ndarray -- predict \"Your text here\"".to_string(),
        ],
        "custom-image-dataset" => vec![
            "📥 Prepare your image dataset in the format described in README.md".to_string(),
            format!("🧠 Train the model: cd {} && cargo run --example custom-image-dataset --features ndarray -- train", project_name),
            "🔍 Test the model: cargo run --example custom-image-dataset --features ndarray -- test".to_string(),
        ],
        "custom-csv-dataset" => vec![
            "📥 Prepare your CSV dataset as described in README.md".to_string(),
            format!("🧠 Train the model: cd {} && cargo run --example custom-csv-dataset --features ndarray -- train", project_name),
            "🔍 Test the model: cargo run --example custom-csv-dataset --features ndarray -- test".to_string(),
        ],
        "custom-training-loop" => vec![
            format!("📥 Generate synthetic data: cd {} && cargo run --example custom-training-loop --features ndarray -- generate", project_name),
            "🧠 Train the model: cargo run --example custom-training-loop --features ndarray -- train".to_string(),
            "🔍 Test the model: cargo run --example custom-training-loop --features ndarray -- test".to_string(),
        ],
        "image-classification-web" => vec![
            format!("📥 Download the dataset: cd {} && cargo run --example image-classification-web --features ndarray -- download", project_name),
            "🧠 Train the model: cargo run --example image-classification-web --features ndarray -- train".to_string(),
            "🌐 Start the web server: cargo run --example image-classification-web --features ndarray -- serve".to_string(),
            "🔍 Open your browser at http://localhost:8080 to use the web interface".to_string(),
        ],
        _ => vec![
            "📝 Check the README.md file for instructions on how to use this example".to_string(),
            "🧠 Most examples can be run with: cargo run --example <example_name> --features ndarray".to_string(),
        ],
    }
}

/// Get available Burn examples
fn get_available_burn_examples(burn_repo_dir: &Path) -> Result<Vec<String>> {
    let examples_dir = burn_repo_dir.join("examples");
    let mut available_examples = Vec::new();
    
    if let Ok(entries) = fs::read_dir(&examples_dir) {
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let example_name = path.file_name().unwrap().to_string_lossy().to_string();
                available_examples.push(example_name);
            }
        }
    }
    
    Ok(available_examples)
}

/// Fix optimizer API changes in Burn 0.16.0
fn fix_optimizer_api(target_dir: &Path) -> Result<()> {
    // Find all Rust files in the project
    let entries = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));
    
    for entry in entries {
        let file_path = entry.path();
        let content = std::fs::read_to_string(file_path)?;
        
        // Check if the file contains optimizer code
        if content.contains("optim") || content.contains("Optimizer") || content.contains("SGD") || content.contains("Adam") {
            // Fix optimizer API
            let mut updated_content = content.clone();
            
            // Pattern 1: Fix SGD optimizer initialization
            // Old: SGD::new(params, SGDConfig { ... })
            // New: SGD::new(SGDConfig { ... })
            updated_content = Regex::new(r"SGD::new\(\s*([a-zA-Z0-9_]+),\s*SGDConfig")
                .unwrap()
                .replace_all(&updated_content, "SGD::new(SGDConfig")
                .to_string();
            
            // Pattern 2: Fix Adam optimizer initialization
            // Old: Adam::new(params, AdamConfig { ... })
            // New: Adam::new(AdamConfig { ... })
            updated_content = Regex::new(r"Adam::new\(\s*([a-zA-Z0-9_]+),\s*AdamConfig")
                .unwrap()
                .replace_all(&updated_content, "Adam::new(AdamConfig")
                .to_string();
            
            // Pattern 3: Fix RMSprop optimizer initialization
            // Old: RMSprop::new(params, RMSpropConfig { ... })
            // New: RMSprop::new(RMSpropConfig { ... })
            updated_content = Regex::new(r"RMSprop::new\(\s*([a-zA-Z0-9_]+),\s*RMSpropConfig")
                .unwrap()
                .replace_all(&updated_content, "RMSprop::new(RMSpropConfig")
                .to_string();
            
            // Pattern 4: Fix AdamW optimizer initialization
            // Old: AdamW::new(params, AdamWConfig { ... })
            // New: AdamW::new(AdamWConfig { ... })
            updated_content = Regex::new(r"AdamW::new\(\s*([a-zA-Z0-9_]+),\s*AdamWConfig")
                .unwrap()
                .replace_all(&updated_content, "AdamW::new(AdamWConfig")
                .to_string();
            
            // Pattern 5: Fix optimizer step method
            // Old: optimizer.step(grads, &mut params)
            // New: optimizer.step(&mut params, grads)
            updated_content = Regex::new(r"([a-zA-Z0-9_]+)\.step\(\s*([a-zA-Z0-9_]+),\s*&mut\s+([a-zA-Z0-9_]+)\s*\)")
                .unwrap()
                .replace_all(&updated_content, |caps: &regex::Captures| {
                    format!("{}.step(&mut {}, {})",
                            &caps[1], &caps[3], &caps[2])
                })
                .to_string();
            
            // Write the updated content back to the file if changes were made
            if updated_content != content {
                std::fs::write(file_path, updated_content)?;
            }
        }
    }
    
    Ok(())
}

/// Fix loss API changes in Burn 0.16.0
fn fix_loss_api(target_dir: &Path) -> Result<()> {
    // Find all Rust files in the project
    let entries = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));
    
    for entry in entries {
        let file_path = entry.path();
        let content = std::fs::read_to_string(file_path)?;
        
        // Check if the file contains loss code
        if content.contains("loss") || content.contains("Loss") || content.contains("MSE") || content.contains("CrossEntropy") {
            // Fix loss API
            let mut updated_content = content.clone();
            
            // Pattern 1: Fix MSELoss initialization
            // Old: MSELoss::new()
            // New: MSELoss::new()
            // (No change needed, but we'll keep this pattern for consistency)
            
            // Pattern 2: Fix CrossEntropyLoss initialization
            // Old: CrossEntropyLoss::new()
            // New: CrossEntropyLoss::new()
            // (No change needed, but we'll keep this pattern for consistency)
            
            // Pattern 3: Fix loss forward method
            // Old: loss.forward(output, target)
            // New: loss.forward(output, target)
            // (No change needed, but we'll keep this pattern for consistency)
            
            // Pattern 4: Fix specific loss-related imports
            updated_content = updated_content.replace(
                "use burn::loss::Loss;", 
                "use burn::loss::Loss;"
            );
            
            // Pattern 5: Fix MSELoss import
            updated_content = updated_content.replace(
                "use burn::loss::MSELoss;", 
                "use burn::loss::MSELoss;"
            );
            
            // Pattern 6: Fix CrossEntropyLoss import
            updated_content = updated_content.replace(
                "use burn::loss::CrossEntropyLoss;", 
                "use burn::loss::CrossEntropyLoss;"
            );
            
            // Pattern 7: Fix NLLLoss import
            updated_content = updated_content.replace(
                "use burn::loss::NLLLoss;", 
                "use burn::loss::NLLLoss;"
            );
            
            // Pattern 8: Fix loss reduction method
            // Old: loss.forward(output, target).mean()
            // New: loss.forward(output, target).mean()
            // (No change needed, but we'll keep this pattern for consistency)
            
            // Write the updated content back to the file if changes were made
            if updated_content != content {
                std::fs::write(file_path, updated_content)?;
            }
        }
    }
    
    Ok(())
}

/// Fix training loop API changes in Burn 0.16.0
fn fix_training_loop_api(target_dir: &Path) -> Result<()> {
    // Find all Rust files in the project
    let entries = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));
    
    for entry in entries {
        let file_path = entry.path();
        let content = std::fs::read_to_string(file_path)?;
        
        // Check if the file contains training loop code
        if content.contains("Learner") || content.contains("TrainingStep") || content.contains("train") {
            // Fix training loop API
            let mut updated_content = content.clone();
            
            // Pattern 1: Fix Learner initialization
            // Old: Learner::new(model, optimizer)
            // New: Learner::new(model)
            updated_content = Regex::new(r"Learner::new\(\s*([a-zA-Z0-9_]+),\s*([a-zA-Z0-9_]+)\s*\)")
                .unwrap()
                .replace_all(&updated_content, "Learner::new($1)")
                .to_string();
            
            // Pattern 2: Fix TrainingStep initialization
            // Old: TrainingStep::new(learner, loss)
            // New: TrainingStep::new(learner, optimizer, loss)
            updated_content = Regex::new(r"TrainingStep::new\(\s*([a-zA-Z0-9_]+),\s*([a-zA-Z0-9_]+)\s*\)")
                .unwrap()
                .replace_all(&updated_content, |caps: &regex::Captures| {
                    // We need to find the optimizer variable name
                    let learner = &caps[1];
                    let loss = &caps[2];
                    
                    // Look for optimizer variable in the content
                    let optimizer_pattern = Regex::new(r"let\s+([a-zA-Z0-9_]+)\s*=\s*[A-Za-z0-9_]+::new\(").unwrap();
                    if let Some(optimizer_caps) = optimizer_pattern.captures(&content) {
                        let optimizer = &optimizer_caps[1];
                        format!("TrainingStep::new({}, {}, {})", learner, optimizer, loss)
                    } else {
                        // If we can't find the optimizer, use a generic name
                        format!("TrainingStep::new({}, optimizer, {})", learner, loss)
                    }
                })
                .to_string();
            
            // Pattern 3: Fix Learner forward method
            // Old: learner.forward(item)
            // New: learner.forward(&item)
            updated_content = Regex::new(r"([a-zA-Z0-9_]+)\.forward\(([a-zA-Z0-9_]+)\)")
                .unwrap()
                .replace_all(&updated_content, "$1.forward(&$2)")
                .to_string();
            
            // Pattern 4: Fix TrainingStep forward method
            // Old: step.forward(batch)
            // New: step.forward(&batch)
            updated_content = Regex::new(r"([a-zA-Z0-9_]+)\.step\(([a-zA-Z0-9_]+)\)")
                .unwrap()
                .replace_all(&updated_content, "$1.step(&$2)")
                .to_string();
            
            // Pattern 5: Fix Trainer initialization
            // Old: Trainer::new(step, config)
            // New: Trainer::new(step, config)
            // (No change needed, but we'll keep this pattern for consistency)
            
            // Write the updated content back to the file if changes were made
            if updated_content != content {
                std::fs::write(file_path, updated_content)?;
            }
        }
    }
    
    Ok(())
}

/// Fix model API changes in Burn 0.16.0
fn fix_model_api(target_dir: &Path) -> Result<()> {
    // Find all Rust files in the project
    let entries = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"));
    
    for entry in entries {
        let file_path = entry.path();
        let content = std::fs::read_to_string(file_path)?;
        
        // Check if the file contains model code
        if content.contains("Module") || content.contains("model") || content.contains("forward") {
            // Fix model API
            let mut updated_content = content.clone();
            
            // Pattern 1: Fix Module trait import
            updated_content = updated_content.replace(
                "use burn::module::Module;", 
                "use burn::nn::Module;"
            );
            
            // Pattern 2: Fix module imports
            updated_content = updated_content.replace(
                "use burn::module::", 
                "use burn::nn::"
            );
            
            // Pattern 3: Fix nn module imports
            updated_content = updated_content.replace(
                "use burn::nn::", 
                "use burn::nn::"
            );
            
            // Pattern 4: Fix sequential module
            updated_content = updated_content.replace(
                "burn::module::Sequential", 
                "burn::nn::Sequential"
            );
            
            // Pattern 5: Fix custom model forward method signatures to take references
            // Old: fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4>
            // New: fn forward(&self, input: &Tensor<B, 4>) -> Tensor<B, 4>
            updated_content = Regex::new(r"fn\s+forward\(\s*&self,\s*([a-zA-Z0-9_]+)\s*:\s*Tensor<([^>]+)>\s*\)")
                .unwrap()
                .replace_all(&updated_content, "fn forward(&self, $1: &Tensor<$2>)")
                .to_string();
            
            // Pattern 6: Fix built-in module forward calls to remove references
            // These modules expect owned tensors, not references
            let built_in_modules = vec![
                "Conv1d", "Conv2d", "Conv3d", 
                "Linear", "BatchNorm", "LayerNorm", 
                "Dropout", "MaxPool1d", "MaxPool2d", "MaxPool3d", 
                "AvgPool1d", "AvgPool2d", "AvgPool3d", 
                "Gelu", "ReLU", "Sigmoid", "Tanh", "Softmax",
                "Embedding", "LSTM", "GRU", "Transformer"
            ];
            
            // Create a regex pattern for built-in modules
            let built_in_pattern = built_in_modules.join("|");
            
            // Pattern 7: Fix forward calls to built-in modules - remove references
            // Old: self.conv.forward(&x)
            // New: self.conv.forward(x)
            let regex_pattern = format!(r"(self\.[a-zA-Z0-9_]+)\.({}|conv|linear|fc|norm|activation|dropout|pool|relu|tanh|sigmoid|gelu)\.forward\(&([a-zA-Z0-9_\.]+)\)", built_in_pattern);
            updated_content = Regex::new(&regex_pattern)
                .unwrap()
                .replace_all(&updated_content, "$1.$2.forward($3)")
                .to_string();
            
            // Pattern 8: Fix static forward calls to built-in modules - remove references
            // Old: Conv2d::forward(&x)
            // New: Conv2d::forward(x)
            let regex_pattern = format!(r"({}|Conv|Linear|BatchNorm|Dropout|Activation|Pool|ReLU|Tanh|Sigmoid|Gelu)::forward\(&([a-zA-Z0-9_\.]+)\)", built_in_pattern);
            updated_content = Regex::new(&regex_pattern)
                .unwrap()
                .replace_all(&updated_content, "$1::forward($2)")
                .to_string();
            
            // Pattern 9: Fix custom model forward calls to add references
            // Old: self.forward(item.images)
            // New: self.forward(&item.images)
            updated_content = Regex::new(r"(self\.forward)\(([a-zA-Z0-9_\.]+)\)")
                .unwrap()
                .replace_all(&updated_content, |caps: &regex::Captures| {
                    // Don't modify if it's already using a reference
                    if caps[2].starts_with("&") {
                        format!("{}.forward({})", &caps[1], &caps[2])
                    } else {
                        format!("{}.forward(&{})", &caps[1], &caps[2])
                    }
                })
                .to_string();
            
            // Pattern 10: Fix custom block forward calls to add references
            // Old: self.block.forward(x)
            // New: self.block.forward(&x)
            updated_content = Regex::new(r"(self\.[a-zA-Z0-9_]+)\.forward\(([a-zA-Z0-9_\.]+)\)")
                .unwrap()
                .replace_all(&updated_content, |caps: &regex::Captures| {
                    // Skip if it's a built-in module
                    let module_name = &caps[1];
                    let input = &caps[2];
                    
                    if built_in_modules.iter().any(|m| module_name.contains(m)) || 
                       module_name.contains("conv") || module_name.contains("linear") || 
                       module_name.contains("fc") || module_name.contains("norm") || 
                       module_name.contains("activation") || module_name.contains("dropout") || 
                       module_name.contains("pool") || module_name.contains("relu") || 
                       module_name.contains("tanh") || module_name.contains("sigmoid") || 
                       module_name.contains("gelu") {
                        format!("{}.forward({})", module_name, input)
                    } else if input.starts_with("&") {
                        format!("{}.forward({})", module_name, input)
                    } else {
                        format!("{}.forward(&{})", module_name, input)
                    }
                })
                .to_string();
            
            // Write the updated content back to the file if changes were made
            if updated_content != content {
                std::fs::write(file_path, updated_content)?;
            }
        }
    }
    
    Ok(())
}

/// Apply specific patches for known problematic files in Burn examples
fn apply_specific_patches(target_dir: &Path) -> Result<()> {
    // Path to the mnist model.rs file
    let mnist_model_path = target_dir.join("src").join("model.rs");
    let mnist_data_path = target_dir.join("src").join("data.rs");
    let mnist_training_path = target_dir.join("src").join("training.rs");
    
    // Check if this is the mnist example
    if mnist_model_path.exists() {
        let content = std::fs::read_to_string(&mnist_model_path)?;
        
        // Check if this is the mnist model.rs file
        if content.contains("MnistBatch") && content.contains("Model<B>") {
            // Apply a specific patch for the mnist model.rs file
            let updated_content = r#"use crate::data::MnistBatch;
use burn::{
    nn::{BatchNorm, PaddingConfig2d, loss::CrossEntropyLossConfig},
    prelude::*,
    tensor::backend::AutodiffBackend,
    train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep},
};
use std::sync::Arc;

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    conv1: ConvBlock<B>,
    conv2: ConvBlock<B>,
    conv3: ConvBlock<B>,
    dropout: nn::Dropout,
    fc1: nn::Linear<B>,
    fc2: nn::Linear<B>,
    activation: nn::Gelu,
}

impl<B: Backend> Default for Model<B> {
    fn default() -> Self {
        let device = B::Device::default();
        Self::new(&device)
    }
}

const NUM_CLASSES: usize = 10;

impl<B: Backend> Model<B> {
    pub fn new(device: &B::Device) -> Self {
        let conv1 = ConvBlock::new([1, 8], [3, 3], device); // out: [Batch,8,26,26]
        let conv2 = ConvBlock::new([8, 16], [3, 3], device); // out: [Batch,16,24x24]
        let conv3 = ConvBlock::new([16, 24], [3, 3], device); // out: [Batch,24,22x22]
        let hidden_size = 24 * 22 * 22;
        let fc1 = nn::LinearConfig::new(hidden_size, 32)
            .with_bias(false)
            .init(device);
        let fc2 = nn::LinearConfig::new(32, NUM_CLASSES)
            .with_bias(false)
            .init(device);

        let dropout = nn::DropoutConfig::new(0.5).init();

        Self {
            conv1,
            conv2,
            conv3,
            dropout,
            fc1,
            fc2,
            activation: nn::Gelu::new(),
        }
    }

    pub fn forward(&self, input: &Tensor<B, 3>) -> Tensor<B, 2> {
        let [batch_size, height, width] = input.dims();

        let x = input.clone().reshape([batch_size, 1, height, width]).detach();
        let x = self.conv1.forward(&x);
        let x = self.conv2.forward(&x);
        let x = self.conv3.forward(&x);

        let [batch_size, channels, height, width] = x.dims();
        let x = x.reshape([batch_size, channels * height * width]);

        let x = self.dropout.forward(x);
        let x = self.fc1.forward(x);
        let x = self.activation.forward(x);

        self.fc2.forward(x)
    }

    pub fn forward_classification(&self, item: MnistBatch<B>) -> ClassificationOutput<B> {
        let targets = item.targets;
        let output = self.forward(&item.images);
        let loss = CrossEntropyLossConfig::new()
            .init(&output.device())
            .forward(output.clone(), targets.clone());

        ClassificationOutput {
            loss,
            output,
            targets,
        }
    }
}

#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    conv: nn::conv::Conv2d<B>,
    norm: BatchNorm<B, 2>,
    activation: nn::Gelu,
}

impl<B: Backend> ConvBlock<B> {
    pub fn new(channels: [usize; 2], kernel_size: [usize; 2], device: &B::Device) -> Self {
        let conv = nn::conv::Conv2dConfig::new(channels, kernel_size)
            .with_padding(PaddingConfig2d::Valid)
            .init(device);
        let norm = nn::BatchNormConfig::new(channels[1]).init(device);

        Self {
            conv,
            norm,
            activation: nn::Gelu::new(),
        }
    }

    pub fn forward(&self, input: &Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(input.clone());
        let x = self.norm.forward(x);

        self.activation.forward(x)
    }
}

impl<B: AutodiffBackend> TrainStep<MnistBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, item: MnistBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(item);

        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<MnistBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, item: MnistBatch<B>) -> ClassificationOutput<B> {
        self.forward_classification(item)
    }
}
"#;
            
            // Write the updated content to the file
            std::fs::write(mnist_model_path, updated_content)?;
        }
    }
    
    // Check if the data.rs file exists (for the mnist example)
    if mnist_data_path.exists() {
        let content = std::fs::read_to_string(&mnist_data_path)?;
        
        // Check if this is the mnist data.rs file
        if content.contains("MnistItem") && content.contains("MnistBatch") {
            // Apply a specific patch for the mnist data.rs file
            let updated_content = r#"use burn::{
    data::{dataloader::batcher::Batcher, dataset::Dataset},
    tensor::{backend::Backend, Int, Shape, Tensor, TensorData},
};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MnistItem {
    pub image: Vec<f32>,
    pub shape: [usize; 2],
    pub label: usize,
}

#[derive(Debug, Clone)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 3>,
    pub targets: Tensor<B, 1, Int>,
}

#[derive(Debug, Clone)]
pub struct MnistBatcher<B: Backend> {
    device: B::Device,
}

impl<B: Backend> MnistBatcher<B> {
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

impl<B: Backend> Default for MnistBatcher<B> {
    fn default() -> Self {
        Self::new(B::Device::default())
    }
}

impl<B: Backend> Batcher<MnistItem, MnistBatch<B>> for MnistBatcher<B> {
    fn batch(&self, items: Vec<MnistItem>) -> MnistBatch<B> {
        let batch_size = items.len();
        
        // Create a flat vector of all pixel values
        let mut image_data = Vec::with_capacity(batch_size * 28 * 28);
        for item in &items {
            image_data.extend_from_slice(&item.image);
        }
        
        // Create the images tensor using from_data
        let images = Tensor::<B, 3>::from_data(
            TensorData::new(image_data, Shape::new([batch_size, 28, 28])),
            &self.device
        );

        // Create the targets tensor using from_data
        let targets = Tensor::<B, 1, Int>::from_data(
            TensorData::new(items.iter().map(|item| item.label as i64).collect::<Vec<_>>(), Shape::new([batch_size])),
            &self.device
        );

        MnistBatch { images, targets }
    }
}

pub struct MnistDataset {
    images: Vec<MnistItem>,
}

impl MnistDataset {
    pub fn new(images: Vec<MnistItem>) -> Self {
        Self { images }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let images = std::fs::read(path.join("images.bin")).unwrap();
        let labels = std::fs::read(path.join("labels.bin")).unwrap();

        let images = images
            .chunks(28 * 28)
            .zip(labels.iter())
            .map(|(chunk, &label)| {
                let values = chunk
                    .iter()
                    .map(|&b| b as f32 / 255.0)
                    .collect::<Vec<_>>();
                
                MnistItem {
                    image: values,
                    shape: [28, 28],
                    label: label as usize,
                }
            })
            .collect::<Vec<_>>();

        Self { images }
    }
    
    pub fn train() -> Self {
        Self::from_path("data/mnist/train")
    }
    
    pub fn test() -> Self {
        Self::from_path("data/mnist/test")
    }
}

impl Dataset<MnistItem> for MnistDataset {
    fn get(&self, index: usize) -> Option<MnistItem> {
        self.images.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.images.len()
    }
}
"#;
            
            // Write the updated content to the file
            std::fs::write(mnist_data_path, updated_content)?;
        }
    }
    
    // Check if the training.rs file exists (for the mnist example)
    if mnist_training_path.exists() {
        let content = std::fs::read_to_string(&mnist_training_path)?;
        
        // Check if this is the mnist training.rs file
        if content.contains("MnistBatch") && content.contains("run<B: AutodiffBackend>") {
            // Apply a specific patch for the mnist training.rs file
            let updated_content = r#"use crate::{
    data::{MnistBatch, MnistBatcher, MnistDataset},
    model::Model,
};
use burn::{
    config::Config,
    data::dataloader::DataLoaderBuilder,
    lr_scheduler::constant::ConstantLr,
    optim::{AdamConfig, SgdConfig},
    record::{CompactRecorder, NoStdTrainingRecorder, Recorder},
    tensor::backend::AutodiffBackend,
    train::{
        metric::{AccuracyMetric, LossMetric},
        LearnerBuilder,
    },
};
use std::path::PathBuf;

#[derive(Config)]
pub struct TrainingConfig {
    pub optimizer: OptimizerConfig,
    pub batch_size: usize,
    pub num_workers: usize,
    pub epochs: usize,
    pub seed: u64,
}

#[derive(Config)]
pub enum OptimizerConfig {
    SGD(SgdConfig),
    Adam(AdamConfig),
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            optimizer: OptimizerConfig::Adam(AdamConfig::new()),
            batch_size: 64,
            num_workers: 4,
            epochs: 10,
            seed: 42,
        }
    }
}

pub fn run<B: AutodiffBackend>(device: B::Device) {
    let config = TrainingConfig::default();

    let batcher_train = MnistBatcher::new(device.clone());
    let batcher_valid = MnistBatcher::new(device.clone());

    let dataloader_train = DataLoaderBuilder::new(batcher_train)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(MnistDataset::train());

    let dataloader_test = DataLoaderBuilder::new(batcher_valid)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(MnistDataset::test());

    let model = Model::new(&device);
    
    // Create optimizer based on config
    match config.optimizer {
        OptimizerConfig::SGD(sgd_config) => {
            let optimizer = sgd_config.init();
            train_model(model, optimizer, device, dataloader_train, dataloader_test, config.epochs);
        }
        OptimizerConfig::Adam(adam_config) => {
            let optimizer = adam_config.init();
            train_model(model, optimizer, device, dataloader_train, dataloader_test, config.epochs);
        }
    }
}

fn train_model<B, M, O>(
    model: M,
    optimizer: O,
    device: B::Device,
    dataloader_train: impl Iterator<Item = MnistDataset>,
    dataloader_test: impl Iterator<Item = MnistDataset>,
    epochs: usize,
) where
    B: AutodiffBackend,
    M: AutodiffModule<B> + std::fmt::Display + 'static,
    O: burn::optim::Optimizer<M, B>,
{
    let learner = LearnerBuilder::new("mnist")
        .devices(vec![device])
        .metric_train(AccuracyMetric::new())
        .metric_valid(AccuracyMetric::new())
        .metric_train(LossMetric::new())
        .metric_valid(LossMetric::new())
        .build(model, optimizer, ConstantLr::new(1.0));

    let model_trained = learner.fit(
        dataloader_train,
        dataloader_test,
        epochs,
    );

    CompactRecorder::new()
        .record(model_trained.into_record(), PathBuf::from("model"))
        .expect("Failed to record trained model");
}
"#;
            
            // Write the updated content to the file
            std::fs::write(mnist_training_path, updated_content)?;
        }
    }
    
    Ok(())
}
