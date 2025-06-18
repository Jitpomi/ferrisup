use anyhow::{Result, anyhow};
use std::env;
use std::fs;
// Removed unused import
use std::io::{self, Write, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, RwLock};
use serde_json::{Value, json, Map};
use handlebars::{Handlebars, Helper, Context, RenderContext, Output, RenderError};
use colored::Colorize;
use dialoguer::Select;
use lazy_static::lazy_static;
use walkdir::WalkDir;
use regex::Regex;
// Cross-platform file permission handling
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use ferrisup_common::to_pascal_case;

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
    // Only return verified working templates
    let templates = vec![
        "minimal".to_string(),
        "library".to_string(),
        "embedded".to_string(),
        "server".to_string(),
        "client".to_string(),
        "serverless".to_string(),
        "data-science".to_string(),
    ];
    
    Ok(templates)
}

/// Returns a list of templates with their descriptions
/// Format: Vec<(name, description)>
pub fn list_templates() -> Result<Vec<(String, String)>> {
    // Define only verified working templates with descriptions
    let templates = vec![
        ("minimal".to_string(), "Simple binary with a single main.rs file".to_string()),
        ("library".to_string(), "Rust library crate with a lib.rs file".to_string()),
        ("embedded".to_string(), "Embedded systems firmware for microcontrollers".to_string()),
        ("server".to_string(), "Web server with API endpoints (Axum, Actix, or Poem)".to_string()),
        ("client".to_string(), "Frontend web application (Dioxus, Tauri, or Leptos)".to_string()),
        ("serverless".to_string(), "Serverless function (AWS Lambda, Cloudflare Workers, etc.)".to_string()),
        ("data-science".to_string(), "Data science and machine learning projects".to_string()),
    ];
    
    Ok(templates)
}

/// Get data science templates with descriptions
pub fn list_data_science_templates() -> Result<Vec<(String, String)>> {
    Ok(vec![
        // Data Analysis
        ("data-science/polars-cli".to_string(), "Data Analysis: Process and analyze data with Polars (similar to pandas)".to_string()),
        
        // Machine Learning
        ("data-science/linfa-examples".to_string(), "Machine Learning: Working examples with Linfa 0.7.1 (classification, regression, clustering)".to_string()),
        ("data-science/rustlearn-examples".to_string(), "Machine Learning: Simple ML examples with rustlearn (classification, regression, clustering)".to_string()),
    ])
}

/// Apply a template to a target directory
pub fn apply_template(template_name: &str, target_dir: &Path, project_name: &str, variables: Option<Value>) -> Result<()> {
    // Get the template configuration
    let template_config = get_template_config(template_name)?;
    
    // Check if the template has a redirect based on a variable
    if let Some(redirect) = template_config.get("redirect") {
        if let Some(redirect_obj) = redirect.as_object() {
            // If we have variables, check for redirects
            if let Some(vars) = variables.as_ref() {
                for (_key, value) in redirect_obj {
                    // For each redirect key, check if we have a matching variable
                    for (_var_name, var_value) in vars.as_object().unwrap_or(&Map::new()) {
                        if var_value.is_string() {
                            if let Some(redirect_path) = value.as_str().filter(|p| !p.is_empty()) {
                                // Apply the redirected template instead
                                return apply_template(redirect_path, target_dir, project_name, variables);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Process the template files
    let template_dir = get_template_dir(template_name)?;
    
    // Register handlebars helpers
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    
    // Register the eq helper for conditional checks
    handlebars.register_helper("eq", Box::new(|h: &handlebars::Helper<'_, '_>, 
        _: &handlebars::Handlebars<'_>, 
        _: &handlebars::Context, 
        _: &mut handlebars::RenderContext<'_, '_>, 
        out: &mut dyn handlebars::Output| 
    -> handlebars::HelperResult {
        let param1 = h.param(0).unwrap().value();
        let param2 = h.param(1).unwrap().value();
        out.write(&(param1 == param2).to_string())?;
        Ok(())
    }));
    
    // Prepare template variables
    let mut template_vars = json!({
        "project_name": project_name,
        "project_name_pascal_case": to_pascal_case(project_name),
        "project_name_snake_case": project_name.replace("-", "_"),
        "project_name_kebab_case": project_name.replace("_", "-")
    });
    
    // Add user-provided variables
    if let Some(ref vars) = variables {
        if let Some(obj) = vars.as_object() {
            if let Some(obj_mut) = template_vars.as_object_mut() {
                for (key, value) in obj {
                    obj_mut.insert(key.clone(), value.clone());
                }
            }
        }
    }
    
    // Debug output only when in verbose mode
    if std::env::var("FERRISUP_VERBOSE").is_ok() {
        println!("Template variables: {}", serde_json::to_string_pretty(&template_vars).unwrap_or_default());
    }
    
    // Skip data science template-specific prompts as they're now handled in the new.rs file
    if template_name.starts_with("data-science/") {
        // We'll use the variables passed from new.rs instead of asking again
        // This avoids duplicate prompts
        let mut additional_vars = Map::new();
        
        // Special handling for linfa-examples which has unique prompts
        if template_name == "data-science/linfa-examples" {
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
                  template_name == "data-science/burn-data-predictor" ||
                  template_name == "data-science/burn-net" {
            
            // Map our template names to the corresponding Burn examples
            let burn_example = match template_name {
                "data-science/burn-value-prediction" => "simple-regression",
                "data-science/burn-text-classifier" => "text-classification",
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
            ferrisup_common::fs::copy_directory_with_template_processing(&example_dir, target_dir)?;
            
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
            for step in next_steps {
                println!("  {}", step);
            }
            
            // Skip the regular template application since we used direct copy
            return Ok(());
        }
        
        // Add the additional variables to the template variables
        if let Some(obj_mut) = template_vars.as_object_mut() {
            for (_key, value) in additional_vars {
                obj_mut.insert(_key, value);
            }
        }
    }
    
    // Process the template-specific options
    let options = template_config.get("options").and_then(|o| o.as_array());
    if let Some(options) = options {
        let vars = template_vars.as_object_mut().unwrap();
        
        // Only prompt for options if skip_framework_prompt is not set to true
        let skip_framework_prompt = variables
            .as_ref()
            .and_then(|v| v.get("skip_framework_prompt"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        if !skip_framework_prompt {
            for option in options {
                let option_obj = option.as_object().unwrap();
                let name = option_obj.get("name").unwrap().as_str().unwrap();
                let description = option_obj.get("description").unwrap().as_str().unwrap();
                
                // Skip server_framework prompt if we're applying a server/* template directly
                if name == "server_framework" && 
                   (template_name == "server/axum" || 
                    template_name == "server/actix" || 
                    template_name == "server/poem") {
                    continue;
                }
                
                // Skip prompting if the value is already provided in variables
                if variables.as_ref().and_then(|v| v.get(name)).is_some() {
                    // Copy the value from input variables
                    if let Some(value) = variables.as_ref().and_then(|v| v.get(name)) {
                        vars.insert(name.to_string(), value.clone());
                    }
                    continue;
                }
                
                let option_type = option_obj.get("type").unwrap().as_str().unwrap();
                
                if option_type == "select" {
                    let options_array = option_obj.get("options").unwrap().as_array().unwrap();
                    let options: Vec<&str> = options_array.iter().map(|o| o.as_str().unwrap()).collect();
                    
                    let selection = Select::new()
                        .with_prompt(description)
                        .default(0)
                        .items(&options)
                        .interact()?;
                        
                    let selected = options[selection];
                    println!("Using {} as the {}", selected, name);
                    vars.insert(name.to_string(), json!(selected));
                } else if option_type == "input" {
                    let default = option_obj.get("default").map(|d| d.as_str().unwrap()).unwrap_or("");
                    let value = prompt_with_default(description, default)?;
                    vars.insert(name.to_string(), json!(value));
                } else if option_type == "boolean" {
                    let default = option_obj.get("default").map(|d| d.as_bool().unwrap()).unwrap_or(false);
                    let options = if default { &["yes", "no"] } else { &["no", "yes"] };
                    let value = prompt_with_options(description, options)?;
                    let bool_value = value == "yes";
                    vars.insert(name.to_string(), json!(bool_value));
                }
            }
        }
    }
    
    // Process conditional files if present
    if let Some(conditional_files) = template_config.get("conditional_files") {
        if let Some(conditional_files_array) = conditional_files.as_array() {
            for condition_group in conditional_files_array {
                if let Some(condition_obj) = condition_group.as_object() {
                    // Check if the condition is met
                    if let Some(when_expr) = condition_obj.get("when").and_then(|w| w.as_str()) {
                        // Simple condition evaluation for now - just check equality
                        // Format: "variable_name == 'value'"
                        let parts: Vec<&str> = when_expr.split("==").collect();
                        if parts.len() == 2 {
                            let var_name = parts[0].trim();
                            let expected_value = parts[1].trim().trim_matches('\'').trim_matches('"');
                            
                            if let Some(var_value) = template_vars.get(var_name).and_then(|v| v.as_str()) {
                                if var_value == expected_value {
                                    // Condition is met, process these files
                                    if let Some(files_array) = condition_obj.get("files").and_then(|f| f.as_array()) {
                                        for file in files_array {
                                            if let Some(file_obj) = file.as_object() {
                                                let source = file_obj.get("source").and_then(|s| s.as_str());
                                                let target = file_obj.get("target").and_then(|t| t.as_str());
                                                
                                                if let (Some(source_path), Some(target_path)) = (source, target) {
                                                    // Skip template.json file
                                                    if source_path == "template.json" || target_path == "template.json" {
                                                        continue;
                                                    }
                                                    
                                                    // Create parent directories for the target
                                                    let target_file = target_dir.join(target_path);
                                                    if let Some(parent) = target_file.parent() {
                                                        fs::create_dir_all(parent)?;
                                                    }
                                                    
                                                    // Get content from template
                                                    let source_file = template_dir.join(source_path);
                                                    
                                                    if !source_file.exists() {
                                                        return Err(anyhow!("Source file does not exist: {}", source_file.display()));
                                                    }
                                                    
                                                    // Check if this is a binary file that should be copied directly
                                                    let is_binary = source_path.ends_with(".parquet") || 
                                                                    source_path.ends_with(".bin") || 
                                                                    source_path.ends_with(".dat") ||
                                                                    source_path.ends_with(".model");
                                                    
                                                    if is_binary {
                                                        // For binary files, just copy them directly without template processing
                                                        fs::copy(&source_file, &target_file)?;
                                                    } else {
                                                        // For text files, apply template processing
                                                        let content = fs::read_to_string(&source_file)
                                                            .map_err(|e| anyhow!("Failed to read source file {}: {}", source_file.display(), e))?;
                                                        
                                                        // Apply template variables
                                                        let rendered = handlebars.render_template(&content, &template_vars)
                                                            .map_err(|e| anyhow!("Failed to render template: {}", e))?;
                                                        
                                                        // Write to target
                                                        fs::write(&target_file, rendered)?;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Process files specified in the template.json
    if let Some(files) = template_config.get("files").and_then(|f| f.as_array()) {
        for file in files {
            if let Some(file_obj) = file.as_object() {
                let source = file_obj.get("source").and_then(|s| s.as_str());
                let target = file_obj.get("target").and_then(|t| t.as_str());
                let condition = file_obj.get("condition").and_then(|c| c.as_str());
                
                if let (Some(source_path), Some(target_path)) = (source, target) {
                    // Skip template.json file
                    if source_path == "template.json" || target_path == "template.json" {
                        continue;
                    }

                    // Check if there's a condition and evaluate it
                    if let Some(condition_expr) = condition {
                        // Parse and evaluate the condition
                        let parts: Vec<&str> = condition_expr.split("==").collect();
                        if parts.len() == 2 {
                            let var_name = parts[0].trim();
                            let expected_value = parts[1].trim().trim_matches('\'').trim_matches('"');
                            
                            if let Some(var_value) = template_vars.get(var_name).and_then(|v| v.as_str()) {
                                if var_value != expected_value {
                                    // Condition not met, skip this file
                                    continue;
                                }
                            } else {
                                // Variable not found, skip this file
                                continue;
                            }
                        } else {
                            // Invalid condition format, skip this file
                            continue;
                        }
                    }

                    // Skip mcu directories that don't match the selected target
                    if let Some(mcu_target) = template_vars.get("mcu_target").and_then(|v| v.as_str()) {
                        if source_path.starts_with("mcu/") && !source_path.starts_with(&format!("mcu/{}", mcu_target)) {
                            continue;
                        }
                    }

                    // Create parent directories for the target
                    let target_file = target_dir.join(target_path);
                    if let Some(parent) = target_file.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    // Get content from template
                    let source_file = template_dir.join(source_path);
                    
                    if !source_file.exists() {
                        return Err(anyhow!("Source file does not exist: {}", source_file.display()));
                    }
                    
                    // Check if this is a binary file that should be copied directly
                    let is_binary = source_path.ends_with(".parquet") || 
                                    source_path.ends_with(".bin") || 
                                    source_path.ends_with(".dat") ||
                                    source_path.ends_with(".model");
                    
                    if is_binary {
                        // For binary files, just copy them directly without template processing
                        fs::copy(&source_file, &target_file)?;
                    } else {
                        // For text files, apply template processing
                        let content = fs::read_to_string(&source_file)
                            .map_err(|e| anyhow!("Failed to read source file {}: {}", source_file.display(), e))?;
                        
                        // Apply template variables
                        let rendered = handlebars.render_template(&content, &template_vars)
                            .map_err(|e| anyhow!("Failed to render template: {}", e))?;
                        
                        // Write to target
                        fs::write(&target_file, rendered)?;
                        
                        // If it's a script, make it executable on Unix
                        #[cfg(unix)]
                        {
                            // We already have the PermissionsExt import at the top level
                            let is_script = target_path.ends_with(".sh") || 
                                            content.starts_with("#!/bin/bash") ||
                                            content.starts_with("#!/usr/bin/env");
                            
                            if is_script {
                                let metadata = fs::metadata(&target_file)?;
                                let mut perms = metadata.permissions();
                                #[cfg(unix)]
                                perms.set_mode(0o755); // rwxr-x
                                fs::set_permissions(&target_file, perms)?;
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Process dependencies if specified
    if let Some(dependencies) = template_config.get("dependencies") {
        process_dependencies(dependencies, target_dir, "dependencies")?;
    }
    
    // For data science templates, we only want to copy the files explicitly listed in the files section
    // to avoid copying files from other data formats (CSV, JSON, Parquet)
    let skip_dir_copying = template_name.starts_with("data-science/");
    
    // Only copy directory contents for non-data-science templates
    if !skip_dir_copying {
        // Copy remaining files from the template directory
        // We need to handle template variables in all files
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        
        // Add template dir to variable set for use in templates
        if let Some(template_vars_obj) = template_vars.as_object_mut() {
            template_vars_obj.insert("template_dir".to_string(), json!(template_dir.to_string_lossy().to_string()));
        }
        
        // Process template files with variables
        process_template_directory(&template_dir, &target_dir, &template_vars, &mut handlebars)?;
        
        // Post-processing: Check for any remaining .template files that weren't processed correctly
        if let Ok(entries) = fs::read_dir(&target_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    
                    if path.is_file() && file_name_str.ends_with(".template") {
                        println!("Post-processing template file: {}", path.display());
                        
                        // Read the template file
                        let content = fs::read_to_string(&path)?;
                        
                        // Render with handlebars
                        let rendered = handlebars.render_template(&content, &template_vars)
                            .map_err(|e| anyhow!("Failed to render template {}: {}", path.display(), e))?;
                        
                        // Create the target path without .template extension
                        let new_name = file_name_str.trim_end_matches(".template");
                        let target_path = target_dir.join(new_name);
                        
                        // Write the rendered content
                        fs::write(&target_path, rendered)?;
                        
                        // Remove the original .template file
                        fs::remove_file(&path)?;
                        
                        println!("Processed template file: {} -> {}", path.display(), target_path.display());
                    }
                }
            }
        }
    }
    
    // After processing all files, clean up any files that shouldn't be in the target directory
    if let Some(mcu_target) = template_vars.get("mcu_target").and_then(|v| v.as_str()) {
        // First, ensure the correct MCU-specific files are copied to the root
        let mcu_dir = template_dir.join("mcu").join(mcu_target);
        if mcu_dir.exists() {
            // Copy the MCU-specific main.rs to src/main.rs
            let mcu_main_rs = mcu_dir.join("src").join("main.rs");
            if mcu_main_rs.exists() {
                let target_main_rs = target_dir.join("src").join("main.rs");
                // Read the source file
                let content = fs::read_to_string(&mcu_main_rs)?;
                
                // Create handlebars instance for templating
                let mut handlebars = Handlebars::new();
                handlebars.register_escape_fn(handlebars::no_escape);
                
                // Apply templating
                let rendered = handlebars.render_template(&content, &template_vars)
                    .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                    
                // Write to target file
                fs::write(target_main_rs, rendered)?;
            }
            
            // Copy the MCU-specific memory.x to memory.x
            let mcu_memory_x = mcu_dir.join("memory.x");
            if mcu_memory_x.exists() {
                let target_memory_x = target_dir.join("memory.x");
                fs::copy(&mcu_memory_x, &target_memory_x)?;
            }
            
            // Copy the MCU-specific .cargo/config.toml to .cargo/config.toml
            let mcu_cargo_config = mcu_dir.join(".cargo").join("config.toml");
            if mcu_cargo_config.exists() {
                let target_cargo_config = target_dir.join(".cargo").join("config.toml");
                // Ensure the target directory exists
                fs::create_dir_all(target_cargo_config.parent().unwrap())?;
                fs::copy(&mcu_cargo_config, &target_cargo_config)?;
            }
            
            // Copy any other MCU-specific files at the root level
            if let Ok(entries) = fs::read_dir(&mcu_dir) {
                for entry in entries {
                    let entry = entry?;
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    
                    // Skip src, .cargo, and memory.x as they're handled separately
                    if file_name_str == "src" || file_name_str == ".cargo" || file_name_str == "memory.x" {
                        continue;
                    }
                    
                    let source_path = entry.path();
                    let target_path = target_dir.join(&file_name);
                    
                    if source_path.is_dir() {
                        ferrisup_common::fs::copy_directory_with_template_processing(&source_path, &target_path)?;
                    } else {
                        // Check if it's a template file
                        if source_path.extension().map_or(false, |ext| ext == "template") {
                            // Read the source file
                            let content = fs::read_to_string(&source_path)?;
                            
                            // Create handlebars instance for templating
                            let mut handlebars = Handlebars::new();
                            handlebars.register_escape_fn(handlebars::no_escape);
                            
                            // Apply templating
                            let rendered = handlebars.render_template(&content, &template_vars)
                                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                            
                            // Write to target file (removing .template extension)
                            let target_file_name = target_path.file_stem().unwrap().to_string_lossy().to_string();
                            let target_file_path = target_dir.join(target_file_name);
                            fs::write(target_file_path, rendered)?;
                        } else {
                            fs::copy(&source_path, &target_path)?;
                        }
                    }
                }
            }
        }
        
        // Clean up mcu directories
        for mcu in &["rp2040", "stm32", "esp32", "arduino"] {
            // Remove all MCU directories
            let mcu_dir = target_dir.join("mcu").join(mcu);
            if mcu_dir.exists() {
                fs::remove_dir_all(&mcu_dir)?;
            }

            // Remove any main.rs.* files
            let main_rs_file = target_dir.join(format!("main.rs.{}", mcu));
            if main_rs_file.exists() {
                fs::remove_file(&main_rs_file)?;
            }
        }
        
        // Remove the common directory as its contents should be copied to the root
        let common_dir = target_dir.join("common");
        if common_dir.exists() {
            fs::remove_dir_all(&common_dir)?;
        }
        
        // Remove the root main.rs file (should be in src/main.rs)
        let root_main_rs = target_dir.join("main.rs");
        if root_main_rs.exists() {
            fs::remove_file(&root_main_rs)?;
        }
        
        // Remove the empty mcu directory
        let mcu_dir = target_dir.join("mcu");
        if mcu_dir.exists() && mcu_dir.is_dir() {
            fs::remove_dir_all(&mcu_dir)?;
        }
    }

    // Clean up cloud provider directories for serverless templates
    if let Some(cloud_provider) = template_vars.get("cloud_provider").and_then(|v| v.as_str()) {
        // Clean up cloud provider directories that don't match the selected provider
        for provider in &["aws", "gcp", "azure", "vercel", "netlify"] {
            if *provider != cloud_provider {
                let provider_dir = target_dir.join(provider);
                if provider_dir.exists() && provider_dir.is_dir() {
                    fs::remove_dir_all(&provider_dir)?;
                }
            }
        }
    }

    // Remove template.json if it was copied
    let template_json_file = target_dir.join("template.json");
    if template_json_file.exists() {
        fs::remove_file(&template_json_file)?;
    }
    
    // Apply fixes for burn templates if needed
    if template_name.starts_with("data-science/burn-") {
        apply_burn_compatibility_fixes(target_dir)?;
    }
    
    // Print successful message
    println!("\n✅ {} project created successfully!", project_name.green());
    
    // Check for next steps in template.json
    if let Ok(template_config) = get_template_config(template_name) {
        if let Some(next_steps) = template_config.get("next_steps").and_then(|s| s.as_array()) {
            println!("\n{}", "Next steps:".bold().green());
            
            // Create a handlebars registry for processing templates
            let mut handlebars = Handlebars::new();
            
            // Add helper functions for conditional logic
            handlebars.register_helper("eq", Box::new(|h: &Helper, _: &Handlebars, _ctx: &Context, _: &mut RenderContext, out: &mut dyn Output| -> Result<(), RenderError> {
                let param1 = h.param(0).ok_or_else(|| RenderError::new("Missing parameter 1"))?.value();
                let param2 = h.param(1).ok_or_else(|| RenderError::new("Missing parameter 2"))?.value();
                out.write(if param1 == param2 { "true" } else { "false" })?;
                Ok(())
            }));
            
            // Convert variables to a format handlebars can use
            let mut data = serde_json::Map::new();
            data.insert("project_name".to_string(), json!(project_name));
            
            // Add template variables to the data
            if let Some(ref vars) = variables {
                if let Some(obj) = vars.as_object() {
                    for (k, v) in obj {
                        data.insert(k.clone(), v.clone());
                    }
                }
                
                // Process data_format based on data_source if needed
                let data_format = if let Some(data_source) = data.get("data_source") {
                    if let Some(source) = data_source.as_str() {
                        match source {
                            "CSV files" => "csv",
                            "JSON data" => "json",
                            "Parquet files" => "parquet",
                            _ => "csv"
                        }
                    } else {
                        "csv"
                    }
                } else {
                    "csv"
                };
                
                // Always set data_format directly
                data.insert("data_format".to_string(), json!(data_format));
            }
            
            // Process each next step
            for step in next_steps {
                if let Some(step_template) = step.as_str() {
                        // First, handle direct substitutions for critical variables
                    let mut step_str = step_template.to_string();
                    
                    // Always replace project_name
                    step_str = step_str.replace("{{project_name}}", project_name);
                    
                    // Handle data_format variable directly
                    if step_str.contains("{{data_format}}") {
                        // Get the data source from the variables
                        let data_source = variables.as_ref()
                            .and_then(|v| v.get("data_source"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("CSV files");
                        
                        // Map data source to file extension
                        let extension = match data_source {
                            "CSV files" => "csv",
                            "JSON data" => "json",
                            "Parquet files" => "parquet",
                            _ => "csv"
                        };
                        
                        step_str = step_str.replace("{{data_format}}", extension);
                    }
                    
                    // Now try handlebars for any remaining variables
                    match handlebars.render_template(&step_str, &json!(data)) {
                        Ok(rendered) => {
                            println!("- {}", rendered);
                        },
                        Err(_) => {
                            // If handlebars fails, just use our direct substitutions
                            println!("- {}", step_str);
                        }
                    }
                }
            }
        }
    }
    
    // Also check for next steps in .ferrisup_next_steps.json
    let next_steps_file = target_dir.join(".ferrisup_next_steps.json");
    if next_steps_file.exists() {
        if let Ok(content) = fs::read_to_string(&next_steps_file) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(steps) = json.get("next_steps").and_then(|s| s.as_array()) {
                    println!("\n{}", "Next steps:".bold().green());
                    for step in steps {
                        if let Some(step_str) = step.as_str() {
                            // Replace {{project_name}} with the actual project name
                            let step_text = step_str.replace("{{project_name}}", project_name);
                            println!("- {}", step_text);
                        }
                    }
                    
                    // Delete the file after reading
                    let _ = fs::remove_file(&next_steps_file);
                }
            }
        }
    }
    
    Ok(())
}

/// Process template files with variable substitution in a directory
fn process_template_directory(src: &Path, dst: &Path, template_vars: &Value, handlebars: &mut Handlebars) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Check if this is a template file (has .template extension)
            let is_template_file = file_name_str.ends_with(".template");
            
            // Determine the target path (remove .template extension if present)
            let target_path = if is_template_file {
                // For template files, remove the .template extension
                let new_name = file_name_str.trim_end_matches(".template");
                let target = dst.join(new_name);
                if env::var("FERRISUP_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
                    println!("Template file will be processed: {} -> {}", path.display(), target.display());
                }
                target
            } else {
                dst.join(&*file_name_str)
            };
            
            if env::var("FERRISUP_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
                println!("Processing {}: {} -> {}", 
                       if is_template_file { "template file" } else { "regular file" },
                       path.display(), 
                       target_path.display());
            }
            
            // Always process template files, and also process files with specific extensions
            let should_process = is_template_file || {
                // For non-template files, check if they have a processable extension
                let ext = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                
                // Check if the file has a processable extension
                matches!(ext, "rs" | "md" | "toml" | "html" | "css" | "json" | "yml" | "yaml") ||
                file_name_str == "Cargo.toml" || 
                file_name_str == "Cargo.lock"
            };
            
            // Debug output only if verbose mode is enabled
            if env::var("FERRISUP_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
                println!("Should process: {} (is_template_file: {})", should_process, is_template_file);
            }
            
            if should_process {
                if env::var("FERRISUP_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
                    println!("Processing file: {} (is_template: {}) -> {}", path.display(), is_template_file, target_path.display());
                }
                
                // Read file content
                let file_content = fs::read_to_string(&path)
                    .map_err(|e| anyhow!("Failed to read file {}: {}", path.display(), e))?;
                
                // Process conditional blocks
                let processed_content = process_conditional_blocks(&file_content, template_vars)?;
                
                // Check if the file contains template variables
                let has_template_vars = processed_content.contains("{{") || 
                                       processed_content.contains("{%") ||
                                       processed_content.contains("#{") || 
                                       processed_content.contains("%}");
                
                let final_content = if has_template_vars {
                    // If it has template variables, render with Handlebars
                    handlebars.render_template(&processed_content, template_vars)
                        .map_err(|e| anyhow!("Failed to render template {}: {}", path.display(), e))?
                } else {
                    // If no template variables, use the content as-is
                    processed_content
                };
                
                // Ensure parent directory exists
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| anyhow!("Failed to create directory {}: {}", parent.display(), e))?;
                }
                
                // Write the final content to the target path
                fs::write(&target_path, final_content)
                    .map_err(|e| anyhow!("Failed to write file {}: {}", target_path.display(), e))?;
            } else {
                // Just copy other files without processing
                fs::copy(&path, &target_path)?;
                if env::var("FERRISUP_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
                    println!("Copied file: {} -> {}", path.display(), target_path.display());
                }
            }
            
            // Set executable bit for .sh files
            if target_path.extension().map_or(false, |ext| ext == "sh") {
                let mut perms = fs::metadata(&target_path)?.permissions();
                // Set executable permissions in a cross-platform way
                #[cfg(unix)]
                {
                    perms.set_mode(perms.mode() | 0o111); // Add execute bit
                }
                // On Windows, we don't need to set execute permissions explicitly
                #[cfg(not(unix))]
                {
                    // Windows doesn't have the concept of executable bit
                    // The OS determines if a file is executable based on its extension
                }
                fs::set_permissions(&target_path, perms)?;
            }
        } else if path.is_dir() {
            // Skip .git directory, .github directory, etc.
            if entry.file_name() == ".git" || entry.file_name() == ".github" || 
               entry.file_name() == "target" || entry.file_name() == "node_modules" {
                continue;
            }
            
            // Process subdirectory recursively
            process_template_directory(&path, &dst.join(entry.file_name()), template_vars, handlebars)?;
            
            // Check for any remaining .template files in the target directory
            let target_dir = dst.join(entry.file_name());
            if let Ok(target_entries) = fs::read_dir(&target_dir) {
                for target_entry in target_entries {
                    if let Ok(target_entry) = target_entry {
                        let target_path = target_entry.path();
                        let target_file_name = target_entry.file_name();
                        let target_file_str = target_file_name.to_string_lossy();
                        
                        if target_path.is_file() && target_file_str.ends_with(".template") {
                            println!("Processing remaining template file: {}", target_path.display());
                            
                            // Read the template file
                            let content = fs::read_to_string(&target_path)?;
                            
                            // Render with handlebars
                            let rendered = handlebars.render_template(&content, template_vars)
                                .map_err(|e| anyhow!("Failed to render template {}: {}", target_path.display(), e))?;
                            
                            // Create the target path without .template extension
                            let new_name = target_file_str.trim_end_matches(".template");
                            let new_target_path = target_dir.join(new_name);
                            
                            // Write the rendered content
                            fs::write(&new_target_path, rendered)?;
                            
                            // Remove the original .template file
                            fs::remove_file(&target_path)?;
                            
                            println!("Processed template file: {} -> {}", target_path.display(), new_target_path.display());
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[allow(dead_code)]
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
        let mut target_path = target_dir.join(target);
        
        // Remove .template extension from the target path if present
        if let Some(filename) = target_path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.ends_with(".template") {
                let new_filename = filename_str.replace(".template", "");
                target_path.pop(); // Remove the old filename
                target_path.push(new_filename); // Add the new filename without .template
            }
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Determine if we should process this file
        let should_process = {
            // Check if it's a template file or has a processable extension
            let is_template = source.ends_with(".template") || target.ends_with(".template");
            let has_processable_extension = [
                ".rs", ".md", ".toml", ".html", 
                ".css", ".json", ".yml", ".yaml"
            ].iter().any(|ext| source.ends_with(ext) || target.ends_with(ext));
            
            is_template || has_processable_extension || target.ends_with("Cargo.toml")
        };
        
        if should_process {
            // Read the file content
            let file_content = fs::read_to_string(&source_path)
                .map_err(|e| anyhow!("Failed to read file {}: {}", source_path.display(), e))?;
            
            // Process conditional blocks
            let processed_content = process_conditional_blocks(&file_content, template_vars)?;
            
            // Render with handlebars
            let rendered = handlebars.render_template(&processed_content, template_vars)
                .map_err(|e| anyhow!("Failed to render template {}: {}", source_path.display(), e))?;
            
            // Write rendered content to the target path
            fs::write(&target_path, rendered)?
        } else {
            // Just copy the file
            fs::copy(&source_path, &target_path)?;
            // Set executable bit for .sh files
            if let Some(ext) = target_path.extension() {
                if ext == "sh" {
                    let mut perms = fs::metadata(&target_path)?.permissions();
                    // Set executable permissions in a cross-platform way
                    #[cfg(unix)]
                    {
                        perms.set_mode(perms.mode() | 0o111); // Add execute bit
                    }
                    // On Windows, we don't need to set execute permissions explicitly
                    #[cfg(not(unix))]
                    {
                        // Windows doesn't have the concept of executable bit
                        // The OS determines if a file is executable based on its extension
                    }
                    fs::set_permissions(&target_path, perms)?;
                }
            }
        }
    }
    
    Ok(())
}

fn process_conditional_blocks(content: &str, variables: &Value) -> Result<String> {
    let mut result = content.to_string();
    
    // Process conditional blocks for cloud_provider
    if let Some(cloud_provider) = variables.get("cloud_provider").and_then(|p| p.as_str()) {
        // Process {{#if (eq cloud_provider "aws")}} blocks
        let providers = ["aws", "gcp", "azure", "vercel", "netlify"];
        
        for provider in providers {
            // Use a simpler approach to avoid format string issues
            let mut start_tag = String::from("{{#if (eq cloud_provider \"");
            start_tag.push_str(provider);
            start_tag.push_str("\")}}");
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
                    // No matching end tag found, break the loop
                    break;
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
        for (_key, value) in obj {
            let placeholder = format!("{{{{{}}}}}",_key);
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
    // Check if there's a .ferrisup_next_steps.json file in the project directory
    let project_dir = Path::new(project_name);
    let next_steps_file = project_dir.join(".ferrisup_next_steps.json");
    
    if next_steps_file.exists() {
        // Load and parse the next steps from the JSON file
        match fs::read_to_string(&next_steps_file) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(json) => {
                        if let Some(steps) = json.get("next_steps").and_then(|s| s.as_array()) {
                            let next_steps: Vec<String> = steps
                                .iter()
                                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                                .collect();
                            
                            // Delete the file after reading
                            let _ = fs::remove_file(&next_steps_file);
                            
                            if !next_steps.is_empty() {
                                return Some(next_steps);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse next steps JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read next steps file: {}", e);
            }
        }
    }
    
    // If no next_steps.json file or it's invalid, try to get from template.json
    if let Ok(template_config) = get_template_config(template_name) {
        if let Some(next_steps) = template_config.get("next_steps") {
            // Check if next_steps is an array
            if let Some(steps) = next_steps.as_array() {
                let mut result = Vec::new();
                
                for step in steps {
                    if let Some(step_str) = step.as_str() {
                        let mut step_text = step_str.to_string();
                        
                        // Replace variables in the step text
                        if let Some(vars) = &variables {
                            // Create a Handlebars instance for rendering
                            let mut handlebars = Handlebars::new();
                            handlebars.register_escape_fn(handlebars::no_escape);
                            
                            // Replace variables in the step text
                            if let Ok(rendered) = handlebars.render_template(&step_text, vars) {
                                step_text = rendered;
                            }
                        }
                        
                        // Replace {{project_name}} with the actual project name
                        step_text = step_text.replace("{{project_name}}", project_name);
                        
                        result.push(step_text);
                    }
                }
                
                if !result.is_empty() {
                    return Some(result);
                }
            }
            
            // Check if next_steps is an object with conditional steps
            if let Some(steps_obj) = next_steps.as_object() {
                // If we have variables, try to find the matching steps
                if let Some(vars) = &variables {
                    // Try to match based on data_format variable
                    if let Some(data_format) = vars.get("data_format").and_then(|f| f.as_str()) {
                        if let Some(format_steps) = steps_obj.get(data_format).and_then(|s| s.as_array()) {
                            let mut result = Vec::new();
                            
                            for step in format_steps {
                                if let Some(step_str) = step.as_str() {
                                    // Replace {{project_name}} with the actual project name
                                    let step_text = step_str.replace("{{project_name}}", project_name);
                                    result.push(step_text);
                                }
                            }
                            
                            if !result.is_empty() {
                                return Some(result);
                            }
                        }
                    }
                    
                    // Try to match based on platform variable
                    if let Some(platform) = vars.get("platform").and_then(|p| p.as_str()) {
                        if let Some(platform_steps) = steps_obj.get(platform).and_then(|s| s.as_array()) {
                            let mut result = Vec::new();
                            
                            for step in platform_steps {
                                if let Some(step_str) = step.as_str() {
                                    // Replace {{project_name}} with the actual project name
                                    let step_text = step_str.replace("{{project_name}}", project_name);
                                    result.push(step_text);
                                }
                            }
                            
                            if !result.is_empty() {
                                return Some(result);
                            }
                        }
                    }
                    
                    // Try to match based on other variables
                    for (_var_name, var_value) in vars.as_object().unwrap() {
                        if let Some(var_str) = var_value.as_str() {
                            if let Some(var_steps) = steps_obj.get(var_str).and_then(|s| s.as_array()) {
                                let mut result = Vec::new();
                                
                                for step in var_steps {
                                    if let Some(step_str) = step.as_str() {
                                        // Replace {{project_name}} with the actual project name
                                        let step_text = step_str.replace("{{project_name}}", project_name);
                                        result.push(step_text);
                                    }
                                }
                                
                                if !result.is_empty() {
                                    return Some(result);
                                }
                            }
                        }
                    }
                }
                
                // If no specific match found, try to use default steps
                if let Some(default_steps) = steps_obj.get("default").and_then(|s| s.as_array()) {
                    let mut result = Vec::new();
                    
                    for step in default_steps {
                        if let Some(step_str) = step.as_str() {
                            // Replace {{project_name}} with the actual project name
                            let step_text = step_str.replace("{{project_name}}", project_name);
                            result.push(step_text);
                        }
                    }
                    
                    if !result.is_empty() {
                        return Some(result);
                    }
                }
            }
        }
    }
    
    // Special case for Burn examples
    if template_name.starts_with("burn-") || (template_name == "burn" && variables.as_ref().and_then(|v| v.get("example")).is_some()) {
        let example = if let Some(vars) = &variables {
            vars.get("example").and_then(|e| e.as_str()).unwrap_or("mnist")
        } else {
            "mnist"
        };
        
        return Some(get_burn_example_next_steps(example, project_name));
    }
    
    // Default steps if no specific steps found
    Some(vec![
        format!("🚀 Navigate to your project: cd {}", project_name),
        "📝 Review the generated code".to_string(),
        "🔧 Build the project: cargo build".to_string(),
        "▶️ Run the project: cargo run".to_string(),
    ])
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
        .default(0)
        .items(options)
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

// Removed unused import

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
            std::fs::write(file_path, updated_content)?;
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
                .replace_all(&updated_content, "fn forward(&self, $1: &$2)")
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
    nn::{loss::CrossEntropyLossConfig, BatchNorm, PaddingConfig2d},
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

    pub fn from_record(record: &impl Record, device: &B::Device) -> Self {
        Module::from_record(record, device)
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

/// Fix MNIST data implementation to work with the latest Burn API
#[allow(dead_code)]
fn fix_mnist_data_implementation(target_dir: &Path) -> Result<()> {
    // Path to the data.rs file
    let data_path = target_dir.join("src").join("data.rs");
    
    // Check if the data.rs file exists
    if data_path.exists() {
        let content = std::fs::read_to_string(&data_path)?;
        
        // Check if this is the MNIST data.rs file
        if content.contains("MnistItem") && content.contains("MnistBatch") {
            // Apply a specific patch for the MNIST data.rs file
            let updated_content = r#"use burn::data::dataset::Dataset;
use burn::data::dataloader::{DataLoader, DataLoaderBuilder, batcher::Batcher};
use burn::tensor::{backend::Backend, Int, Tensor, Data, Shape};
use std::path::Path;
use std::sync::Arc;
use burn::record::{Recorder, CompactRecorder};
use burn::module::Module;
use burn::data::dataloader::DataLoader;

/// Normalize a single MNIST pixel value (u8 or f32) to f32 with PyTorch stats.
pub fn normalize_mnist_pixel<T: Into<f32>>(pixel: T) -> f32 {
    ((pixel.into() / 255.0) - 0.1307) / 0.3081
}

#[derive(Debug, Clone)]
pub struct MnistItem {
    pub image: Vec<f32>,
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

impl<B: Backend> Batcher<MnistItem, MnistBatch<B>> for MnistBatcher<B> {
    fn batch(&self, items: Vec<MnistItem>) -> MnistBatch<B> {
        let batch_size = items.len();
        
        // Create a flat vector of all pixel values
        let mut image_data = Vec::with_capacity(batch_size * 28 * 28);
        for item in &items {
            image_data.extend_from_slice(&item.image);
        }
        
        // Create the images tensor
        let images = Tensor::<B, 3>::from_data(
            Data::new(image_data, Shape::new([batch_size, 28, 28])),
            &self.device
        );

        // Create the targets tensor
        let targets = Tensor::<B, 1, Int>::from_data(
            Data::new(items.iter().map(|item| item.label as i64).collect::<Vec<_>>(), Shape::new([batch_size])),
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
        let images = std::fs::read(path.join("train-images-idx3-ubyte")).unwrap();
        let labels = std::fs::read(path.join("train-labels-idx1-ubyte")).unwrap();

        // Skip the header bytes: 16 for images, 8 for labels
        let images = &images[16..];
        let labels = &labels[8..];

        let images = images
            .chunks(28 * 28)
            .zip(labels.iter())
            .map(|(chunk, &label)| {
                let values = chunk
                    .iter()
                    .map(|&b| normalize_mnist_pixel(b))
                    .collect::<Vec<_>>();
                
                MnistItem {
                    image: values,
                    label: label as usize,
                }
            })
            .collect::<Vec<_>>();

        Self { images }
    }
    
    pub fn train() -> Self {
        Self::from_path("data/mnist")
    }
    
    pub fn test() -> Self {
        Self::from_path("data/mnist")
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

/// Build a `DataLoader` for MNIST training or testing data.
pub fn mnist_dataloader<B: Backend + 'static>(
    train: bool,
    device: B::Device,
    batch_size: usize,
    shuffle: Option<u64>,
    num_workers: usize,
) -> Arc<dyn DataLoader<MnistBatch<B>>> {
    let dataset = if train {
        MnistDataset::train()
    } else {
        MnistDataset::test()
    };
    let batcher = MnistBatcher::<B>::new(device);
    let mut builder = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .num_workers(num_workers);
    if let Some(seed) = shuffle {
        builder = builder.shuffle(seed);
    }
    builder.build(dataset)
}"#;
            
            // Write the updated content to the file
            std::fs::write(data_path, updated_content)?;
        }
    }
    
    Ok(())
}

/// Fix MNIST model implementation to work with the latest Burn API
#[allow(dead_code)]
fn fix_mnist_model_implementation(target_dir: &Path) -> Result<()> {
    // Path to the model.rs file
    let model_path = target_dir.join("src").join("model.rs");
    
    // Check if the model.rs file exists
    if model_path.exists() {
        let content = std::fs::read_to_string(&model_path)?;
        
        // Check if this is the MNIST model.rs file
        if content.contains("Model") && content.contains("ConvBlock") {
            // Apply a specific patch for the MNIST model.rs file
            let updated_content = r#"use crate::data::MnistBatch;
use burn::{
    module::Module,
    nn::{loss::CrossEntropyLossConfig, BatchNorm, PaddingConfig2d},
    tensor::backend::Backend,
    tensor::backend::AutodiffBackend,
    tensor::Tensor,
    train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep},
    record::Record,
};

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
        
        let x = input.clone().reshape([batch_size, 1, height, width]);
        
        let x = self.conv1.forward(&x);
        let x = self.conv2.forward(&x);
        let x = self.conv3.forward(&x);
        
        let [batch_size, channels, height, width] = x.dims();
        
        let x = x.reshape([batch_size, channels * height * width]);
        
        let x = self.dropout.forward(x);
        let x = self.fc1.forward(x);
        let x = self.activation.forward(x);
        let x = self.fc2.forward(x);
        
        x
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

    pub fn from_record(record: &impl Record, device: &B::Device) -> Self {
        Module::from_record(record, device)
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
            std::fs::write(model_path, updated_content)?;
        }
    }
    
    Ok(())
}

/// Fix MNIST training implementation to work with the latest Burn API
#[allow(dead_code)]
fn fix_mnist_training_implementation(target_dir: &Path) -> Result<()> {
    // Path to the training.rs file
    let training_path = target_dir.join("src").join("training.rs");
    
    // Check if the training.rs file exists
    if training_path.exists() {
        let content = std::fs::read_to_string(&training_path)?;
        
        // Check if this is the MNIST training.rs file
        if content.contains("train") && content.contains("evaluate") {
            // Apply a specific patch for the MNIST training.rs file
            let updated_content = r#"use crate::data::{MnistBatch, mnist_dataloader};
use crate::model::Model;
use burn::{
    tensor::backend::Backend,
    train::{
        ClassificationOutput, LearningRate, Optimizer, OptimizerConfig, StepOutput, TrainStep, ValidStep,
    },
    record::{Recorder, CompactRecorder},
};
use std::sync::Arc;

pub fn train<B: burn::tensor::backend::AutodiffBackend>(
    device: &B::Device,
    num_epochs: usize,
    batch_size: usize,
    learning_rate: f64,
    model_path: String,
) {
    // Create the model and optimizer
    let model = Model::new(device);
    let mut optimizer = burn::optim::AdamConfig::new()
        .with_learning_rate(learning_rate)
        .with_weight_decay(1e-5)
        .init();

    // Create the training and validation data loaders
    let train_loader = mnist_dataloader::<B>(true, device.clone(), batch_size, Some(42), 2);
    let valid_loader = mnist_dataloader::<B>(false, device.clone(), batch_size, None, 2);

    // Initialize the recorder
    let mut recorder = CompactRecorder::new();

    // Training loop
    for epoch in 0..num_epochs {
        let mut train_loss = 0.0;
        let mut train_acc = 0.0;
        let mut train_batches = 0;

        // Training
        for batch in train_loader.iter() {
            let output = model.step(batch);
            let batch_loss = output.loss.clone().into_scalar();
            let batch_accuracy = accuracy(output.item);

            train_loss += batch_loss;
            train_acc += batch_accuracy;
            train_batches += 1;

            // Update the model
            optimizer.update(&mut *output.model, output.gradients);

            // Print progress
            if train_batches % 100 == 0 {
                println!(
                    "Epoch: {}/{}, Batch: {}, Loss: {:.4}, Accuracy: {:.2}%",
                    epoch + 1,
                    num_epochs,
                    train_batches,
                    train_loss / train_batches as f64,
                    train_acc * 100.0 / train_batches as f64
                );
            }
        }

        // Calculate average training metrics
        train_loss /= train_batches as f64;
        train_acc /= train_batches as f64;

        // Validation
        let (val_loss, val_acc) = evaluate::<B>(&model, valid_loader.as_ref());

        println!(
            "Epoch: {}/{}, Train Loss: {:.4}, Train Acc: {:.2}%, Val Loss: {:.4}, Val Acc: {:.2}%",
            epoch + 1,
            num_epochs,
            train_loss,
            train_acc * 100.0,
            val_loss,
            val_acc * 100.0
        );
    }

    // Save the model
    recorder.record(&model);
    recorder.save(model_path).expect("Failed to save model");
}

pub fn evaluate<B: Backend>(
    model: &Model<B>,
    loader: &dyn burn::data::dataloader::DataLoader<MnistBatch<B>>,
) -> (f64, f64) {
    let mut total_loss = 0.0;
    let mut total_acc = 0.0;
    let mut num_batches = 0;

    for batch in loader.iter() {
        let output = model.step(batch);
        let batch_loss = output.loss.into_scalar();
        let batch_accuracy = accuracy(output);

        total_loss += batch_loss;
        total_acc += batch_accuracy;
        num_batches += 1;
    }

    (total_loss / num_batches as f64, total_acc / num_batches as f64)
}

fn accuracy<B: Backend>(output: ClassificationOutput<B>) -> f64 {
    let predictions = output.output.argmax(1);
    let targets = output.targets;
    
    let pred_data = predictions.to_data();
    let target_data = targets.to_data();
    
    let pred = pred_data.as_slice::<i64>().unwrap();
    let target = target_data.as_slice::<i64>().unwrap();
    
    let correct = pred.iter().zip(target.iter()).filter(|&(a, b)| a == b).count();
    correct as f64 / pred.len() as f64
}"#;
            
            // Write the updated content to the file
            std::fs::write(training_path, updated_content)?;
        }
    }
    
    Ok(())
}

/// Fix MNIST main implementation to work with the latest Burn API
#[allow(dead_code)]
fn fix_mnist_main_implementation(target_dir: &Path) -> Result<()> {
    // Path to the main.rs file
    let main_path = target_dir.join("src").join("main.rs");
    
    // Check if the main.rs file exists
    if main_path.exists() {
        let content = std::fs::read_to_string(&main_path)?;
        
        // Check if this is the MNIST main.rs file
        if content.contains("train") && content.contains("evaluate") && content.contains("predict") {
            // Apply a specific patch for the MNIST main.rs file
            let updated_content = r#"use clap::{Parser, Subcommand};
use std::path::PathBuf;
use image;

mod model;
mod data;
mod training;

use data::{MnistBatch, normalize_mnist_pixel};
use model::Model;
use training::{train, evaluate};
use burn::tensor::{backend::Backend, Tensor, Data, Shape};
use burn_autodiff::Autodiff;
use std::sync::Arc;
use burn::record::{Recorder, CompactRecorder};
use burn::module::Module;
use burn::data::dataloader::DataLoader;

// For training
// (Burn autodiff backend wrapper)
type TrainBackend = Autodiff<burn_ndarray::NdArray>;
type InferenceBackend = burn_ndarray::NdArray;

#[derive(Parser)]
#[command(subcommand)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Train {
        #[arg(short, long, default_value = "10")]
        num_epochs: usize,
        #[arg(short, long, default_value = "64")]
        batch_size: usize,
        #[arg(short, long, default_value = "0.001")]
        learning_rate: f64,
        #[arg(short, long, default_value = "./model.json")]
        model_path: PathBuf,
    },
    Evaluate {
        #[arg(short, long)]
        model_path: PathBuf,
        #[arg(short, long, default_value = "64")]
        batch_size: usize,
    },
    Predict {
        #[arg(short, long)]
        model_path: PathBuf,
        #[arg(short, long)]
        image_path: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Train { num_epochs, batch_size, learning_rate, model_path } => {
            println!("🚀 Training MNIST digit recognition model");
            // Check for MNIST data presence
            if !std::path::Path::new("./data/mnist/train-images-idx3-ubyte").exists() {
                eprintln!("❌ MNIST data not found. Please run ./download_mnist.sh before training.");
                std::process::exit(1);
            }
            let device = <TrainBackend as Backend>::Device::default();
            train::<TrainBackend>(
                &device,
                num_epochs,
                batch_size,
                learning_rate,
                model_path.to_string_lossy().to_string(),
            );
            println!("✅ Training completed successfully!");
        },
        Commands::Evaluate { model_path, batch_size } => {
            println!("🔍 Evaluating MNIST digit recognition model");
            // Check for model file presence
            if !model_path.exists() {
                eprintln!("❌ Model file not found: {}", model_path.display());
                std::process::exit(1);
            }
            let device = <InferenceBackend as Backend>::Device::default();
            let record = CompactRecorder::new().load(model_path.to_path_buf(), &device)?;
            let model = Model::<InferenceBackend>::from_record(&record, &device);
            let test_loader: Arc<dyn DataLoader<MnistBatch<InferenceBackend>>> = data::mnist_dataloader::<InferenceBackend>(false, device.clone(), batch_size, None, 2);
            let (loss, accuracy) = evaluate::<InferenceBackend>(&model, test_loader.as_ref());
            println!("📊 Test accuracy: {:.2}%", accuracy * 100.0);
            println!("📉 Test loss: {:.4}", loss);
        },
        Commands::Predict { model_path, image_path } => {
            println!("🔮 Predicting digit from image");
            // Check for model file presence
            if !model_path.exists() {
                eprintln!("❌ Model file not found: {}", model_path.display());
                std::process::exit(1);
            }
            if !image_path.exists() {
                eprintln!("❌ Image file not found: {}", image_path.display());
                std::process::exit(1);
            }
            let device = <InferenceBackend as Backend>::Device::default();
            let record = CompactRecorder::new().load(model_path.to_path_buf(), &device)?;
            let model = Model::<InferenceBackend>::from_record(&record, &device);
            let image = image::open(image_path)?.to_luma8();
            let image = if image.dimensions() != (28, 28) {
                image::imageops::resize(&image, 28, 28, image::imageops::FilterType::Nearest)
            } else {
                image
            };
            let image_data: Vec<f32> = image.pixels().map(|p| normalize_mnist_pixel(p[0])).collect();
            let input = Tensor::<InferenceBackend, 3>::from_data(
                Data::new(image_data, Shape::new([1, 28, 28])),
                &device
            );
            let output = model.forward(&input);
            let pred_data = output.argmax(1).to_data();
            let pred_slice = pred_data.as_slice::<i64>().unwrap_or(&[0]);
            let pred = pred_slice[0];
            println!("Predicted digit: {}", pred);
        }
    }
    Ok(())
}"#;
            
            // Write the updated content to the file
            std::fs::write(main_path, updated_content)?;
        }
    }
    
    Ok(())
}
