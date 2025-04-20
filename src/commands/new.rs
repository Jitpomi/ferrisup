use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, anyhow};
use dialoguer::{Select, Input};
use crate::template_manager;
use crate::utils::create_directory;
use serde_json::{self, json, Value};
use std::fs;
use std::io;
use handlebars::Handlebars;

// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

// Note: For frameworks and libraries that have official CLIs (like Dioxus and Tauri),
// we use those CLIs directly instead of maintaining our own templates.
// This ensures we're always using the most up-to-date project creation methods
// and reduces maintenance burden.

// Main execute function to handle Leptos project creation
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
    create_directory(app_path)?;

    // Get template
    let mut template = match template {
        Some(template) => template.to_string(),
        None => {
            if no_interactive {
                return Err(anyhow!("Template is required in non-interactive mode"));
            }
            
            let templates_with_desc = template_manager::list_templates()?;
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

    // Declare additional_vars here
    let mut additional_vars = None;

    // Get template configuration to check for options
    let template_config = template_manager::get_template_config(&template)?;
    
    // Handle special templates
    if template == "server" {
        // Server template handling
        let framework_options = ["axum", "actix", "poem"];
        
        let selection = Select::new()
            .with_prompt("Which web framework would you like to use?")
            .items(&framework_options)
            .default(0)
            .interact()?;
            
        let framework_selected = framework_options[selection];
        println!("Using {} as the server_framework", framework_selected);
        
        // Handle server template creation manually instead of using the template system
        let framework_template_dir_path = PathBuf::from(format!("{}/templates/server/{}", env!("CARGO_MANIFEST_DIR"), framework_selected));
        
        if !framework_template_dir_path.exists() {
            return Err(anyhow::anyhow!("Could not find template directory for {} framework", framework_selected));
        }
        
        // Copy src directory
        let src_dir = framework_template_dir_path.join("src");
        if src_dir.exists() {
            copy_dir_all(&src_dir, &app_path.join("src"))?;
        } else {
            return Err(anyhow::anyhow!("Could not find src directory for {} framework", framework_selected));
        }
        
        // Copy Cargo.toml.template and process it
        let cargo_toml_template = framework_template_dir_path.join("Cargo.toml.template");
        if cargo_toml_template.exists() {
            let content = fs::read_to_string(&cargo_toml_template)?;
            
            // Create handlebars instance for templating
            let mut handlebars = Handlebars::new();
            handlebars.register_escape_fn(handlebars::no_escape);
            
            // Create template vars
            let template_vars = json!({
                "project_name": name,
                "project_name_pascal_case": to_pascal_case(&name)
            });
            
            // Apply templating
            let rendered = handlebars.render_template(&content, &template_vars)
                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                
            // Write to target file
            fs::write(app_path.join("Cargo.toml"), rendered)?;
        } else {
            return Err(anyhow::anyhow!("Could not find Cargo.toml.template for {} framework", framework_selected));
        }
        
        // Copy README.md from the framework-specific template
        let readme_src = framework_template_dir_path.join("README.md");
        
        if readme_src.exists() {
            let content = fs::read_to_string(&readme_src)?;
            
            // Create handlebars instance for templating
            let mut handlebars = Handlebars::new();
            handlebars.register_escape_fn(handlebars::no_escape);
            
            // Create template vars
            let template_vars = json!({
                "project_name": name,
                "project_name_pascal_case": to_pascal_case(&name)
            });
            
            // Apply templating
            let rendered = handlebars.render_template(&content, &template_vars)
                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                
            // Write to target file
            fs::write(app_path.join("README.md"), rendered)?;
        } else {
            // Fallback to the main server README if framework-specific one doesn't exist
            let main_readme_src = PathBuf::from(format!("{}/templates/server/README.md", env!("CARGO_MANIFEST_DIR")));
            if main_readme_src.exists() {
                let content = fs::read_to_string(&main_readme_src)?;
                
                // Create handlebars instance for templating
                let mut handlebars = Handlebars::new();
                handlebars.register_escape_fn(handlebars::no_escape);
                
                // Register helpers
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
                
                // Create template vars
                let template_vars = json!({
                    "project_name": name,
                    "project_name_pascal_case": to_pascal_case(&name),
                    "server_framework": framework_selected
                });
                
                // Apply templating
                let rendered = handlebars.render_template(&content, &template_vars)
                    .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                    
                // Write to target file
                fs::write(app_path.join("README.md"), rendered)?;
            }
        }
    } else if template == "serverless" {
        // Serverless template handling
        let provider_options = ["aws", "gcp", "azure", "vercel", "netlify"];
        
        let selection = Select::new()
            .with_prompt("Which cloud provider would you like to target for your serverless function?")
            .items(&provider_options)
            .default(0)
            .interact()?;
            
        let provider_selected = provider_options[selection];
        
        // Display appropriate help message based on selected provider
        let help_messages = json!({
            "aws": "AWS Lambda is a serverless compute service that runs your code in response to events and automatically manages the underlying compute resources.",
            "gcp": "Google Cloud Functions is a serverless execution environment for building and connecting cloud services.",
            "azure": "Azure Functions is a serverless solution that allows you to write less code, maintain less infrastructure, and save on costs.",
            "vercel": "Vercel Functions provide a serverless platform for deploying functions that run on-demand and scale automatically.",
            "netlify": "Netlify Functions let you deploy server-side code that runs in response to events, without having to run a dedicated server."
        });
        
        if let Some(help_message) = help_messages.get(provider_selected).and_then(|v| v.as_str()) {
            println!("ℹ️  {}", help_message);
        }
        
        println!("Using {} as the cloud_provider", provider_selected);
        
        // Handle serverless template creation manually
        let provider_template_dir_path = PathBuf::from(format!("{}/templates/serverless/{}", env!("CARGO_MANIFEST_DIR"), provider_selected));
        
        if !provider_template_dir_path.exists() {
            return Err(anyhow::anyhow!("Could not find template directory for {} provider", provider_selected));
        }
        
        // Copy src directory
        let src_dir = provider_template_dir_path.join("src");
        if src_dir.exists() {
            copy_dir_all(&src_dir, &app_path.join("src"))?;
        } else {
            return Err(anyhow::anyhow!("Could not find src directory for {} provider", provider_selected));
        }
        
        // Copy Cargo.toml.template and process it
        let cargo_toml_template = provider_template_dir_path.join("Cargo.toml.template");
        if cargo_toml_template.exists() {
            let content = fs::read_to_string(&cargo_toml_template)?;
            
            // Create handlebars instance for templating
            let mut handlebars = Handlebars::new();
            handlebars.register_escape_fn(handlebars::no_escape);
            
            // Create template vars
            let template_vars = json!({
                "project_name": name,
                "project_name_pascal_case": to_pascal_case(&name)
            });
            
            // Apply templating
            let rendered = handlebars.render_template(&content, &template_vars)
                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                
            // Write to target file
            fs::write(app_path.join("Cargo.toml"), rendered)?;
        } else {
            return Err(anyhow::anyhow!("Could not find Cargo.toml.template for {} provider", provider_selected));
        }
        
        // Copy README.md from the provider-specific template
        let readme_src = provider_template_dir_path.join("README.md.template");
        
        if readme_src.exists() {
            let content = fs::read_to_string(&readme_src)?;
            
            // Create handlebars instance for templating
            let mut handlebars = Handlebars::new();
            handlebars.register_escape_fn(handlebars::no_escape);
            
            // Create template vars
            let template_vars = json!({
                "project_name": name,
                "project_name_pascal_case": to_pascal_case(&name)
            });
            
            // Apply templating
            let rendered = handlebars.render_template(&content, &template_vars)
                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                
            // Write to target file
            fs::write(app_path.join("README.md"), rendered)?;
        }
        
        // Copy provider-specific configuration files
        let template_json_path = provider_template_dir_path.join("template.json");
        let template_config = if template_json_path.exists() {
            let content = fs::read_to_string(&template_json_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
        } else {
            json!({})
        };
        
        if let Some(files) = template_config.get("files").and_then(|f| f.as_array()) {
            for file_entry in files {
                if let (Some(source), Some(target)) = (
                    file_entry.get("source").and_then(|s| s.as_str()),
                    file_entry.get("target").and_then(|t| t.as_str())
                ) {
                    let source_path = provider_template_dir_path.join(source);
                    if source_path.exists() {
                        if source_path.is_dir() {
                            copy_dir_all(&source_path, &app_path.join(target))?;
                        } else {
                            // Read the source file
                            let content = fs::read_to_string(&source_path)?;
                            
                            // Create handlebars instance for templating
                            let mut handlebars = Handlebars::new();
                            handlebars.register_escape_fn(handlebars::no_escape);
                            
                            // Create template vars
                            let template_vars = json!({
                                "project_name": name,
                                "project_name_pascal_case": to_pascal_case(&name)
                            });
                            
                            // Apply templating
                            let rendered = handlebars.render_template(&content, &template_vars)
                                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                                
                            // Write to target file
                            let target_path = app_path.join(target);
                            if let Some(parent) = target_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            fs::write(target_path, rendered)?;
                        }
                    }
                }
            }
        }
        
        // Copy any provider-specific files directly from the provider directory
        for entry in fs::read_dir(&provider_template_dir_path)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Skip src, template.json, Cargo.toml.template, and README.md.template
            if file_name_str == "src" || file_name_str == "template.json" || 
               file_name_str == "Cargo.toml.template" || file_name_str == "README.md.template" {
                continue;
            }
            
            let source_path = entry.path();
            let target_path = app_path.join(&file_name);
            
            if source_path.is_dir() {
                copy_dir_all(&source_path, &target_path)?;
            } else {
                // Read the source file
                let content = fs::read_to_string(&source_path)?;
                
                // Create handlebars instance for templating
                let mut handlebars = Handlebars::new();
                handlebars.register_escape_fn(handlebars::no_escape);
                
                // Create template vars
                let template_vars = json!({
                    "project_name": name,
                    "project_name_pascal_case": to_pascal_case(&name)
                });
                
                // Apply templating
                let rendered = handlebars.render_template(&content, &template_vars)
                    .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                    
                // Write to target file
                fs::write(target_path, rendered)?;
            }
        }
        
        // Show provider-specific next steps
        let template_json = fs::read_to_string(PathBuf::from(format!("{}/templates/serverless/template.json", env!("CARGO_MANIFEST_DIR"))))?;
        let template_config: Value = serde_json::from_str(&template_json)?;
        let next_steps_key = format!("next_steps_{}", provider_selected);
        
        println!("\nNext steps:");
        if let Some(next_steps) = template_config.get(&next_steps_key).and_then(|s| s.as_array()) {
            for step in next_steps {
                if let Some(step_str) = step.as_str() {
                    let processed_step = step_str.replace("{{project_name}}", &name);
                    println!("  {}", processed_step);
                }
            }
        }
    } else if template == "edge" {
        // Handle edge template specifically to support the hierarchical structure
        // Get the edge template configuration
        let edge_template_path = format!("{}/templates/edge", env!("CARGO_MANIFEST_DIR"));
        let edge_template_json_path = Path::new(&edge_template_path).join("template.json");
        
        if edge_template_json_path.exists() {
            // Read the edge template configuration
            let edge_config = fs::read_to_string(&edge_template_json_path)
                .map_err(|_| anyhow!("Failed to read edge template configuration"))?;
                
            let edge_template_json: Value = serde_json::from_str(&edge_config)
                .map_err(|_| anyhow!("Failed to parse edge template configuration"))?;
            
            // First level: Get application type options
            if let Some(options) = edge_template_json.get("options") {
                // Get the edge_type options
                if let Some(edge_type) = options.get("edge_type") {
                    if let Some(values) = edge_type.get("values").and_then(|v| v.as_array()) {
                        let app_type_options: Vec<&str> = values.iter()
                            .filter_map(|v| v.as_str())
                            .collect();
                            
                        if app_type_options.is_empty() {
                            return Err(anyhow!("No edge application types found"));
                        }
                        
                        // Get the descriptions/help text if available
                        let empty_map = serde_json::Map::new();
                        let app_type_help = edge_type.get("help")
                            .and_then(|h| h.as_object())
                            .unwrap_or(&empty_map);
                            
                        // Create formatted options with descriptions
                        let app_type_display: Vec<String> = app_type_options.iter()
                            .map(|&opt| {
                                if let Some(help) = app_type_help.get(opt).and_then(|h| h.as_str()) {
                                    format!("{} - {}", opt, help)
                                } else {
                                    opt.to_string()
                                }
                            })
                            .collect();
                        
                        println!("Edge computing applications allow you to run Rust code close to your users.");
                            
                        // Let user select application type
                        let app_type_selection = Select::new()
                            .with_prompt("Select edge application type")
                            .items(&app_type_display)
                            .default(0)
                            .interact()?;
                            
                        let selected_app_type = app_type_options[app_type_selection];
                        println!("Selected application type: {}", selected_app_type);
                        
                        // Second level: Get provider options for the selected type
                        let provider_field = match selected_app_type {
                            "static-site" => "static_site_provider",
                            "api-function" => "api_function_provider",
                            "web-component" => "web_component_type",
                            _ => return Err(anyhow!("Unsupported application type")),
                        };
                        
                        if let Some(provider_option) = options.get(provider_field) {
                            if let Some(provider_values) = provider_option.get("values").and_then(|v| v.as_array()) {
                                let provider_options: Vec<&str> = provider_values.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect();
                                    
                                if provider_options.is_empty() {
                                    return Err(anyhow!("No providers found for the selected application type"));
                                }
                                
                                // Get the descriptions/help text if available
                                let empty_map = serde_json::Map::new();
                                let provider_help = provider_option.get("help")
                                    .and_then(|h| h.as_object())
                                    .unwrap_or(&empty_map);
                                    
                                // Create formatted options with descriptions
                                let provider_display: Vec<String> = provider_options.iter()
                                    .map(|&opt| {
                                        if let Some(help) = provider_help.get(opt).and_then(|h| h.as_str()) {
                                            format!("{} - {}", opt, help)
                                        } else {
                                            opt.to_string()
                                        }
                                    })
                                    .collect();
                                
                                // Let user select provider
                                let provider_selection = Select::new()
                                    .with_prompt("Select provider")
                                    .items(&provider_display)
                                    .default(0)
                                    .interact()?;
                                    
                                let selected_provider = provider_options[provider_selection];
                                println!("Selected provider: {}", selected_provider);
                                
                                // Create variables for template
                                let mut vars_map = serde_json::Map::new();
                                vars_map.insert("edge_type".to_string(), json!(selected_app_type));
                                vars_map.insert(provider_field.to_string(), json!(selected_provider));
                                
                                // Check if there's a redirect in the template config
                                if let Some(redirect) = edge_template_json.get("redirect") {
                                    if let Some(app_redirect) = redirect.get(selected_app_type) {
                                        if let Some(_provider_redirect) = app_redirect.get(selected_provider) {
                                            // We found a redirect configuration, use it to determine the template
                                            template = format!("edge/{}/{}", selected_app_type, selected_provider);
                                            additional_vars = Some(json!(vars_map));
                                            
                                            // Debug log the actual path we're trying to use
                                            let full_template_path = format!("{}/templates/{}", env!("CARGO_MANIFEST_DIR"), template);
                                            println!("Using template: {}", template);
                                            println!("Full template path: {}", full_template_path);
                                            
                                            // Check if the directory exists
                                            if !Path::new(&full_template_path).exists() {
                                                println!("⚠️ Warning: Template directory does not exist at {}", full_template_path);
                                                return Err(anyhow!("Template directory not found"));
                                            } else {
                                                println!("✅ Template directory exists");
                                                // List files in the directory for debugging
                                                if let Ok(entries) = fs::read_dir(&full_template_path) {
                                                    println!("Files in template directory:");
                                                    for entry in entries {
                                                        if let Ok(entry) = entry {
                                                            println!("  - {}", entry.file_name().to_string_lossy());
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            // Handle the edge template explicitly
                                            handle_edge_template(&template, app_path, &name, additional_vars.clone())?;
                                            return Ok(());
                                        } else {
                                            return Err(anyhow!("No template configuration found for provider: {}", selected_provider));
                                        }
                                    } else {
                                        return Err(anyhow!("No template configuration found for application type: {}", selected_app_type));
                                    }
                                } else {
                                    return Err(anyhow!("No redirect configuration found in the template"));
                                }
                            } else {
                                return Err(anyhow!("No provider values found"));
                            }
                        } else {
                            return Err(anyhow!("No provider options found for the selected application type"));
                        }
                    } else {
                        return Err(anyhow!("No edge_type values found"));
                    }
                } else {
                    return Err(anyhow!("No edge_type configuration found"));
                }
            } else {
                return Err(anyhow!("No options found in edge template configuration"));
            }
        } else {
            return Err(anyhow!("Edge template configuration not found"));
        }
    } else {
        // Skip options handling for templates we're handling manually
        if template != "server" && template != "serverless" {
            // Check if the template has options that need user input
            if let Some(options) = template_config.get("options").and_then(|o| o.as_array()) {
                let mut vars = serde_json::Map::new();
                
                for option in options {
                    if let (Some(name), Some(desc), Some(option_type)) = (
                        option.get("name").and_then(|n| n.as_str()),
                        option.get("description").and_then(|d| d.as_str()),
                        option.get("type").and_then(|t| t.as_str())
                    ) {
                        if option_type == "select" && option.get("options").is_some() {
                            let option_values: Vec<&str> = option.get("options")
                                .and_then(|o| o.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                                .unwrap_or_default();
                            
                            if !option_values.is_empty() {
                                let default_idx = option.get("default")
                                    .and_then(|d| d.as_str())
                                    .and_then(|d| option_values.iter().position(|&v| v == d))
                                    .unwrap_or(0);
                                
                                let selection = Select::new()
                                    .with_prompt(desc)
                                    .items(&option_values)
                                    .default(default_idx)
                                    .interact()?;
                                
                                let selected_value = option_values[selection];
                                vars.insert(name.to_string(), json!(selected_value));
                                
                                // If there's a help message for this option, display it
                                if let Some(help_messages) = template_config.get("help") {
                                    if let Some(help_message) = help_messages.get(selected_value).and_then(|m| m.as_str()) {
                                        println!("ℹ️  {}", help_message);
                                    }
                                }
                                
                                // If we're selecting a static server, and it's not "none", display guidance
                                if name == "static_server" && selected_value != "none" {
                                    println!("📝 Using {} as static file server. Run `{} . --port 8080` to start.", 
                                        selected_value, selected_value);
                                }
                                
                                // Echo selection
                                println!("Using {} as the {}", selected_value, name);
                            }
                        } else if option_type == "input" {
                            let default = option.get("default").and_then(|d| d.as_str()).unwrap_or("");
                            
                            let input = Input::<String>::new()
                                .with_prompt(desc)
                                .default(default.to_string())
                                .interact()?;
                            
                            vars.insert(name.to_string(), json!(input));
                            
                            // Echo input
                            println!("Using {} as the {}", input, name);
                        }
                    }
                }
                
                // Set additional_vars if we have any
                if !vars.is_empty() {
                    additional_vars = Some(json!(vars));
                }
            }
        }
    }

    // Handle special cases for client frameworks
    let mut _framework = String::new();

    // For client template, prompt for framework selection
    if template == "client" {
        println!("Template description: Custom template: client");
        println!("Using template: client");
        
        // Get client framework
        let frameworks = vec!["dioxus", "tauri", "leptos"];
        let selection = Select::new()
            .with_prompt("Select Rust client framework")
            .items(&frameworks)
            .default(0)
            .interact()?;
            
        let framework_selected = frameworks[selection];
        
        // For Leptos, prompt for specific template type
        if framework_selected == "leptos" {
            println!("📦 Using Leptos templates to bootstrap the project");
            println!("🔧 Checking for required dependencies...");
            
            // Check for wasm32-unknown-unknown target
            println!("🔍 Checking for wasm32-unknown-unknown target...");
            let wasm_check = Command::new("rustup")
                .args(["target", "list", "--installed"])
                .output()?;
            
            let wasm_output = String::from_utf8_lossy(&wasm_check.stdout);
            if !wasm_output.contains("wasm32-unknown-unknown") {
                println!("⚠️ wasm32-unknown-unknown target not found. Installing...");
                let status = Command::new("rustup")
                    .args(["target", "add", "wasm32-unknown-unknown"])
                    .status()?;
                
                if !status.success() {
                    println!("❌ Failed to install wasm32-unknown-unknown target.");
                    println!("Please install it manually with: rustup target add wasm32-unknown-unknown");
                } else {
                    println!("✅ wasm32-unknown-unknown target installed successfully");
                }
            } else {
                println!("✅ wasm32-unknown-unknown target is already installed");
            }
            
            // Check for trunk (needed for counter, router, todo templates)
            println!("🔍 Checking for Trunk...");
            let trunk_check = Command::new("trunk")
                .arg("--version")
                .output();
            
            match trunk_check {
                Ok(_) => println!("✅ Trunk is already installed"),
                Err(_) => {
                    println!("⚠️ Trunk not found. Installing...");
                    let status = Command::new("cargo")
                        .args(["install", "trunk", "--locked"])
                        .status()?;
                    
                    if !status.success() {
                        println!("❌ Failed to install Trunk.");
                        println!("Please install it manually with: cargo install trunk --locked");
                    } else {
                        println!("✅ Trunk installed successfully");
                    }
                }
            }
            
            let leptos_templates = vec![
                "Counter - Simple counter with reactive state",
                "Router - Multi-page application with routing",
                "Todo - Todo application with filtering",
            ];
            
            let leptos_selection = Select::new()
                .with_prompt("✨ Which Leptos template would you like to use?")
                .items(&leptos_templates)
                .default(0)
                .interact()?;
                
            // Map selection to template name
            template = match leptos_selection {
                0 => "counter".to_string(),
                1 => "router".to_string(),
                2 => "todo".to_string(),
                _ => "counter".to_string(), // Default to counter if somehow none selected
            };
            
            println!("🔧 Creating new Leptos project with {} template...", template);
            
            // Use template_manager to apply the template instead of hardcoded functions
            let template_path = format!("client/leptos/{}", template);
            
            // Apply the template using the template manager
            template_manager::apply_template(&template_path, app_path, &name, additional_vars.clone())?;
            
            // DO NOT print next steps here; let the template manager handle it
            return Ok(());
            
        } else if framework_selected == "dioxus" {
            println!("📦 Creating Dioxus project with dioxus-cli");
            
            // Check if dioxus-cli is installed
            println!("🔍 Checking for dioxus-cli...");
            let dx_check = Command::new("dx")
                .arg("--version")
                .output();
                
            let dx_installed = match dx_check {
                Ok(output) => output.status.success(),
                Err(_) => false
            };
            
            if !dx_installed {
                println!("⚠️ dioxus-cli not found. Installing...");
                let install_status = Command::new("cargo")
                    .args(["install", "dioxus-cli"])
                    .status()?;
                
                if !install_status.success() {
                    return Err(anyhow!("Failed to install dioxus-cli"));
                }
                println!("✅ dioxus-cli installed successfully");
            } else {
                println!("✅ dioxus-cli is already installed");
            }
            
            // Create project directory
            create_directory(app_path)?;
            
            // Let dioxus-cli handle the project creation with its own prompts
            println!("🔧 Creating Dioxus project...");
            let create_status = Command::new("dx")
                .args(["init"])
                .arg(".")
                .current_dir(app_path)
                .status()?;
                
            if !create_status.success() {
                return Err(anyhow!("Failed to create Dioxus project with dioxus-cli"));
            }
            
            // Ensure WASM target is installed for web projects
            println!("🔧 Ensuring WASM target is installed...");
            let _ = Command::new("rustup")
                .args(["target", "add", "wasm32-unknown-unknown"])
                .status();
            
            // Print success message with instructions
            println!("\n🎉 Project {} created successfully!", name);
            println!("\nNext steps:");
            println!("  cd {}", name);
            // Detect if this is a Dioxus workspace (web, desktop, mobile all exist)
            let web_exists = std::path::Path::new(&format!("{}/web", name)).exists();
            let desktop_exists = std::path::Path::new(&format!("{}/desktop", name)).exists();
            let mobile_exists = std::path::Path::new(&format!("{}/mobile", name)).exists();
            if web_exists || desktop_exists || mobile_exists {
                println!("  dx serve --package web    # For web application");
                println!("  dx serve --package desktop    # For desktop application");
                println!("  dx serve --package mobile    # For mobile application");
            } else {
                println!("  dx serve");
            }
            
            return Ok(());
            
        } else if framework_selected == "tauri" {
            println!("📦 Creating Tauri project with create-tauri-app");
            
            // Get the parent directory
            let parent_dir = app_path.parent().unwrap_or(Path::new("."));
            
            // Make sure we're not creating the project in the current directory
            // If app_path is just a filename without a directory, use the current directory
            let working_dir = if parent_dir == Path::new("") {
                Path::new(".")
            } else {
                parent_dir
            };
            
            // Create a temporary script to handle the create-tauri-app command
            // This will allow us to run the command interactively while still using the project name
            let script_content = format!(r#"#!/bin/sh
cd "{}"
npx create-tauri-app {}
"#, working_dir.display(), name);
            
            let temp_dir = std::env::temp_dir();
            let script_path = temp_dir.join("ferrisup_tauri_script.sh");
            
            // Write the script to a temporary file
            fs::write(&script_path, script_content)?;
            
            // Make the script executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&script_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms)?;
            }
            
            // Run the script
            let create_status = Command::new(&script_path)
                .status()?;
                
            // Clean up the temporary script
            let _ = fs::remove_file(&script_path);
                
            if !create_status.success() {
                return Err(anyhow!("Failed to create Tauri project with create-tauri-app"));
            }
            
            // Print success message
            println!("\n🎉 Project {} created successfully!", name);
            
            return Ok(());
            
        } else {
            // If not Leptos, Dioxus, or Tauri, use the selected framework as the template
            template = framework_selected.to_string();
        }
    } else if template == "data-science" {
        // For data science templates, first prompt for the ML framework
        // Using a simpler approach to avoid UI rendering issues
        let framework_options = vec![
            "Data Analysis with Polars",
            "Machine Learning with Linfa",
            "Deep Learning with Burn"
        ];
        
        // Create a selection without additional prompt text to avoid duplication
        let selection = Select::new()
            .with_prompt("📊 Select a Data Science approach")
            .default(0)
            .items(&framework_options)
            .interact()?;
            
        // Based on framework selection, show appropriate templates
        match selection {
            0 => {
                // Polars selected
                template = "data-science/polars-cli".to_string();
                println!("📈 Selected: Data Analysis with Polars");
            },
            1 => {
                // Linfa selected
                template = "data-science/linfa-examples".to_string();
                println!("🔍 Selected: Machine Learning with Linfa");
            },
            2 => {
                // Burn selected - show task categories
                // Using a simpler approach for category selection
                let burn_categories = vec![
                    "Image Processing",
                    "Text Processing",
                    "Numerical Data",
                    "Advanced & Experimental"
                ];
                
                let category_selection = Select::new()
                    .with_prompt("🧠 Select a Deep Learning task category")
                    .default(0)
                    .items(&burn_categories)
                    .interact()?;
                
                // Define the tasks and their corresponding template paths for each category
                let (category_prompt, tasks, template_paths) = match category_selection {
                    0 => {
                        // First present the image processing task categories
                        let image_categories = vec![
                            "Image Classification",
                            // Future categories (commented out until implemented)
                            // "Image Generation (Coming Soon)",
                            // "Image Segmentation (Coming Soon)",
                            // "Image Detection (Coming Soon)"
                        ];
                        
                        println!("🖼️ Select an Image Processing task:");
                        let image_category = Select::new()
                            .items(&image_categories)
                            .default(0)
                            .interact()?;
                        
                        // Based on the selected category, show appropriate options
                        match image_category {
                            0 => {
                                // Image Classification options
                                (
                                    "📊 Select an Image Classification model",
                                    vec![
                                        "MNIST Digit Recognition (Simple)",
                                        "General Image Classifier (CIFAR-10/Custom)"
                                    ],
                                    vec![
                                        "data-science/burn-image-recognition",
                                        "data-science/burn-image-classifier"
                                    ]
                                )
                            },
                            // Add other image categories in the future
                            _ => (
                                "📊 Select an Image Classification model",
                                vec!["MNIST Digit Recognition (Simple)"],
                                vec!["data-science/burn-image-recognition"]
                            )
                        }
                    },
                    1 => (
                        "📝 Select a Text Processing task",
                        vec![
                            "Text Classifier",
                            "Text Analyzer (Sentiment Analysis)"
                        ],
                        vec![
                            "data-science/burn-text-classifier",
                            "data-science/burn-text-analyzer"
                        ]
                    ),
                    2 => (
                        "📊 Select a Numerical Data task",
                        vec![
                            "Value Prediction",
                            "Data Predictor (Advanced Regression)"
                        ],
                        vec![
                            "data-science/burn-value-prediction",
                            "data-science/burn-data-predictor"
                        ]
                    ),
                    3 => (
                        "⚙️ Select an Advanced or Experimental task",
                        vec![
                            "Neural Network Playground"
                        ],
                        vec![
                            "data-science/burn-net"
                        ]
                    ),
                    _ => (
                        "🖼️ Select an Image Classification model",
                        vec!["MNIST Digit Recognition (Simple)"],
                        vec!["data-science/burn-image-recognition"]
                    ),
                };
                
                // Show task selection and get user choice
                let task_selection = Select::new()
                    .with_prompt(category_prompt)
                    .default(0)
                    .items(&tasks)
                    .interact()?;
                
                // Set the template path based on the task selection
                template = if task_selection < template_paths.len() {
                    template_paths[task_selection].to_string()
                } else {
                    // Fallback to the first template if selection is out of bounds
                    template_paths[0].to_string()
                };
                
                println!("🧠 Selected: {}", tasks[task_selection]);
            },
            _ => {
                // Fallback
                template = "data-science/polars-cli".to_string();
                println!("📈 Selected: Data Analysis with Polars (default)");
            }
        }
        
        println!("🔍 Checking for wasm32-unknown-unknown target...");
        check_dependencies(&template)?;
        
        println!("\n📊 Setting up {} data science project...", template.replace("data-science/", ""));
    }

    // Check for required dependencies based on template
    check_dependencies(&template)?;

    // Handle special cases for 
    // 
    // 
    // 
    // 
    // 
    // 
    
    
    // embedded templates
    if template == "embedded" {
        println!("📦 Creating embedded project for microcontrollers");
        
        // Check if the user selected Embassy framework from the options
        let use_embassy = if let Some(vars) = &additional_vars {
            if let Some(framework) = vars.get("framework").and_then(|f| f.as_str()) {
                framework == "Yes, use Embassy framework"
            } else {
                false
            }
        } else {
            false
        };
        
        if use_embassy {
            // User selected Embassy framework
            println!("📦 Creating Embassy project using cargo-embassy");
            
            // Check if cargo-embassy is installed
            println!("🔍 Checking for cargo-embassy...");
            let embassy_check = Command::new("cargo")
                .args(["embassy", "--version"])
                .output();
                
            let embassy_installed = match embassy_check {
                Ok(output) => output.status.success(),
                Err(_) => false
            };
            
            if !embassy_installed {
                println!("⚠️ cargo-embassy not found. Installing...");
                let status = Command::new("cargo")
                    .args(["install", "cargo-embassy"])
                    .status()?;
                
                if !status.success() {
                    println!("❌ Failed to install cargo-embassy.");
                    println!("Please install it manually with: cargo install cargo-embassy");
                    return Err(anyhow!("Failed to install cargo-embassy"));
                } else {
                    println!("✅ cargo-embassy installed successfully");
                }
            } else {
                println!("✅ cargo-embassy is already installed");
            }
            
            // Get microcontroller chip for Embassy
            let mcu_targets = vec!["rp2040", "stm32f4", "nrf52840", "esp32c3"];
            let selection = Select::new()
                .with_prompt("Select microcontroller chip")
                .items(&mcu_targets)
                .default(0)
                .interact()?;
                
            let mcu_chip = mcu_targets[selection];
            println!("Using {} as the microcontroller chip", mcu_chip);
            
            // Create the project using cargo-embassy
            println!("🔄 Creating new Embassy project...");
            
            // Create a parent directory for the Embassy project
            let parent_dir = Path::new(".").join("embassy_temp");
            fs::create_dir_all(&parent_dir)?;
            
            // Run cargo-embassy init from the parent directory
            let status = Command::new("cargo")
                .args(["embassy", "init", "--chip", mcu_chip, &name])
                .current_dir(&parent_dir)
                .status()?;
                
            if !status.success() {
                println!("❌ Failed to create Embassy project.");
                return Err(anyhow!("Failed to create Embassy project"));
            }
            
            // Move the generated project to the target directory
            let project_dir = parent_dir.join(&name);
            if project_dir.exists() {
                // Copy all files from the generated project to the target directory
                let target_dir = Path::new(&name);
                if !target_dir.exists() {
                    fs::create_dir_all(target_dir)?;
                }
                
                copy_dir_all(&project_dir, target_dir)?;
                
                // Clean up the temporary directory
                fs::remove_dir_all(parent_dir)?;
                
                println!("🎉 Embassy project {} created successfully!", name);
                println!("\nNext steps:");
                println!("  cd {}", name);
                println!("  # Build the project");
                println!("  cargo build --release");
                println!("  # Run the project");
                println!("  cargo run --release");
                
                return Ok(());
            } else {
                println!("❌ Failed to create Embassy project.");
                return Err(anyhow!("Failed to create Embassy project: Project directory not found"));
            }
        } else {
            // Standard embedded template
            // Get microcontroller target from options
            let mcu_target = if let Some(vars) = &additional_vars {
                vars.get("mcu_target")
                    .and_then(|t| t.as_str())
                    .unwrap_or("rp2040")
                    .to_string() // Clone the string to avoid borrowing issues
            } else {
                "rp2040".to_string()
            };
            
            println!("Using {} as the microcontroller target", mcu_target);
            
            // Create target-specific dependencies string
            let mcu_target_deps = match mcu_target.as_str() {
                "rp2040" => "rp2040-hal = \"0.9\"\nrp2040-boot2 = \"0.3\"\nusb-device = \"0.2\"\nusbd-serial = \"0.1\"",
                "stm32" => "stm32f4xx-hal = { version = \"0.17\", features = [\"stm32f411\"] }",
                "esp32" => "esp32-hal = \"0.16\"\nesp-backtrace = \"0.9\"\nesp-println = \"0.6\"",
                "arduino" => "arduino-hal = \"0.1\"\navr-device = \"0.5\"\nufmt = \"0.2\"",
                _ => "",
            };
            
            // Create variables for template substitution
            let mut vars = serde_json::Map::new();
            vars.insert("mcu_target".to_string(), json!(mcu_target));
            vars.insert("mcu_target_deps".to_string(), json!(mcu_target_deps));
            
            // Merge with existing additional_vars if any
            if let Some(ref existing_vars) = additional_vars {
                if let Some(existing_obj) = existing_vars.as_object() {
                    for (k, v) in existing_obj {
                        if k != "mcu_target" && k != "mcu_target_deps" {
                            vars.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            
            // Apply the template using the template manager
            template_manager::apply_template(&template, app_path, &name, Some(serde_json::Value::Object(vars)))?;
            
            // Suggest installing the appropriate Rust target
            let rust_target = match mcu_target.as_str() {
                "rp2040" => "thumbv6m-none-eabi",
                "stm32" => "thumbv7em-none-eabihf",
                "esp32" => "xtensa-esp32-none-elf",
                "arduino" => "avr-unknown-gnu-atmega328",
                _ => "thumbv6m-none-eabi",
            };
            
            println!("\nℹ️ You'll need to install the appropriate Rust target:");
            println!("  rustup target add {}", rust_target);
            
            match mcu_target.as_str() {
                "rp2040" => {
                    println!("  cargo install probe-run");
                    println!("  cargo run --target {}", rust_target);
                },
                "esp32" => {
                    println!("  cargo install espflash");
                    println!("  cargo build --target {}", rust_target);
                    println!("  espflash flash --monitor target/{}/debug/{}", rust_target, name);
                },
                "arduino" => {
                    println!("  cargo install ravedude");
                    println!("  cargo run --target {}", rust_target);
                },
                _ => {
                    println!("  cargo build --target {}", rust_target);
                }
            }
        }
    } else if template == "counter" || template == "router" || template == "todo" {
        // For Leptos templates, prepend "client/leptos/"
        let template_path = format!("client/leptos/{}", template);
        template_manager::apply_template(&template_path, app_path, &name, additional_vars.clone())?;
    } else {
        // For other templates, use as is
        template_manager::apply_template(&template, app_path, &name, additional_vars.clone())?;
    }

    // Initialize git repository if requested
    if git {
        println!("🔄 Initializing git repository...");
        let status = Command::new("git")
            .args(["init"])
            .current_dir(app_path)
            .status()?;
        if !status.success() {
            return Err(anyhow!("Failed to initialize git repository"));
        }
        
        // Create .gitignore file
        let gitignore = r#"/target
/dist
/Cargo.lock
**/*.rs.bk
"#;
        std::fs::write(app_path.join(".gitignore"), gitignore)?;
        println!("✅ Git repository initialized");
    }

    // Build project if requested
    if build {
        println!("🔄 Building project...");
        let status = Command::new("cargo")
            .args(["build"])
            .current_dir(app_path)
            .status()?;
        if !status.success() {
            return Err(anyhow!("Failed to build project"));
        }
        println!("✅ Project built successfully");
    }

    // Print success message with instructions
    println!("\n🎉 Project {} created successfully!", name);
    
    // We don't need to print next steps here as they're already printed in apply_template
    // The next steps include the static server command if applicable

    Ok(())
}

// Helper function to check and install required dependencies
fn check_dependencies(template: &str) -> Result<()> {
    // Check for wasm32-unknown-unknown target
    println!("🔍 Checking for wasm32-unknown-unknown target...");
    let wasm_check = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()?;
    
    let wasm_output = String::from_utf8_lossy(&wasm_check.stdout);
    if !wasm_output.contains("wasm32-unknown-unknown") {
        println!("⚠️ wasm32-unknown-unknown target not found. Installing...");
        let status = Command::new("rustup")
            .args(["target", "add", "wasm32-unknown-unknown"])
            .status()?;
        
        if !status.success() {
            println!("❌ Failed to install wasm32-unknown-unknown target.");
            println!("Please install it manually with: rustup target add wasm32-unknown-unknown");
        } else {
            println!("✅ wasm32-unknown-unknown target installed successfully");
        }
    } else {
        println!("✅ wasm32-unknown-unknown target is already installed");
    }
    
    // Check if we're using an edge template that needs wasm-pack
    if template.starts_with("edge/") {
        println!("🔍 Checking for wasm-pack...");
        let wasm_pack_check = Command::new("wasm-pack")
            .arg("--version")
            .output();
        
        match wasm_pack_check {
            Ok(_) => println!("✅ wasm-pack is already installed"),
            Err(_) => {
                println!("⚠️ wasm-pack not found. Installing...");
                let status = Command::new("cargo")
                    .args(["install", "wasm-pack"])
                    .status()?;
                
                if !status.success() {
                    println!("❌ Failed to install wasm-pack.");
                    println!("Please install it manually with: cargo install wasm-pack");
                } else {
                    println!("✅ wasm-pack installed successfully");
                }
            }
        }
        
        // For Cloudflare Workers templates, check for Wrangler CLI
        if template.contains("cloudflare-workers") || template.contains("cloudflare-pages") {
            println!("🔍 Checking for Cloudflare Wrangler CLI...");
            let wrangler_check = Command::new("wrangler")
                .arg("--version")
                .output();
            
            match wrangler_check {
                Ok(_) => println!("✅ Wrangler CLI is already installed"),
                Err(_) => {
                    println!("ℹ️ Wrangler CLI not found.");
                    println!("You may need to install it with: npm install -g wrangler");
                    println!("Learn more at: https://developers.cloudflare.com/workers/wrangler/install-and-update/");
                }
            }
        }
        
        // For Vercel templates, check for Vercel CLI
        if template.contains("vercel") {
            println!("🔍 Checking for Vercel CLI...");
            let vercel_check = Command::new("vercel")
                .arg("--version")
                .output();
            
            match vercel_check {
                Ok(_) => println!("✅ Vercel CLI is already installed"),
                Err(_) => {
                    println!("ℹ️ Vercel CLI not found.");
                    println!("You may need to install it with: npm install -g vercel");
                    println!("Learn more at: https://vercel.com/docs/cli");
                }
            }
        }
        
        // For Netlify templates, check for Netlify CLI
        if template.contains("netlify") {
            println!("🔍 Checking for Netlify CLI...");
            let netlify_check = Command::new("netlify")
                .arg("--version")
                .output();
            
            match netlify_check {
                Ok(_) => println!("✅ Netlify CLI is already installed"),
                Err(_) => {
                    println!("ℹ️ Netlify CLI not found.");
                    println!("You may need to install it with: npm install -g netlify-cli");
                    println!("Learn more at: https://docs.netlify.com/cli/get-started/");
                }
            }
        }
        
        // For Fastly templates, check for Fastly CLI
        if template.contains("fastly") {
            println!("🔍 Checking for Fastly CLI...");
            let fastly_check = Command::new("fastly")
                .arg("--version")
                .output();
            
            match fastly_check {
                Ok(_) => println!("✅ Fastly CLI is already installed"),
                Err(_) => {
                    println!("ℹ️ Fastly CLI not found.");
                    println!("You may need to install it from: https://developer.fastly.com/learning/tools/cli/");
                }
            }
        }
    }
    
    // Check for trunk (needed for counter, router, todo templates)
    if template == "counter" || template == "router" || template == "todo" {
        println!("🔍 Checking for Trunk...");
        let trunk_check = Command::new("trunk")
            .arg("--version")
            .output();
        
        match trunk_check {
            Ok(_) => println!("✅ Trunk is already installed"),
            Err(_) => {
                println!("⚠️ Trunk not found. Installing...");
                let status = Command::new("cargo")
                    .args(["install", "trunk", "--locked"])
                    .status()?;
                
                if !status.success() {
                    println!("❌ Failed to install Trunk.");
                    println!("Please install it manually with: cargo install trunk --locked");
                } else {
                    println!("✅ Trunk installed successfully");
                }
            }
        }
    }
    
    Ok(())
}

// Helper function to convert a string to PascalCase
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

// Helper function to handle edge templates
fn handle_edge_template(template: &str, app_path: &Path, name: &str, additional_vars: Option<serde_json::Value>) -> Result<()> {
    // Handle edge template creation manually
    let template_dir_path = PathBuf::from(format!("{}/templates/{}", env!("CARGO_MANIFEST_DIR"), template));
    
    if !template_dir_path.exists() {
        return Err(anyhow::anyhow!("Could not find template directory for {} template", template));
    }
    
    // Create src directory if needed
    fs::create_dir_all(app_path.join("src"))?;
    
    // Check if the template has lib.rs in the root, which is common for edge templates
    let lib_rs = template_dir_path.join("lib.rs");
    if lib_rs.exists() {
        let content = fs::read_to_string(&lib_rs)?;
        
        // Create handlebars instance for templating
        let mut handlebars = Handlebars::new();
        // Important: Disable escaping to preserve raw string literals and SVG content
        handlebars.register_escape_fn(handlebars::no_escape);
        
        // Create template vars
        let template_vars = json!({
            "project_name": name,
            "crate_name": name.replace("-", "_"),
            "project_name_pascal_case": to_pascal_case(&name),
            "authors": "Your Name <your.email@example.com>"
        });
        
        // Apply templating
        let rendered = handlebars.render_template(&content, &template_vars)
            .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
            
        // Write to src/lib.rs in the target directory
        fs::write(app_path.join("src").join("lib.rs"), rendered)?;
    }
    
    // Copy Cargo.toml.template and process it
    let cargo_toml_template = template_dir_path.join("Cargo.toml.template");
    let cargo_toml = template_dir_path.join("Cargo.toml");
    
    // Check for either Cargo.toml.template or Cargo.toml
    let cargo_toml_path = if cargo_toml_template.exists() {
        cargo_toml_template
    } else if cargo_toml.exists() {
        cargo_toml
    } else {
        return Err(anyhow::anyhow!("Could not find Cargo.toml or Cargo.toml.template for {} template", template));
    };
    
    let content = fs::read_to_string(&cargo_toml_path)?;
    
    // Create handlebars instance for templating
    let mut handlebars = Handlebars::new();
    // Important: Disable escaping to preserve raw string literals and SVG content
    handlebars.register_escape_fn(handlebars::no_escape);
    
    // Create template vars
    let template_vars = json!({
        "project_name": name,
        "crate_name": name.replace("-", "_"),
        "project_name_pascal_case": to_pascal_case(&name),
        "authors": "Your Name <your.email@example.com>"
    });
    
    // Apply templating
    let rendered = handlebars.render_template(&content, &template_vars)
        .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
        
    // Write to target file
    fs::write(app_path.join("Cargo.toml"), rendered)?;
    
    // Copy README.md from the template
    let readme_src = template_dir_path.join("README.md");
    
    if readme_src.exists() {
        let content = fs::read_to_string(&readme_src)?;
        
        // Create handlebars instance for templating
        let mut handlebars = Handlebars::new();
        // Important: Disable escaping to preserve raw string literals and SVG content
        handlebars.register_escape_fn(handlebars::no_escape);
        
        // Create template vars
        let template_vars = json!({
            "project_name": name,
            "crate_name": name.replace("-", "_"),
            "project_name_pascal_case": to_pascal_case(&name),
            "authors": "Your Name <your.email@example.com>"
        });
        
        // Apply templating
        let rendered = handlebars.render_template(&content, &template_vars)
            .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
            
        // Write to target file
        fs::write(app_path.join("README.md"), rendered)?;
    }
    
    // Copy any template-specific files directly from the template directory
    for entry in fs::read_dir(&template_dir_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        
        // Skip already processed files
        if file_name_str == "lib.rs" || file_name_str == "template.json" || 
           file_name_str == "Cargo.toml.template" || file_name_str == "Cargo.toml" || file_name_str == "README.md" {
            continue;
        }
        
        let source_path = entry.path();
        let target_path = app_path.join(&file_name);
        
        if source_path.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else {
            // Read the source file
            let content = fs::read_to_string(&source_path)?;
            
            // Create handlebars instance for templating
            let mut handlebars = Handlebars::new();
            // Important: Disable escaping to preserve raw string literals and SVG content
            handlebars.register_escape_fn(handlebars::no_escape);
            
            // Create template vars
            let template_vars = json!({
                "project_name": name,
                "crate_name": name.replace("-", "_"),
                "project_name_pascal_case": to_pascal_case(&name),
                "authors": "Your Name <your.email@example.com>"
            });
            
            // Apply templating
            let rendered = handlebars.render_template(&content, &template_vars)
                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                
            // Write to target file
            fs::write(target_path, rendered)?;
        }
    }
    
    Ok(())
}
