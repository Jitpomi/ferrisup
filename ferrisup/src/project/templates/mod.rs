mod manager;

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
use shared::to_pascal_case;

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
        "client".to_string(),
        "serverless".to_string(),
        "data-science".to_string(),
        "edge".to_string(),
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
    // IMPORTANT: Only include the 8 core templates that are actually available in the new command
    let templates = vec![
        ("minimal".to_string(), "Simple binary with a single main.rs file".to_string()),
        ("library".to_string(), "Rust library crate with a lib.rs file".to_string()),
        ("embedded".to_string(), "Embedded systems firmware for microcontrollers".to_string()),
        ("server".to_string(), "Web server with API endpoints (Axum, Actix, or Poem)".to_string()),
        ("client".to_string(), "Frontend web application (Leptos, Yew, or Dioxus)".to_string()),
        ("serverless".to_string(), "Serverless function (AWS Lambda, Cloudflare Workers, etc.)".to_string()),
        ("data-science".to_string(), "Data science and machine learning projects".to_string()),
        ("edge".to_string(), "Edge computing applications (Cloudflare, Vercel, Fastly, AWS, etc.)".to_string()),
    ];
    
    // Return only the core templates without discovering additional ones
    // This ensures the list matches exactly what's shown in the new command
    
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
        "project_name_kebab": project_name.replace(" ", "-").to_lowercase(),
        "project_name_snake": project_name.replace(" ", "_").to_lowercase(),
        "project_name_pascal": to_pascal_case(project_name),
        "date": "2025-04-20",
        "year": "2025",
    });
    
    // Add user-provided variables
    if let Some(ref vars) = variables {
        if let Some(obj) = vars.as_object() {
            if let Some(obj_mut) = template_vars.as_object_mut() {
                for (_key, value) in obj {
                    obj_mut.insert(_key.clone(), value.clone());
                }
            }
        }
    }
    
    // Process template-specific options
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
    
    // Process files specified in the template.json
    if let Some(files) = template_config.get("files").and_then(|f| f.as_array()) {
        for file in files {
            if let Some(file_obj) = file.as_object() {
                let source = file_obj.get("source").and_then(|s| s.as_str());
                let target = file_obj.get("target").and_then(|t| t.as_str());
                
                if let (Some(source_path), Some(target_path)) = (source, target) {
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
                        use std::os::unix::fs::PermissionsExt;
                        let is_script = target_path.ends_with(".sh") || 
                                        content.starts_with("#!/bin/bash") ||
                                        content.starts_with("#!/usr/bin/env");
                        
                        if is_script {
                            let metadata = fs::metadata(&target_file)?;
                            let mut perms = metadata.permissions();
                            perms.set_mode(0o755); // rwxr-xr-x
                            fs::set_permissions(&target_file, perms)?;
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
    }
    
    // Apply fixes for burn templates if needed
    if template_name.starts_with("data-science/burn-") {
        apply_burn_compatibility_fixes(target_dir)?;
    }
    
    // Print successful message
    println!("\n✅ {} project created successfully!", project_name.green());
    
    if let Some(next_steps) = get_template_next_steps(template_name, project_name, Some(template_vars.clone())) {
        println!("\n{}", "Next steps:".bold().green());
        for step in next_steps {
            println!("- {}", step);
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
            
            // Determine target path - remove .template extension if present
            let target_file_name = if file_name_str.ends_with(".template") {
                file_name_str.replace(".template", "")
            } else {
                file_name_str.to_string()
            };
            
            let target_path = dst.join(&target_file_name);
            
            // Process files that need template variable substitution
            if path.extension().map_or(false, |ext| 
                ext == "template" || ext == "rs" || ext == "md" || ext == "toml" || 
                ext == "html" || ext == "css" || ext == "json" || ext == "yml" || ext == "yaml"
            ) {
                // Read template content
                let template_content = fs::read_to_string(&path)?;
                
                // Process conditional blocks
                let processed_content = process_conditional_blocks(&template_content, template_vars)?;
                
                // Render with handlebars
                let rendered = handlebars.render_template(&processed_content, template_vars)
                    .map_err(|e| anyhow!("Failed to render template: {}", e))?;
                
                // Write rendered content
                let mut file = File::create(&target_path)?;
                file.write_all(rendered.as_bytes())?;
            } else {
                // Just copy other files without processing
                fs::copy(&path, &target_path)?;
            }
            
            // Set executable bit for .sh files
            if target_path.extension().map_or(false, |ext| ext == "sh") {
                let mut perms = fs::metadata(&target_path)?.permissions();
                perms.set_mode(perms.mode() | 0o111); // Add execute bit
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

#[allow(dead_code)]
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
                        for (_var_name, _var_value) in vars {
                            if let Some(replacement) = replacement_obj.get(_var_name) {
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

#[allow(dead_code)]
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

/// Get the template configuration
pub fn get_template_config(template_name: &str) -> Result<Value> {
    let template_dir = get_template_dir(template_name)?;
    
    // Read the template configuration
    let template_config_path = template_dir.join("template.json");
    let template_config_str = fs::read_to_string(&template_config_path)?;
    let template_config: Value = serde_json::from_str(&template_config_str)?;
    
    Ok(template_config)
}

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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
                .replace_all(&updated_content, "$1.step(&mut $3, $2)")
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

/// Apply fixes for burn templates if needed
#[allow(dead_code)]
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
