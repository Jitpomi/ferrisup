use anyhow::Result;
use std::fs;

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
    let mut templates = Vec::new();
    
    // Add built-in templates with descriptions
    templates.push(("minimal".to_string(), "Simple binary with a single main.rs file".to_string()));
    templates.push(("library".to_string(), "Rust library crate with a lib.rs file".to_string()));
    templates.push(("full-stack".to_string(), "Complete application with client, server, and shared libraries".to_string()));
    templates.push(("gen-ai".to_string(), "AI-focused project with inference and model components".to_string()));
    templates.push(("edge-app".to_string(), "WebAssembly-based application for edge computing".to_string()));
    templates.push(("embedded".to_string(), "Embedded systems firmware for microcontrollers".to_string()));
    templates.push(("serverless".to_string(), "Serverless functions for cloud deployment".to_string()));
    templates.push(("iot-device".to_string(), "IoT device firmware with connectivity features".to_string()));
    templates.push(("ml-pipeline".to_string(), "Machine learning data processing pipeline".to_string()));
    templates.push(("data-science".to_string(), "Data science project with analysis tools".to_string()));
    
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
