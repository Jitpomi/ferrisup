use anyhow::Result;
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
        ("data-science/polars-cli".to_string(), "Data analysis CLI using Polars (similar to pandas in Python)".to_string()),
        ("data-science/burn-net".to_string(), "Deep learning with Burn (similar to PyTorch in Python)".to_string()),
        ("data-science/linfa-lab".to_string(), "Machine learning with Linfa (similar to scikit-learn in Python)".to_string()),
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
                &["CSV files", "Parquet files", "JSON data"]
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
        } else if template_name == "data-science/burn-net" {
            println!("\n{}", "Burn Neural Network Configuration:".bold());
            
            let model_type = prompt_with_options(
                "What type of neural network model do you want to build?",
                &["Feedforward (MLP)", "Convolutional (CNN)", "Recurrent (RNN/LSTM)", "Transformer", "Custom architecture"]
            )?;
            additional_vars.insert("model_type".to_string(), json!(model_type));
            
            let dataset = prompt_with_options(
                "What dataset will you be working with?",
                &["MNIST", "CIFAR-10", "Custom dataset", "Synthetic data", "Text corpus"]
            )?;
            additional_vars.insert("dataset".to_string(), json!(dataset));
            
            let hardware = prompt_with_options(
                "Which hardware acceleration do you plan to use?",
                &["CPU only", "CUDA (NVIDIA GPU)", "Metal (Apple GPU)", "WebGPU", "Multiple backends"]
            )?;
            additional_vars.insert("hardware".to_string(), json!(hardware));
            
            println!("\n✅ Burn neural network project configured successfully!");
        } else if template_name == "data-science/linfa-lab" {
            println!("\n{}", "Linfa Machine Learning Configuration:".bold());
            
            let ml_task = prompt_with_options(
                "What machine learning task will you be working on?",
                &["Classification", "Regression", "Clustering", "Dimensionality reduction", "Multiple tasks"]
            )?;
            additional_vars.insert("ml_task".to_string(), json!(ml_task));
            
            let algorithm = prompt_with_options(
                "Which algorithm would you like to start with?",
                &["Linear models", "Decision trees", "Support vector machines", "K-means clustering", "PCA"]
            )?;
            additional_vars.insert("algorithm".to_string(), json!(algorithm));
            
            let dataset_size = prompt_with_options(
                "What is the expected size of your dataset?",
                &["Small (fits in memory)", "Medium (needs batching)", "Large (distributed processing)", "Unknown/variable"]
            )?;
            additional_vars.insert("dataset_size".to_string(), json!(dataset_size));
            
            println!("\n✅ Linfa machine learning project configured successfully!");
        }
        
        // Add the additional variables to the template variables
        if let Some(obj_mut) = template_vars.as_object_mut() {
            for (key, value) in additional_vars {
                obj_mut.insert(key, value);
            }
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
                
                if let Some(var_value) = vars.get(var_name) {
                    if let Some(value_str) = var_value.as_str() {
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
fn apply_transformations(content: &str, transformations: &[Value], variables: &Value) -> Result<String> {
    let mut result = content.to_string();
    
    for transformation in transformations {
        if let Some(pattern) = transformation.get("pattern").and_then(|p| p.as_str()) {
            if let Some(replacement_value) = transformation.get("replacement") {
                // If replacement is an object, it may contain variable references
                if let Some(replacement_obj) = replacement_value.as_object() {
                    // Check for variable matches in the replacement object
                    if let Some(vars) = variables.as_object() {
                        for (var_key, var_value) in vars {
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
    let template_config_str = fs::read_to_string(template_config_path)?;
    let template_config: Value = serde_json::from_str(&template_config_str)?;
    
    Ok(template_config)
}

/// Replace template variables in a string
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

/// Prompt the user with a question and return their answer
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
