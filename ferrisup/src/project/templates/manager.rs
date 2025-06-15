// Template manager for applying templates and retrieving next steps
use anyhow::{Result, Context};
use handlebars::Handlebars;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::fs;
// Removed unused import
use std::collections::HashSet;
use colored::Colorize;
use shared::cargo::*;

/// Get all available templates
#[allow(dead_code)]
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

/// List available templates with their descriptions
#[allow(dead_code)]
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
    
    // Get the templates directory path
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
    if let Ok(entries) = fs::read_dir(&templates_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    // Skip if already in the list
                    if templates.iter().any(|(name, _)| name == dir_name) {
                        continue;
                    }
                    
                    // Check for template.json to get description
                    let template_json = entry.path().join("template.json");
                    if template_json.exists() {
                        if let Ok(content) = fs::read_to_string(&template_json) {
                            if let Ok(config) = serde_json::from_str::<Value>(&content) {
                                if let Some(description) = config.get("description").and_then(|d| d.as_str()) {
                                    templates.push((dir_name.to_string(), description.to_string()));
                                    continue;
                                }
                            }
                        }
                    }
                    
                    // Default description if not found
                    templates.push((dir_name.to_string(), "Custom template".to_string()));
                }
            }
        }
    }
    
    Ok(templates)
}

/// Find a template directory by name
#[allow(dead_code)]
pub fn find_template_directory(template_name: &str) -> Result<PathBuf> {
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
    
    // Check for direct match
    let direct_path = PathBuf::from(&templates_dir).join(template_name);
    if direct_path.exists() && direct_path.is_dir() {
        return Ok(direct_path);
    }
    
    // Check for subdirectory match (e.g. data-science/polars-cli)
    let parts: Vec<&str> = template_name.split('/').collect();
    if parts.len() > 1 {
        let category = parts[0];
        let subtemplate = parts[1];
        
        let subdir_path = PathBuf::from(&templates_dir).join(category).join(subtemplate);
        if subdir_path.exists() && subdir_path.is_dir() {
            return Ok(subdir_path);
        }
    }
    
    // Try to find in subdirectories
    for entry in fs::read_dir(&templates_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            if let Some(dir_name) = entry.file_name().to_str() {
                // Check if this directory contains our template
                let subdir_path = entry.path().join(template_name);
                if subdir_path.exists() && subdir_path.is_dir() {
                    return Ok(subdir_path);
                }
                
                // Check for partial matches
                for subentry in fs::read_dir(entry.path())? {
                    let subentry = subentry?;
                    if subentry.path().is_dir() {
                        if let Some(subdir_name) = subentry.file_name().to_str() {
                            if subdir_name == template_name || format!("{}/{}", dir_name, subdir_name) == template_name {
                                return Ok(subentry.path());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Template '{}' not found", template_name))
}

/// Get template configuration from template.json
#[allow(dead_code)]
pub fn get_template_config(template_name: &str) -> Result<Value> {
    let template_dir = find_template_directory(template_name)?;
    let template_json = template_dir.join("template.json");
    
    if !template_json.exists() {
        return Err(anyhow::anyhow!("Template configuration not found for '{}'", template_name));
    }
    
    let content = fs::read_to_string(&template_json)?;
    let config = serde_json::from_str(&content)?;
    
    Ok(config)
}

/// Apply a template to a target directory
#[allow(dead_code)]
pub fn apply_template(
    template_name: &str,
    target_dir: &Path,
    project_name: &str,
    variables: Option<Value>,
) -> Result<()> {
    println!("Applying template {} to {}", template_name, target_dir.display());
    
    // Check if the template exists
    if !template_exists(template_name) {
        return Err(anyhow::anyhow!("Template '{}' not found", template_name));
    }

    // Read the template configuration
    let template_config = get_template_config(template_name)?;

    // Get the template files to copy
    let files = match template_config.get("files") {
        Some(Value::Array(files)) => files,
        _ => return Err(anyhow::anyhow!("No files specified in template")),
    };

    // Get the template directory
    let template_dir = get_template_dir(template_name)?;

    // Create a Handlebars instance for rendering templates
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    // Prepare the template variables
    let mut template_vars = json!({
        "project_name": project_name,
    });

    // Add any additional variables
    if let Some(vars) = variables {
        if let Some(obj) = vars.as_object() {
            for (k, v) in obj {
                template_vars[k] = v.clone();
            }
        }
    }

    // Create a set to track which files we've processed
    let mut processed_files = HashSet::new();

    // Process each file in the template
    for file in files {
        let source = match file.get("source") {
            Some(Value::String(source)) => source,
            _ => continue,
        };

        let target = match file.get("target") {
            Some(Value::String(target)) => target,
            _ => continue,
        };

        // Skip template.json file
        if source == "template.json" || target == "template.json" {
            continue;
        }

        // Skip mcu directories that don't match the selected target
        if let Some(mcu_target) = template_vars.get("mcu_target").and_then(|v| v.as_str()) {
            if source.starts_with("mcu/") && !source.starts_with(&format!("mcu/{}", mcu_target)) {
                continue;
            }
        }

        // Apply variables to the file source and target paths
        let source_rendered = handlebars.render_template(source, &template_vars)
            .unwrap_or_else(|_| source.clone());
        let target_rendered = handlebars.render_template(target, &template_vars)
            .unwrap_or_else(|_| target.clone());

        // Calculate the absolute paths
        let source_abs_path = template_dir.join(&source_rendered);
        let target_abs_path = target_dir.join(&target_rendered);

        // Add this file to the processed set
        processed_files.insert(target_abs_path.to_string_lossy().to_string());

        // Skip if the source file doesn't exist
        if !source_abs_path.exists() {
            println!("Warning: Source file does not exist: {}", source_abs_path.display());
            continue;
        }

        // Create parent directory if needed
        if let Some(parent) = target_abs_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if the file has a .template extension - if so, apply variable substitution
        if source_abs_path.extension().map_or(false, |ext| ext == "template") {
            // Read the template content
            let template_content = match fs::read_to_string(&source_abs_path) {
                Ok(content) => content,
                Err(e) => {
                    println!("Error reading template file {}: {}", source_abs_path.display(), e);
                    continue;
                }
            };

            // Render the template with variables
            let rendered = match handlebars.render_template(&template_content, &template_vars) {
                Ok(rendered) => rendered,
                Err(e) => {
                    println!("Error rendering template {}: {}", source_abs_path.display(), e);
                    continue;
                }
            };

            // Write the rendered template to the target file
            match fs::write(&target_abs_path, rendered) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error writing file {}: {}", target_abs_path.display(), e);
                    continue;
                }
            }
        } else {
            // Copy the file without modification
            if let Err(e) = fs::copy(&source_abs_path, &target_abs_path) {
                println!("Error copying file from {} to {}: {}", source_abs_path.display(), target_abs_path.display(), e);
                continue;
            }
        }
    }

    // Process conditional files if present
    if let Some(conditional_files) = template_config.get("conditional_files") {
        if let Some(conditional_files_array) = conditional_files.as_array() {
            // For each conditional group
            for condition_group in conditional_files_array {
                if let Some(condition_obj) = condition_group.as_object() {
                    // Check if the condition is met
                    if let Some(when_expr) = condition_obj.get("when").and_then(|w| w.as_str()) {
                        if let Some((var_name, expected_value)) = parse_condition(when_expr) {
                            // Check if the variable exists and matches the expected value
                            if let Some(actual_value) = template_vars.get(var_name).and_then(|v| v.as_str()) {
                                if actual_value == expected_value {
                                    // If condition matches, add the files
                                    if let Some(files_array) = condition_obj.get("files").and_then(|f| f.as_array()) {
                                        for file in files_array {
                                            let source = match file.get("source") {
                                                Some(Value::String(source)) => source,
                                                _ => continue,
                                            };

                                            let target = match file.get("target") {
                                                Some(Value::String(target)) => target,
                                                _ => continue,
                                            };

                                            // Skip template.json file
                                            if source == "template.json" || target == "template.json" {
                                                continue;
                                            }

                                            // Skip mcu directories that don't match the selected target
                                            if let Some(mcu_target) = template_vars.get("mcu_target").and_then(|v| v.as_str()) {
                                                if source.starts_with("mcu/") && !source.starts_with(&format!("mcu/{}", mcu_target)) {
                                                    continue;
                                                }
                                            }

                                            // Apply variables to the file source and target paths
                                            let source_rendered = handlebars.render_template(source, &template_vars)
                                                .unwrap_or_else(|_| source.clone());
                                            let target_rendered = handlebars.render_template(target, &template_vars)
                                                .unwrap_or_else(|_| target.clone());

                                            // Calculate the absolute paths
                                            let source_abs_path = template_dir.join(&source_rendered);
                                            let target_abs_path = target_dir.join(&target_rendered);

                                            // Create parent directory if needed
                                            if let Some(parent) = target_abs_path.parent() {
                                                fs::create_dir_all(parent)?;
                                            }

                                            // Add this file to the processed set
                                            processed_files.insert(target_abs_path.to_string_lossy().to_string());

                                            // Skip if the source file doesn't exist
                                            if !source_abs_path.exists() {
                                                println!("Warning: Source file does not exist: {}", source_abs_path.display());
                                                continue;
                                            }

                                            // Check if the file has a .template extension - if so, apply variable substitution
                                            if source_abs_path.extension().map_or(false, |ext| ext == "template") {
                                                // Read the template content
                                                let template_content = match fs::read_to_string(&source_abs_path) {
                                                    Ok(content) => content,
                                                    Err(e) => {
                                                        println!("Error reading template file {}: {}", source_abs_path.display(), e);
                                                        continue;
                                                    }
                                                };

                                                // Render the template with variables
                                                let rendered = match handlebars.render_template(&template_content, &template_vars) {
                                                    Ok(rendered) => rendered,
                                                    Err(e) => {
                                                        println!("Error rendering template {}: {}", source_abs_path.display(), e);
                                                        continue;
                                                    }
                                                };

                                                // Write the rendered template to the target file
                                                match fs::write(&target_abs_path, rendered) {
                                                    Ok(_) => (),
                                                    Err(e) => {
                                                        println!("Error writing file {}: {}", target_abs_path.display(), e);
                                                        continue;
                                                    }
                                                }
                                            } else {
                                                // Copy the file without modification
                                                if let Err(e) = fs::copy(&source_abs_path, &target_abs_path) {
                                                    println!("Error copying file from {} to {}: {}", source_abs_path.display(), target_abs_path.display(), e);
                                                    continue;
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

    // After processing all files, clean up any files that shouldn't be in the target directory
    if let Some(mcu_target) = template_vars.get("mcu_target").and_then(|v| v.as_str()) {
        // Clean up mcu directories that don't match the selected target
        for mcu in &["rp2040", "stm32", "esp32", "arduino"] {
            if *mcu != mcu_target {
                let mcu_dir = target_dir.join("mcu").join(mcu);
                if mcu_dir.exists() {
                    fs::remove_dir_all(&mcu_dir)?;
                }

                // Also remove any main.rs.* files that don't match the selected target
                let main_rs_file = target_dir.join("src").join(format!("main.rs.{}", mcu));
                if main_rs_file.exists() {
                    fs::remove_file(&main_rs_file)?;
                }
            }
        }
    }

    // Remove template.json if it was copied
    let template_json_file = target_dir.join("template.json");
    if template_json_file.exists() {
        fs::remove_file(&template_json_file)?;
    }

    // Copy dependencies to the Cargo.toml file if needed
    if let Some(deps) = get_template_dependencies(template_name, &template_vars) {
        update_cargo_toml(target_dir, &deps)?;
    }

    // Display next steps if available
    if let Some(next_steps) = get_template_next_steps(template_name, project_name, Some(template_vars.clone())) {
        println!("\n{}", "Next steps:".bold().green());
        for step in next_steps {
            println!("- {}", step);
        }
    }

    // Success
    Ok(())
}

/// Get next steps for a template
#[allow(dead_code)]
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
            // Check if next_steps is a simple array (old format)
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
            
            // Check if next_steps is an object with the new format structure
            if let Some(steps_obj) = next_steps.as_object() {
                let mut result = Vec::new();
                
                // Add default steps if present
                if let Some(default_steps) = steps_obj.get("default").and_then(|s| s.as_array()) {
                    for step in default_steps {
                        if let Some(step_str) = step.as_str() {
                            let mut step_text = step_str.to_string();
                            
                            // Apply variable substitution
                            if let Some(vars) = &variables {
                                let mut handlebars = Handlebars::new();
                                handlebars.register_escape_fn(handlebars::no_escape);
                                
                                if let Ok(rendered) = handlebars.render_template(&step_text, vars) {
                                    step_text = rendered;
                                }
                            }
                            
                            step_text = step_text.replace("{{project_name}}", project_name);
                            result.push(step_text);
                        }
                    }
                }
                
                // Process conditional steps if present
                if let Some(conditional_steps) = steps_obj.get("conditional").and_then(|s| s.as_array()) {
                    if let Some(vars) = &variables {
                        for condition_obj in conditional_steps {
                            if let Some(condition_map) = condition_obj.as_object() {
                                // Get the condition expression
                                if let Some(when_expr) = condition_map.get("when").and_then(|w| w.as_str()) {
                                    // Simple condition evaluation for now - just check for exact matches
                                    // Format: "variable == 'value'"
                                    if let Some((var_name, expected_value)) = parse_condition(when_expr) {
                                        // Check if the variable exists and matches the expected value
                                        if let Some(actual_value) = vars.get(var_name).and_then(|v| v.as_str()) {
                                            if actual_value == expected_value {
                                                // If condition matches, add the steps
                                                if let Some(steps) = condition_map.get("steps").and_then(|s| s.as_array()) {
                                                    for step in steps {
                                                        if let Some(step_str) = step.as_str() {
                                                            let mut step_text = step_str.to_string();
                                                            
                                                            // Apply variable substitution
                                                            let mut handlebars = Handlebars::new();
                                                            handlebars.register_escape_fn(handlebars::no_escape);
                                                            
                                                            if let Ok(rendered) = handlebars.render_template(&step_text, vars) {
                                                                step_text = rendered;
                                                            }
                                                            
                                                            step_text = step_text.replace("{{project_name}}", project_name);
                                                            result.push(step_text);
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
                
                // Add post-setup info if present
                if let Some(post_setup) = template_config.get("post_setup_info") {
                    if let Some(post_setup_obj) = post_setup.as_object() {
                        if let Some(conditional_messages) = post_setup_obj.get("conditional").and_then(|c| c.as_array()) {
                            if let Some(vars) = &variables {
                                for condition_obj in conditional_messages {
                                    if let Some(condition_map) = condition_obj.as_object() {
                                        if let Some(when_expr) = condition_map.get("when").and_then(|w| w.as_str()) {
                                            if let Some((var_name, expected_value)) = parse_condition(when_expr) {
                                                if let Some(actual_value) = vars.get(var_name).and_then(|v| v.as_str()) {
                                                    if actual_value == expected_value {
                                                        if let Some(message) = condition_map.get("message").and_then(|m| m.as_str()) {
                                                            result.push(message.to_string());
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
                
                if !result.is_empty() {
                    return Some(result);
                }
                
                // If we get here with empty result, also check for old format for backward compatibility
                if let Some(vars) = &variables {
                    // Try to match based on data_format variable - important for Parquet support
                    if let Some(data_format) = vars.get("data_format").and_then(|f| f.as_str()) {
                        if let Some(format_steps) = steps_obj.get(data_format).and_then(|s| s.as_array()) {
                            let mut data_format_result = Vec::new();
                            
                            for step in format_steps {
                                if let Some(step_str) = step.as_str() {
                                    // Replace {{project_name}} with the actual project name
                                    let step_text = step_str.replace("{{project_name}}", project_name);
                                    data_format_result.push(step_text);
                                }
                            }
                            
                            if !data_format_result.is_empty() {
                                return Some(data_format_result);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Default steps if no specific steps found
    Some(vec![
        format!("🚀 Navigate to your project: cd {}", project_name),
        "📝 Review the generated code".to_string(),
        "🔧 Build the project: cargo build".to_string(),
        "▶️ Run the project: cargo run".to_string(),
    ])
}

/// Parse a simple condition string like "variable == 'value'" into a tuple (variable_name, expected_value)
/// This function supports both single and double quotes around the value.
#[allow(dead_code)]
fn parse_condition(condition: &str) -> Option<(&str, &str)> {
    // Look for patterns like "variable == 'value'" or "variable == \"value\""
    let parts: Vec<&str> = condition.split("==").collect();
    if parts.len() != 2 {
        return None;
    }
    
    let variable = parts[0].trim();
    let value = parts[1].trim();
    
    // Remove quotes from value
    let value = if (value.starts_with('\'') && value.ends_with('\'')) || 
                  (value.starts_with('"') && value.ends_with('"')) {
        &value[1..value.len()-1]
    } else {
        value
    };
    
    Some((variable, value))
}

/// Use the shared module's copy_directory function
#[allow(dead_code)]
// Removed unused import

/// Evaluate a condition expression against variables
#[allow(dead_code)]
fn evaluate_condition(condition: &str, variables: &Value) -> bool {
    // Simple condition parsing
    let parts: Vec<&str> = condition.split_whitespace().collect();
    if parts.len() != 3 {
        return false;
    }
    
    let var_name = parts[0];
    let operator = parts[1];
    let expected_value = parts[2].trim_matches('"');
    
    if let Some(var_value) = variables.get(var_name).and_then(|v| v.as_str()) {
        match operator {
            "==" => var_value == expected_value,
            "!=" => var_value != expected_value,
            _ => false,
        }
    } else {
        false
    }
}

/// Get the template directory for a given template name
#[allow(dead_code)]
pub fn get_template_dir(template_name: &str) -> Result<PathBuf> {
    let templates_dir = get_templates_dir()?;
    
    // Handle template names with subdirectories (like client/leptos/counter)
    let template_dir = templates_dir.join(template_name);
    if template_dir.exists() && template_dir.is_dir() {
        return Ok(template_dir);
    }
    
    Err(anyhow::anyhow!("Template directory not found: {}", template_name))
}

/// Check if a template exists
#[allow(dead_code)]
pub fn template_exists(template_name: &str) -> bool {
    if let Ok(template_dir) = get_template_dir(template_name) {
        return template_dir.exists() && template_dir.is_dir();
    }
    false
}

/// Get the base templates directory
#[allow(dead_code)]
pub fn get_templates_dir() -> Result<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            // Fall back to current directory if CARGO_MANIFEST_DIR is not set
            std::env::current_dir()
        })?;
    
    Ok(manifest_dir.join("templates"))
}

/// Update Cargo.toml with template dependencies
#[allow(dead_code)]
fn update_cargo_toml(project_dir: &Path, dependencies: &[String]) -> Result<()> {
    let cargo_path = project_dir.join("Cargo.toml");
    if !cargo_path.exists() || dependencies.is_empty() {
        return Ok(());  // Nothing to update
    }
    
    // Instead of complex parsing, let's use a simpler approach:
    // 1. First write the dependencies to a temporary Cargo.toml file
    // 2. Then use cargo add to add each dependency to the main Cargo.toml
    
    // Create a temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_cargo_path = temp_dir.path().join("Cargo.toml");
    
    // Create a minimal Cargo.toml with the dependencies
    let mut temp_cargo_content = String::from("[package]\nname = \"temp\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n");
    for dep in dependencies {
        temp_cargo_content.push_str(dep);
        temp_cargo_content.push_str("\n");
    }
    
    // Write the temporary Cargo.toml
    fs::write(&temp_cargo_path, temp_cargo_content)?;
    
    // Parse the temporary Cargo.toml to extract dependencies in the format we need
    let doc = fs::read_to_string(&temp_cargo_path)?
        .parse::<toml_edit::DocumentMut>()
        .context("Failed to parse temporary Cargo.toml")?;
    
    if let Some(deps) = doc.get("dependencies") {
        // Use our extract_dependencies function to get the dependencies in the right format
        let deps_to_add = extract_dependencies(deps)?;
        
        // Use our enhanced utility function to add the dependencies to the actual Cargo.toml
        if !deps_to_add.is_empty() {
            update_cargo_with_dependencies(&cargo_path, deps_to_add, false)?
        }
    }
    
    Ok(())
}

/// Get dependencies for a template based on variables
#[allow(dead_code)]
fn get_template_dependencies(template_name: &str, variables: &Value) -> Option<Vec<String>> {
    if let Ok(template_config) = get_template_config(template_name) {
        if let Some(deps) = template_config.get("dependencies") {
            let mut result = Vec::new();
            
            // Add default dependencies
            if let Some(default_deps) = deps.get("default").and_then(|d| d.as_array()) {
                for dep in default_deps {
                    if let Some(dep_str) = dep.as_str() {
                        result.push(dep_str.to_string());
                    }
                }
            }
            
            // Add conditional dependencies
            if let Some(deps_obj) = deps.as_object() {
                if let Some(var_obj) = variables.as_object() {
                    for (_var_name, var_val) in var_obj {
                        if let Some(var_val_str) = var_val.as_str() {
                            if let Some(cond_deps) = deps_obj.get(var_val_str).and_then(|d| d.as_array()) {
                                for dep in cond_deps {
                                    if let Some(dep_str) = dep.as_str() {
                                        result.push(dep_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if !result.is_empty() {
                return Some(result);
            }
        }
    }
    
    None
}
