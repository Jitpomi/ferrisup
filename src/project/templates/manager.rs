// Template manager for applying templates and retrieving next steps
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;

/// Get all available templates
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
pub fn apply_template(
    template_name: &str,
    target_dir: &Path,
    project_name: &str,
    variables: Option<Value>,
) -> Result<()> {
    println!("Applying template {} to {}", template_name, target_dir.display());
    
    // Get template directory
    let template_dir = find_template_directory(template_name)?;
    
    // Get template configuration
    let template_config = get_template_config(template_name)?;
    
    // Create variables map
    let mut template_vars = json!({
        "project_name": project_name,
        "project_name_pascal_case": to_pascal_case(project_name),
    });
    
    // Add custom variables
    if let Some(vars) = variables {
        if let Some(obj) = vars.as_object() {
            if let Some(obj_mut) = template_vars.as_object_mut() {
                for (key, value) in obj {
                    obj_mut.insert(key.clone(), value.clone());
                }
            }
        }
    }
    
    // Create Handlebars instance
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    
    // Process template files
    if let Some(files) = template_config.get("files").and_then(|f| f.as_array()) {
        for file_entry in files {
            if let Some(file_obj) = file_entry.as_object() {
                let source = file_obj.get("source").and_then(|s| s.as_str());
                let target = file_obj.get("target").and_then(|t| t.as_str());
                let condition = file_obj.get("condition").and_then(|c| c.as_str());
                
                // Skip if condition not met
                if let Some(cond) = condition {
                    if !evaluate_condition(cond, &template_vars) {
                        continue;
                    }
                }
                
                if let (Some(src), Some(tgt)) = (source, target) {
                    let src_path = template_dir.join(src);
                    let tgt_path = target_dir.join(tgt);
                    
                    // Create parent directories
                    if let Some(parent) = tgt_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    
                    if src_path.is_dir() {
                        // Copy directory
                        copy_dir_all(&src_path, &tgt_path)?;
                    } else if src_path.exists() {
                        // Process template file
                        let content = fs::read_to_string(&src_path)?;
                        let processed = handlebars.render_template(&content, &template_vars)?;
                        fs::write(&tgt_path, processed)?;
                        
                        // Set executable permission for scripts
                        if src_path.extension().map_or(false, |ext| ext == "sh") {
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let mut perms = fs::metadata(&tgt_path)?.permissions();
                                perms.set_mode(0o755);
                                fs::set_permissions(&tgt_path, perms)?;
                            }
                        }
                    } else {
                        return Err(anyhow::anyhow!("Source file not found: {}", src_path.display()));
                    }
                }
            }
        }
    } else {
        // Just copy all files from the template directory
        copy_dir_all(&template_dir, target_dir)?;
    }
    
    // Run post-generation hooks
    if let Some(hooks) = template_config.get("hooks").and_then(|h| h.as_object()) {
        if let Some(post_gen) = hooks.get("post_gen").and_then(|p| p.as_str()) {
            let hook_path = target_dir.join(post_gen);
            if hook_path.exists() {
                println!("Running post-generation hook: {}", post_gen);
                
                // Create a temporary variables file
                let vars_file = target_dir.join(".variables.json");
                fs::write(&vars_file, template_vars.to_string())?;
                
                // Run the hook
                let status = std::process::Command::new(&hook_path)
                    .current_dir(target_dir)
                    .status()?;
                
                if !status.success() {
                    println!("⚠️ Post-generation hook failed with status: {}", status);
                }
                
                // Clean up
                let _ = fs::remove_file(vars_file);
            }
        }
    }
    
    println!("✅ Template applied successfully!");
    Ok(())
}

/// Get next steps for a template
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
                                    // Format: "variable_name == 'value'"
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

/// Parse a simple condition string in the format "variable_name == 'value'"
fn parse_condition(condition: &str) -> Option<(&str, &str)> {
    // Split the condition by the equality operator
    let parts: Vec<&str> = condition.split("==").collect();
    if parts.len() != 2 {
        return None;
    }
    
    // Trim whitespace and quotes from the variable name and value
    let var_name = parts[0].trim();
    let value = parts[1].trim().trim_matches('\'').trim_matches('"');
    
    Some((var_name, value))
}

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        } else {
            capitalize_next = true;
        }
    }
    
    result
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_dir_all(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    
    Ok(())
}

/// Evaluate a condition expression against variables
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
