use anyhow::Result;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::Write;
use serde_json::{Value, json};

pub fn get_template(name: &str) -> Result<String> {
    let templates = get_all_templates()?;
    
    if templates.contains(&name.to_string()) {
        Ok(name.to_string())
    } else {
        // Fall back to minimal if template not found
        Ok("minimal".to_string())
    }
}

pub fn get_all_templates() -> Result<Vec<String>> {
    // List all built-in templates
    let templates = vec![
        "minimal".to_string(),
        "library".to_string(),
        "full-stack".to_string(),
        "gen-ai".to_string(),
        "edge-app".to_string(),
        "embedded".to_string(),
        "serverless".to_string(),
        "iot-device".to_string(),
        "ml-pipeline".to_string(),
        "data-science".to_string(),
    ];
    
    // Check for custom templates in the templates directory
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
    if let Ok(entries) = fs::read_dir(&templates_dir) {
        let mut all_templates = templates;
        
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
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
    let mut templates = vec![
        ("minimal".to_string(), "Simple binary with a single main.rs file".to_string()),
        ("library".to_string(), "Rust library crate with a lib.rs file".to_string()),
        ("full-stack".to_string(), "Complete application with client, server, and shared libraries".to_string()),
        ("gen-ai".to_string(), "AI-focused project with inference and model components".to_string()),
        ("edge-app".to_string(), "WebAssembly-based application for edge computing".to_string()),
        ("embedded".to_string(), "Embedded systems firmware for microcontrollers".to_string()),
        ("serverless".to_string(), "Serverless functions for cloud deployment".to_string()),
        ("iot-device".to_string(), "IoT device firmware with connectivity features".to_string()),
        ("ml-pipeline".to_string(), "Machine learning data processing pipeline".to_string()),
        ("data-science".to_string(), "Data science project with analysis tools".to_string()),
    ];
    
    // Check for custom templates in the templates directory
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
    if let Ok(entries) = fs::read_dir(&templates_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    // Skip templates we've already added
                    if templates.iter().any(|(name, _)| name == dir_name) {
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
    // Create the target directory if it doesn't exist
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }
    
    // Combine default variables with provided variables
    let mut template_vars = json!({
        "project_name": project_name,
    });
    
    if let Some(vars) = variables {
        if let Some(obj) = template_vars.as_object_mut() {
            for (k, v) in vars.as_object().unwrap() {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    
    // Find the template directory
    let template_dir = find_template_directory(template_name)?;
    
    // Read the template.json file
    let template_json_path = template_dir.join("template.json");
    let template_json = fs::read_to_string(&template_json_path)
        .map_err(|e| anyhow::anyhow!("Failed to read template.json: {}", e))?;
    
    let template_config: Value = serde_json::from_str(&template_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse template.json: {}", e))?;
    
    // Process each file in the template
    if let Some(files) = template_config.get("files").and_then(|f| f.as_array()) {
        for file_entry in files {
            let source = file_entry.get("source").and_then(|s| s.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'source' in template file entry"))?;
            let target = file_entry.get("target").and_then(|t| t.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'target' in template file entry"))?;
            
            // Replace variables in target path
            let target_path = replace_variables(target, &template_vars);
            let target_file_path = target_dir.join(target_path);
            
            // Create parent directories if they don't exist
            if let Some(parent) = target_file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Read the source file
            let source_path = template_dir.join(source);
            let content = fs::read_to_string(&source_path)
                .map_err(|e| anyhow::anyhow!("Failed to read source file {}: {}", source, e))?;
            
            // Replace variables in content
            let processed_content = replace_variables(&content, &template_vars);
            
            // Write to the target file
            let mut file = File::create(&target_file_path)
                .map_err(|e| anyhow::anyhow!("Failed to create target file {}: {}", target, e))?;
            
            file.write_all(processed_content.as_bytes())
                .map_err(|e| anyhow::anyhow!("Failed to write to target file {}: {}", target, e))?;
        }
    }
    
    // Process dependencies if present
    if let Some(dependencies) = template_config.get("dependencies") {
        process_dependencies(dependencies, target_dir, "dependencies")?;
    }
    
    // Process dev-dependencies if present
    if let Some(dev_dependencies) = template_config.get("dev-dependencies") {
        process_dependencies(dev_dependencies, target_dir, "dev-dependencies")?;
    }
    
    println!("✅ Successfully applied template: {}", template_name);
    Ok(())
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
    // For now, we'll just print a message about dependencies
    // In a real implementation, you'd modify the Cargo.toml file
    
    if let Some(deps) = dependencies.as_object() {
        for (key, value) in deps {
            if let Some(deps_array) = value.as_array() {
                println!("📦 Adding {} for {}: {} items", section, key, deps_array.len());
            }
        }
    }
    
    Ok(())
}
