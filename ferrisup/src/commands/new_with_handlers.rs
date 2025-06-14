use anyhow::{Result, anyhow};
use colored::Colorize;
use dialoguer::{Select, Input};
use serde_json::{Value, json, Map};
use std::path::Path;
use std::process::Command;
use std::fs;
use crate::project;
use shared::to_pascal_case;

/// Main execute function to handle project creation
#[allow(dead_code)]
pub fn execute(
    name: Option<&str>,
    template: Option<&str>,
    git: bool,
    build: bool,
    no_interactive: bool,
    _project_type: Option<&str>,
) -> Result<()> {
    // Get project name
    let name = match name {
        Some(name) => name.to_string(),
        None => {
            if no_interactive {
                return Err(anyhow!("Project name is required in non-interactive mode"));
            }
            Input::<String>::new()
                .with_prompt("Project name")
                .interact()?
        }
    };

    // Create project directory
    let app_path = Path::new(&name);
    ensure_directory_exists(app_path)?;

    // Get template
    let template = match template {
        Some(template) => template.to_string(),
        None => {
            if no_interactive {
                return Err(anyhow!("Template is required in non-interactive mode"));
            }
            
            let templates_with_desc = project::list_templates()?;
            let mut templates: Vec<&str> = templates_with_desc.iter().map(|(name, _)| name.as_str()).collect();
            
            // Add edge template if not present
            if !templates.contains(&"edge") {
                templates.push("edge");
            }
            
            // Create a vector of template descriptions
            let descriptions: Vec<String> = templates.iter().map(|&name| {
                if name == "edge" {
                    return "Edge computing applications (Cloudflare, Vercel, Fastly, AWS, etc.)".to_string();
                }
                
                // Find the description for this template
                for (template_name, desc) in &templates_with_desc {
                    if template_name == name {
                        return desc.clone();
                    }
                }
                
                // Default description if not found
                "".to_string()
            }).collect();
            
            // Format the items for display
            let template_items: Vec<String> = templates.iter()
                .zip(descriptions.iter())
                .map(|(&name, desc)| format!("{} - {}", name, desc))
                .collect();
            
            let selection = Select::new()
                .with_prompt("Select a template")
                .items(&template_items)
                .default(0)
                .interact()?;
                
            templates[selection].to_string()
        }
    };

    // Collect variables for the template
    let variables = collect_template_variables(&template, &name, no_interactive)?;
    
    // Add the template name to the variables
    let mut vars_map = variables.as_object().cloned().unwrap_or_default();
    vars_map.insert("template".to_string(), json!(template));
    let variables = Value::Object(vars_map);
    
    // Find a handler for this template
    let handler = project::find_handler(&template, &variables)
        .ok_or_else(|| anyhow!("No handler found for template {}", template))?;
    
    println!("📦 Creating {} project with {} handler", name, handler.name());
    
    // Initialize the project using the selected handler
    handler.initialize_project(&name, app_path, &variables)?;
    
    // Initialize git repository if requested
    if git {
        println!("🔄 Initializing git repository...");
        let _ = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(app_path)
            .status();
            
        // Create a .gitignore file if it doesn't exist
        let gitignore_path = app_path.join(".gitignore");
        if !gitignore_path.exists() {
            let gitignore_content = "/target\n/Cargo.lock\n.DS_Store\n";
            fs::write(gitignore_path, gitignore_content)?;
        }
    }
    
    // Build the project if requested
    if build {
        println!("🔄 Building project...");
        let status = Command::new("cargo")
            .args(["build"])
            .current_dir(app_path)
            .status()?;
            
        if !status.success() {
            println!("⚠️ Project build failed. You may need to modify some files before building.");
        } else {
            println!("✅ Project built successfully!");
        }
    }
    
    // Print successful message
    println!("\n✅ {} project created successfully!", name.green());
    
    // Display next steps
    let next_steps = handler.get_next_steps(&name, &variables);
    if !next_steps.is_empty() {
        println!("\n{}", "Next steps:".bold().green());
        for step in next_steps {
            println!("- {}", step);
        }
    }
    
    println!("\n🎉 Project {} created successfully!", name);
    
    Ok(())
}

/// Process template variables and collect them into a JSON object
#[allow(dead_code)]
fn collect_template_variables(template: &str, name: &str, no_interactive: bool) -> Result<Value> {
    let variables = process_variables(template, name, no_interactive)?;
    Ok(variables)
}

/// Helper function to process variables
#[allow(dead_code)]
fn process_variables(template: &str, name: &str, no_interactive: bool) -> Result<Value> {
    // Get template configuration
    let template_config = project::get_template_config(template)?;
    
    // Start with default variables
    let mut variables = Map::new();
    variables.insert("project_name".to_string(), json!(name));
    variables.insert("project_name_pascal_case".to_string(), json!(to_pascal_case(name)));
    
    // Process prompts from template config
    if let Some(prompts) = template_config.get("prompts").and_then(|p| p.as_array()) {
        for prompt in prompts {
            if let Some(prompt_obj) = prompt.as_object() {
                let prompt_name = prompt_obj.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                let prompt_question = prompt_obj.get("question").and_then(|q| q.as_str()).unwrap_or("Question");
                
                if let Some(options) = prompt_obj.get("options").and_then(|o| o.as_array()) {
                    // Convert options to string vec
                    let options_vec: Vec<&str> = options.iter()
                        .filter_map(|o| o.as_str())
                        .collect();
                        
                    if options_vec.is_empty() {
                        continue;
                    }
                    
                    // Get default selection
                    let default_idx = prompt_obj.get("default")
                        .and_then(|d| d.as_str())
                        .and_then(|d| options_vec.iter().position(|&o| o == d))
                        .unwrap_or(0);
                    
                    let selection = if no_interactive {
                        default_idx
                    } else {
                        Select::new()
                            .with_prompt(prompt_question)
                            .items(&options_vec)
                            .default(default_idx)
                            .interact()?
                    };
                    
                    let selected_value = options_vec[selection];
                    println!("Using {} as the {}", selected_value, prompt_name);
                    variables.insert(prompt_name.to_string(), json!(selected_value));
                } else {
                    // Simple text input
                    let default_value = prompt_obj.get("default")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");
                        
                    let value = if no_interactive {
                        default_value.to_string()
                    } else {
                        Input::<String>::new()
                            .with_prompt(prompt_question)
                            .with_initial_text(default_value)
                            .interact()?
                    };
                    
                    println!("Using {} as the {}", value, prompt_name);
                    variables.insert(prompt_name.to_string(), json!(value));
                }
            }
        }
    }
    
    // Add custom variables defined in the template
    if let Some(template_vars) = template_config.get("variables").and_then(|v| v.as_object()) {
        for (key, value) in template_vars {
            if !variables.contains_key(key) {
                variables.insert(key.clone(), value.clone());
            }
        }
    }
    
    Ok(Value::Object(variables))
}

/// Helper function to ensure a directory exists
#[allow(dead_code)]
fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| anyhow!("Failed to create directory: {}", e))?;
    }
    Ok(())
}
