use std::path::{Path, PathBuf};
use std::fs;
// Removed unused import
use std::process::Command;
use colored::Colorize;
use anyhow::{Result, anyhow};
use dialoguer::{Select, Input};
use crate::template_manager;
use serde_json::{self, json, Value};
use handlebars::Handlebars;
use ferrisup_common::{fs::*, to_pascal_case};

// Using the ferrisup_common module's copy_directory function for directory operations

// Note: For frameworks and libraries that have official CLIs (like Dioxus and Tauri),
// we use those CLIs directly instead of maintaining our own templates.
// This ensures we're always using the most up-to-date project creation methods
// and reduces maintenance burden.

// Main execute function to handle project creation
pub fn execute(
    name: Option<&str>,
    component_type: Option<&str>,
    framework: Option<&str>,
    provider: Option<&str>,
    application_type: Option<&str>,
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
                .with_prompt("Component name")
                .interact()?
        }
    };

    // Create project directory
    let app_path = Path::new(&name);
    create_directory(app_path)?;

    // Get component type
    let mut template = match component_type {
        Some(template) => template.to_string(),
        None => {
            if no_interactive {
                return Err(anyhow!("Component type is required in non-interactive mode"));
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
                .with_prompt("Select a component type")
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
        
        // Use the framework parameter if provided, otherwise prompt for selection
        let framework_selected = if let Some(fw) = framework {
            if framework_options.contains(&fw) {
                fw.to_string()
            } else {
                println!("Warning: Provided framework '{}' is not valid for server components", fw);
                println!("Valid options are: axum, actix, poem");
                
                let selection = Select::new()
                    .with_prompt("Which web framework would you like to use?")
                    .items(&framework_options)
                    .default(0)
                    .interact()?;
                    
                framework_options[selection].to_string()
            }
        } else {
            let selection = Select::new()
                .with_prompt("Which web framework would you like to use?")
                .items(&framework_options)
                .default(0)
                .interact()?;
                
            framework_options[selection].to_string()
        };
        
        println!("Using {} as the server_framework", framework_selected);
        
        // We completely bypass the normal template processing for server templates
        // to ensure only the selected framework is included
        
        // Create the framework-specific paths
        let template_root = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));
        let framework_dir = PathBuf::from(format!("{}/server/{}", template_root, framework_selected));
        
        if !framework_dir.exists() {
            return Err(anyhow::anyhow!("Could not find template directory for {} framework", framework_selected));
        }
        
        // Process the framework-specific Cargo.toml
        let cargo_toml_path = framework_dir.join("Cargo.toml.template");
        if cargo_toml_path.exists() {
            let content = fs::read_to_string(&cargo_toml_path)?;
            
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
        
        // Process src directory
        let src_dir = framework_dir.join("src");
        if src_dir.exists() {
            // Create the target src directory
            fs::create_dir_all(app_path.join("src"))?;
            
            // Copy all files from the source src directory
            for entry in fs::read_dir(&src_dir)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let source_path = entry.path();
                let target_path = app_path.join("src").join(&file_name);
                
                if source_path.is_dir() {
                    copy_directory(&source_path, &target_path)?;
                } else {
                    // Read file content
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
                    let rendered = match handlebars.render_template(&content, &template_vars) {
                        Ok(result) => result,
                        Err(e) => {
                            println!("Warning: Template parsing error in {}: {}", file_name.to_string_lossy(), e);
                            content
                        }
                    };
                    
                    // Write to target file
                    fs::write(target_path, rendered)?;
                }
            }
        } else {
            return Err(anyhow::anyhow!("Could not find src directory for {} framework", framework_selected));
        }
        
        // Process README.md
        let readme_path = framework_dir.join("README.md");
        if readme_path.exists() {
            let content = fs::read_to_string(&readme_path)?;
            
            // Create handlebars instance for templating
            let mut handlebars = Handlebars::new();
            handlebars.register_escape_fn(handlebars::no_escape);
            
            // Create template vars
            let template_vars = json!({
                "project_name": name,
                "project_name_pascal_case": to_pascal_case(&name),
                "server_framework": framework_selected
            });
            
            // Apply templating
            let rendered = match handlebars.render_template(&content, &template_vars) {
                Ok(result) => result,
                Err(e) => {
                    println!("Warning: Template parsing error in README.md: {}", e);
                    content
                }
            };
            
            // Write to target file
            fs::write(app_path.join("README.md"), rendered)?;
        } else {
            // Try to use the common README.md
            let common_readme_path = PathBuf::from(format!("{}/server/README.md", template_root));
            if common_readme_path.exists() {
                let content = fs::read_to_string(&common_readme_path)?;
                
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
                let rendered = match handlebars.render_template(&content, &template_vars) {
                    Ok(result) => result,
                    Err(e) => {
                        println!("Warning: Template parsing error in README.md: {}", e);
                        content
                    }
                };
                
                // Write to target file
                fs::write(app_path.join("README.md"), rendered)?;
            }
        }
        
        // Copy any other framework-specific files (excluding the directories we already processed)
        for entry in fs::read_dir(&framework_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Skip directories and files we've already processed
            if file_name_str == "src" || file_name_str == "Cargo.toml.template" || 
               file_name_str == "README.md" || file_name_str == "template.json" {
                continue;
            }
            
            let source_path = entry.path();
            let target_path = app_path.join(&file_name);
            
            if source_path.is_dir() {
                copy_directory(&source_path, &target_path)?;
            } else {
                // Read file content
                let content = fs::read_to_string(&source_path)?;
                
                // Create handlebars instance for templating
                let mut handlebars = Handlebars::new();
                handlebars.register_escape_fn(handlebars::no_escape);
                
                // Create template vars
                let template_vars = json!({
                    "project_name": name,
                    "project_name_pascal_case": to_pascal_case(&name),
                    "server_framework": framework_selected
                });
                
                // Apply templating
                let rendered = match handlebars.render_template(&content, &template_vars) {
                    Ok(result) => result,
                    Err(e) => {
                        println!("Warning: Template parsing error in {}: {}", file_name_str, e);
                        content
                    }
                };
                
                // Write to target file
                fs::write(target_path, rendered)?;
            }
        }
        
        // Display success message and next steps
        println!("\n✅ {} project created successfully!", name);
        
        // Get next steps for the selected framework
        let template_json_path = PathBuf::from(format!("{}/templates/server/template.json", env!("CARGO_MANIFEST_DIR")));
        if template_json_path.exists() {
            let template_json = fs::read_to_string(&template_json_path)?;
            let template_config: Value = serde_json::from_str(&template_json)?;
            let next_steps_key = format!("next_steps_{}", framework_selected);
            
            println!("\n🚀 Next steps:");
            if let Some(next_steps) = template_config.get(&next_steps_key).and_then(|s| s.as_array()) {
                for step in next_steps {
                    if let Some(step_str) = step.as_str() {
                        let processed_step = step_str.replace("{{project_name}}", &name);
                        println!("  {}", processed_step);
                    }
                }
            } else {
                // Generic next steps if framework-specific ones aren't found
                println!("  1. cd {}", name);
                println!("  2. cargo run");
                println!("  3. Visit http://localhost:3000 in your browser");
            }
        } else {
            // Generic next steps if template.json isn't found
            println!("\n🚀 Next steps:");
            println!("  1. cd {}", name);
            println!("  2. cargo run");
            println!("  3. Visit http://localhost:3000 in your browser");
        }
        
        // Skip the rest of the template handling code
        return Ok(());
    } else if template == "serverless" {
        // Get cloud provider for serverless function
        let providers = ["aws", "gcp", "azure", "vercel", "netlify"];
        
        // Use the provider parameter if provided, otherwise prompt for selection
        let selected_provider = if let Some(prov) = provider {
            if providers.contains(&prov) {
                println!("Using {} as the cloud provider for your serverless function", prov);
                prov.to_string()
            } else {
                println!("Warning: Provided provider '{}' is not valid for serverless components", prov);
                println!("Valid options are: aws, gcp, azure, vercel, netlify");
                
                let selection = Select::new()
                    .with_prompt("Which cloud provider would you like to target for your serverless function?")
                    .items(&providers)
                    .default(0)
                    .interact()?;
                    
                providers[selection].to_string()
            }
        } else {
            let selection = Select::new()
                .with_prompt("Which cloud provider would you like to target for your serverless function?")
                .items(&providers)
                .default(0)
                .interact()?;
                
            providers[selection].to_string()
        };
        
        // Create additional variables with the selected provider
        let mut vars = serde_json::Map::new();
        vars.insert("cloud_provider".to_string(), json!(selected_provider));
        
        // Use the template manager for serverless template with the selected provider
        template_manager::apply_template(&template, app_path, &name, Some(serde_json::Value::Object(vars)))?;
        
        // Clean up provider-specific directories that weren't selected
        let providers = ["aws", "gcp", "azure", "vercel", "netlify"];
        let template_json_path = PathBuf::from(format!("{}/templates/serverless/template.json", env!("CARGO_MANIFEST_DIR")));
        let template_content = fs::read_to_string(&template_json_path)?;
        let template_json: serde_json::Value = serde_json::from_str(&template_content)?;
        
        // Get the selected cloud provider from the template variables
        let _selected_provider = if let Some(vars) = template_json.get("variables").and_then(|v| v.as_object()) {
            if let Some(provider) = vars.get("cloud_provider").and_then(|p| p.as_str()) {
                provider
            } else {
                // Default to aws if not found
                "aws"
            }
        } else {
            // Default to aws if variables not found
            "aws"
        };
        
        // Remove all provider directories
        for provider in providers {
            let provider_dir = app_path.join(provider);
            if provider_dir.exists() && provider_dir.is_dir() {
                fs::remove_dir_all(provider_dir)?;
            }
        }
        
        // Remove template.json file
        let template_json_file = app_path.join("template.json");
        if template_json_file.exists() {
            fs::remove_file(template_json_file)?;
        }
        
        // Remove main.rs file in root directory if it exists (should be in src/main.rs)
        let root_main_rs = app_path.join("main.rs");
        if root_main_rs.exists() {
            fs::remove_file(root_main_rs)?;
        }
        
        return Ok(());
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
                            
                        // Use the application_type parameter if provided, otherwise prompt for selection
                        let selected_app_type = if let Some(app_type) = application_type {
                            if app_type_options.contains(&app_type) {
                                println!("Using {} as the edge application type", app_type);
                                app_type.to_string()
                            } else {
                                println!("Warning: Provided application type '{}' is not valid for edge components", app_type);
                                println!("Valid options are: {}", app_type_options.join(", "));
                                
                                // Let user select application type
                                let app_type_selection = Select::new()
                                    .with_prompt("Select edge application type")
                                    .items(&app_type_display)
                                    .default(0)
                                    .interact()?;
                                    
                                app_type_options[app_type_selection].to_string()
                            }
                        } else {
                            // Let user select application type
                            let app_type_selection = Select::new()
                                .with_prompt("Select edge application type")
                                .items(&app_type_display)
                                .default(0)
                                .interact()?;
                                
                            app_type_options[app_type_selection].to_string()
                        };

                        println!("Selected application type: {}", selected_app_type);

                        // Second level: Get provider options for the selected type
                        let provider_field = match selected_app_type.as_str() {
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
                                
                                // Clone the selected_app_type for later use
                                let app_type_clone = selected_app_type.clone();
                                // No need to clone a &str reference
                                let provider_clone = selected_provider;
                                
                                // Check if there's a redirect in the template config
                                if let Some(redirect) = edge_template_json.get("redirect") {
                                    if let Some(app_redirect) = redirect.get(&selected_app_type) {
                                        if let Some(_provider_redirect) = app_redirect.get(&selected_provider) {
                                            // We found a redirect configuration, use it to determine the template
                                            template = format!("edge/{}/{}", app_type_clone, provider_clone);
                                            additional_vars = Some(json!(vars_map));
                                            
                                            // Check if the template directory exists
                                            let full_template_path = format!("{}/templates/{}", env!("CARGO_MANIFEST_DIR"), template);
                                            println!("Using template: {}", template);
                                            
                                            // Check if the directory exists
                                            if !Path::new(&full_template_path).exists() {
                                                return Err(anyhow!("Template directory not found: {}", full_template_path));
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
        // Skip options handling for templates we're handling manually or data science templates
        if template != "server" && template != "serverless" && !template.starts_with("data-science/") {
            // Check if the template has prompts that need user input
            if let Some(prompts) = template_config.get("prompts").and_then(|p| p.as_array()) {
                println!("\n📊 Data Science Project Configuration\n");
                
                let mut vars = serde_json::Map::new();
                
                for prompt in prompts {
                    if let (Some(name), Some(question), Some(options)) = (
                        prompt.get("name").and_then(|n| n.as_str()),
                        prompt.get("question").and_then(|q| q.as_str()),
                        prompt.get("options").and_then(|o| o.as_array())
                    ) {
                        let option_values: Vec<&str> = options
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect();
                        
                        if !option_values.is_empty() {
                            let default_idx = prompt.get("default")
                                .and_then(|d| d.as_str())
                                .and_then(|d| option_values.iter().position(|&v| v == d))
                                .unwrap_or(0);
                            
                            let selection = Select::new()
                                .with_prompt(question)
                                .items(&option_values)
                                .default(default_idx)
                                .interact()?;
                            
                            let selected_value = option_values[selection];
                            vars.insert(name.to_string(), json!(selected_value));
                            
                            // Print the selection
                            println!("\n📊 {} {}: {}", 
                                match name {
                                    "data_source" => "What type of data will you be working with?",
                                    "analysis_type" => "What type of analysis do you plan to perform?",
                                    "visualization" => "Do you need data visualization capabilities?",
                                    _ => question
                                },
                                selected_value,
                                if name == "visualization" && selected_value == "yes" {
                                    "\n📈 Visualization support will be added to your project."
                                } else {
                                    ""
                                }
                            );
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
        
        // Use the framework parameter if provided, otherwise prompt for selection
        let framework_selected = if let Some(fw) = framework {
            if frameworks.contains(&fw) {
                fw.to_string()
            } else {
                println!("Warning: Provided framework '{}' is not valid for client components", fw);
                println!("Valid options are: dioxus, tauri, leptos");
                
                let selection = Select::new()
                    .with_prompt("Select Rust client framework")
                    .items(&frameworks)
                    .default(0)
                    .interact()?;
                    
                frameworks[selection].to_string()
            }
        } else {
            let selection = Select::new()
                .with_prompt("Select Rust client framework")
                .items(&frameworks)
                .default(0)
                .interact()?;
                
            frameworks[selection].to_string()
        };
        
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
            
            // For Leptos templates, prepend "client/leptos/"
            let template_path = format!("client/leptos/{}", template);
            
            if let Err(e) = template_manager::apply_template(
                &template_path,
                app_path,
                &name,
                additional_vars,
            ) {
                return Err(e);
            }
            
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
            
            // Check for wasm32-unknown-unknown target (required for web)
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
            
            // Check for aarch64-apple-ios-sim target (required for iOS simulator)
            println!("🔍 Checking for aarch64-apple-ios-sim target...");
            let ios_check = Command::new("rustup")
                .args(["target", "list", "--installed"])
                .output()?;
            
            let ios_output = String::from_utf8_lossy(&ios_check.stdout);
            if !ios_output.contains("aarch64-apple-ios-sim") {
                println!("⚠️ aarch64-apple-ios-sim target not found.");
                println!("This is something that Rustup can do for you. Rustup is Rust's toolchain");
                println!("installer and manager, and it can add the iOS simulator target to your Rust installation.");
                
                // Ask if user wants to install iOS simulator target
                let install_ios = dialoguer::Confirm::new()
                    .with_prompt("Would you like to install the iOS simulator target?")
                    .default(true)
                    .interact()?;
                
                if install_ios {
                    println!("Installing aarch64-apple-ios-sim target...");
                    let status = Command::new("rustup")
                        .args(["target", "add", "aarch64-apple-ios-sim"])
                        .status()?;
                    
                    if !status.success() {
                        println!("❌ Failed to install aarch64-apple-ios-sim target.");
                        println!("Please install it manually with: rustup target add aarch64-apple-ios-sim");
                    } else {
                        println!("✅ aarch64-apple-ios-sim target installed successfully");
                    }
                } else {
                    println!("Skipping iOS simulator target installation.");
                    println!("You can install it later with: rustup target add aarch64-apple-ios-sim");
                }
            } else {
                println!("✅ aarch64-apple-ios-sim target is already installed");
            }
            
            // Check for Android targets if on macOS or Linux
            #[cfg(any(target_os = "macos", target_os = "linux"))]
            {
                println!("🔍 Checking for Android targets...");
                let android_check = Command::new("rustup")
                    .args(["target", "list", "--installed"])
                    .output()?;
                
                let android_output = String::from_utf8_lossy(&android_check.stdout);
                if !android_output.contains("aarch64-linux-android") {
                    println!("⚠️ aarch64-linux-android target not found.");
                    println!("For most modern Android devices, you'll want the aarch64-linux-android target.");
                    
                    // Ask if user wants to install Android target
                    let install_android = dialoguer::Confirm::new()
                        .with_prompt("Would you like to install the Android target?")
                        .default(true)
                        .interact()?;
                    
                    if install_android {
                        println!("Installing aarch64-linux-android target...");
                        let status = Command::new("rustup")
                            .args(["target", "add", "aarch64-linux-android"])
                            .status()?;
                        
                        if !status.success() {
                            println!("❌ Failed to install aarch64-linux-android target.");
                            println!("Please install it manually with: rustup target add aarch64-linux-android");
                        } else {
                            println!("✅ aarch64-linux-android target installed successfully");
                        }
                    } else {
                        println!("Skipping Android target installation.");
                        println!("You can install it later with: rustup target add aarch64-linux-android");
                    }
                } else {
                    println!("✅ aarch64-linux-android target is already installed");
                }
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
        // For data science templates, check if framework is provided via command line
        if let Some(fw) = framework {
            match fw.to_lowercase().as_str() {
                "polars" => {
                    template = "data-science/polars-cli".to_string();
                    println!("📈 Selected: Data Analysis with Polars");
                },
                "linfa" => {
                    template = "data-science/linfa-examples".to_string();
                    println!("🔍 Selected: Machine Learning with Linfa");
                },
                _ => {
                    println!("Warning: Provided framework '{}' is not valid for data-science components", fw);
                    println!("Valid options are: polars, linfa");
                    
                    // If invalid framework is provided, fall back to interactive selection
                    let framework_options = vec![
                        "Data Analysis with Polars",
                        "Machine Learning with Linfa"
                    ];
                    
                    let selection = Select::new()
                        .with_prompt("📊 Select a Data Science approach")
                        .default(0)
                        .items(&framework_options)
                        .interact()?;
                        
                    match selection {
                        0 => {
                            template = "data-science/polars-cli".to_string();
                            println!("📈 Selected: Data Analysis with Polars");
                        },
                        1 => {
                            template = "data-science/linfa-examples".to_string();
                            println!("🔍 Selected: Machine Learning with Linfa");
                        },
                        _ => {
                            template = "data-science/polars-cli".to_string();
                            println!("📈 Selected: Data Analysis with Polars (default)");
                        }
                    }
                }
            }
        } else {
            // No framework provided, use interactive selection
            let framework_options = vec![
                "Data Analysis with Polars",
                "Machine Learning with Linfa"
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
                _ => {
                    // Fallback
                    template = "data-science/polars-cli".to_string();
                    println!("📈 Selected: Data Analysis with Polars (default)");
                }
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
        
        // Create a variable to store the framework selection for template variables
        let framework_selection;
        
        // Check if the user provided a framework parameter
        let use_embassy = if let Some(fw) = framework {
            match fw.to_lowercase().as_str() {
                "embassy" => {
                    println!("Using Embassy framework for embedded development");
                    framework_selection = "Yes, use Embassy framework".to_string();
                    true
                },
                "none" | "standard" => {
                    println!("Using standard embedded template without a framework");
                    framework_selection = "No, use standard embedded template".to_string();
                    false
                },
                _ => {
                    println!("Warning: Provided framework '{}' is not valid for embedded components", fw);
                    println!("Valid options are: embassy, none");
                    
                    // Prompt for framework selection
                    let frameworks = vec!["No, use standard embedded template", "Yes, use Embassy framework"];
                    let selection = Select::new()
                        .with_prompt("Do you want to use an embedded framework?")
                        .items(&frameworks)
                        .default(0)
                        .interact()?;
                    
                    framework_selection = frameworks[selection].to_string();
                    framework_selection == "Yes, use Embassy framework"
                }
            }
        } else if let Some(vars) = &additional_vars {
            // Fall back to additional_vars if no framework parameter
            if let Some(framework_option) = vars.get("framework").and_then(|f| f.as_str()) {
                framework_selection = framework_option.to_string();
                framework_option == "Yes, use Embassy framework"
            } else {
                // Prompt for framework selection
                let frameworks = vec!["No, use standard embedded template", "Yes, use Embassy framework"];
                let selection = Select::new()
                    .with_prompt("Do you want to use an embedded framework?")
                    .items(&frameworks)
                    .default(0)
                    .interact()?;
                
                framework_selection = frameworks[selection].to_string();
                framework_selection == "Yes, use Embassy framework"
            }
        } else {
            // Prompt for framework selection
            let frameworks = vec!["No, use standard embedded template", "Yes, use Embassy framework"];
            let selection = Select::new()
                .with_prompt("Do you want to use an embedded framework?")
                .items(&frameworks)
                .default(0)
                .interact()?;
            
            framework_selection = frameworks[selection].to_string();
            framework_selection == "Yes, use Embassy framework"
        };
        
        // Create or update additional variables with the framework selection
        let mut vars_map = if let Some(ref existing_vars) = additional_vars {
            if let Some(existing_map) = existing_vars.as_object() {
                // Clone the existing map
                existing_map.clone()
            } else {
                serde_json::Map::new()
            }
        } else {
            serde_json::Map::new()
        };
        
        // Add or update the framework selection
        vars_map.insert("framework".to_string(), json!(framework_selection));
        
        // Update additional_vars with the merged variables
        additional_vars = Some(json!(vars_map));
        
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
            
            // Check if ESP toolchain is needed (for ESP32 chips)
            let is_esp_chip = mcu_chip.starts_with("esp");
            if is_esp_chip {
                println!("🔍 Checking for ESP Rust toolchain...");
                
                // Check if ESP toolchain is installed
                let esp_check = Command::new("rustup")
                    .args(["toolchain", "list"])
                    .output()?;
                
                let toolchains = String::from_utf8_lossy(&esp_check.stdout);
                let esp_installed = toolchains.contains("esp");
                
                if !esp_installed {
                    println!("⚠️ ESP Rust toolchain not found. Installing...");
                    
                    // Install ESP toolchain
                    let install_status = Command::new("rustup")
                        .args(["toolchain", "install", "esp"])
                        .status()?;
                    
                    if !install_status.success() {
                        println!("❌ Failed to install ESP toolchain.");
                        println!("Please install it manually with: rustup toolchain install esp");
                        println!("See https://esp-rs.github.io/book/installation/index.html for more information.");
                    } else {
                        println!("✅ ESP toolchain installed successfully");
                    }
                } else {
                    println!("✅ ESP toolchain is already installed");
                }
            }
            
            // Move the generated project to the target directory
            let project_dir = parent_dir.join(&name);
            if project_dir.exists() {
                // Copy all files from the generated project to the target directory
                let target_dir = Path::new(&name);
                if !target_dir.exists() {
                    fs::create_dir_all(target_dir)?;
                }
                
                copy_directory(&project_dir, target_dir)?;
                
                // Clean up the temporary directory
                fs::remove_dir_all(parent_dir)?;
                
                println!("🎉 Embassy project {} created successfully!", name);
                println!("\nNext steps:");
                println!("  cd {}", name);
                
                // Add ESP-specific instructions if needed
                if mcu_chip.starts_with("esp") {
                    println!("\nℹ️ This project uses the ESP Rust toolchain");
                    println!("  Make sure it's installed with: rustup toolchain install esp");
                    println!("  For more information: https://esp-rs.github.io/book/installation/index.html");
                    println!("\n  # Build the project");
                    println!("  cargo build --release");
                    println!("  # Flash to device");
                    println!("  cargo run --release");
                    println!("\nℹ️ If you see an error about custom toolchain, run:");
                    println!("  rustup toolchain install esp");
                } else {
                    println!("  # Build the project");
                    println!("  cargo build --release");
                    println!("  # Run the project");
                    println!("  cargo run --release");
                }
                
                return Ok(());
            } else {
                println!("❌ Failed to create Embassy project.");
                return Err(anyhow!("Failed to create Embassy project: Project directory not found"));
            }
        } else {
            // Standard embedded template
            // Get microcontroller target from options
            let mcu_target = if let Some(vars) = &additional_vars {
                if vars.get("mcu_target").is_some() {
                    // If mcu_target is explicitly provided in additional_vars, use it
                    vars.get("mcu_target")
                        .and_then(|t| t.as_str())
                        .unwrap_or("rp2040")
                        .to_string()
                } else {
                    // Otherwise prompt for selection
                    let mcu_targets = vec!["rp2040", "stm32", "esp32", "arduino"];
                    let selection = Select::new()
                        .with_prompt("Select microcontroller target")
                        .items(&mcu_targets)
                        .default(0)
                        .interact()?;
                    
                    let selected_target = mcu_targets[selection].to_string();
                    
                    // Update additional_vars with the selected target
                    if let Some(ref mut vars_obj) = additional_vars {
                        if let Some(vars_map) = vars_obj.as_object_mut() {
                            vars_map.insert("mcu_target".to_string(), json!(selected_target.clone()));
                        }
                    }
                    
                    selected_target
                }
            } else {
                // No additional_vars, prompt for selection
                let mcu_targets = vec!["rp2040", "stm32", "esp32", "arduino"];
                let selection = Select::new()
                    .with_prompt("Select microcontroller target")
                    .items(&mcu_targets)
                    .default(0)
                    .interact()?;
                
                // Create additional_vars with the selected target
                let selected_target = mcu_targets[selection].to_string();
                let mut vars_map = serde_json::Map::new();
                vars_map.insert("mcu_target".to_string(), json!(selected_target.clone()));
                additional_vars = Some(json!(vars_map));
                
                selected_target
            };
            
            println!("Using {} as the microcontroller target", mcu_target);
            
            // Create target-specific dependencies string
            let mcu_target_deps = match mcu_target.as_str() {
                "rp2040" => "rp2040-hal = \"0.9\"\nrp2040-boot2 = \"0.3\"\nusb-device = \"0.2\"\nusbd-serial = \"0.1\"",
                "stm32" => "stm32f4xx-hal = { version = \"0.17\", features = [\"stm32f411\"] }",
                "esp32" => "esp32-hal = \"0.16\"\nesp-backtrace = \"0.9\"\nesp-println = \"0.6\"",
                "arduino" => "arduino-hal = { git = \"https://github.com/rahix/avr-hal\", rev = \"7dfa6d322b9df98b2d98afe0e14a97afe0187ac1\" }\navr-device = \"0.5\"\nufmt = \"0.2\"",
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
                    // Check if the target is installed
                    let output = Command::new("rustup")
                        .args(["target", "list", "--installed"])
                        .output()
                        .unwrap_or_else(|_| panic!("Failed to execute rustup command"));
                    
                    let installed_targets = String::from_utf8_lossy(&output.stdout);
                    let target_installed = installed_targets.contains(rust_target);
                    
                    if !target_installed {
                        println!("🔍 Checking for {} target...", rust_target);
                        println!("❌ {} target is not installed", rust_target);
                    } else {
                        println!("🔍 Checking for {} target...", rust_target);
                        println!("✅ {} target is already installed", rust_target);
                    }
                    
                    println!("\nℹ️ Setup instructions for Arduino development:");
                    println!("  1. Install the AVR target:  rustup target add {}", rust_target);
                    println!("  2. Install ravedude:       cargo install ravedude");
                    println!("  3. Build the project:      cargo build --target {}", rust_target);
                    println!("  4. Flash to Arduino:       cargo run --target {}", rust_target);
                    println!("\nNote: arduino-hal is sourced from GitHub, not crates.io");
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
        // For data science templates, handle the prompts directly
        if template.starts_with("data-science/") {
            // Get the template configuration to access prompts
            let template_config = template_manager::get_template_config(&template)?;
            
            println!("\n📊 Data Science Project Configuration\n");
            
            // Create a map to store the user's selections
            let mut template_vars = serde_json::Map::new();
            
            // Process prompts if they exist
            if let Some(prompts) = template_config.get("prompts").and_then(|p| p.as_array()) {
                for prompt in prompts {
                    if let (Some(name), Some(question), Some(options)) = (
                        prompt.get("name").and_then(|n| n.as_str()),
                        prompt.get("question").and_then(|q| q.as_str()),
                        prompt.get("options").and_then(|o| o.as_array())
                    ) {
                        let option_values: Vec<&str> = options
                            .iter()
                            .filter_map(|v| v.as_str())
                            .collect();
                        
                        if !option_values.is_empty() {
                            let default_idx = prompt.get("default")
                                .and_then(|d| d.as_str())
                                .and_then(|d| option_values.iter().position(|&v| v == d))
                                .unwrap_or(0);
                            
                            // Create the selection prompt
                            let selection = Select::new()
                                .with_prompt(question)
                                .items(&option_values)
                                .default(default_idx)
                                .interact()?;
                            
                            let selected_value = option_values[selection];
                            template_vars.insert(name.to_string(), json!(selected_value));
                        }
                    }
                }
            }
            
            // Add any additional variables that might have been set earlier
            if let Some(ref additional) = additional_vars {
                if let Some(obj) = additional.as_object() {
                    for (k, v) in obj {
                        if !template_vars.contains_key(k) {
                            template_vars.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            
            // Debug output only when in verbose mode
            if std::env::var("FERRISUP_VERBOSE").is_ok() {
                println!("Template variables: {}", json!(template_vars));
            }
            
            // Apply the template with the user's selections
            template_manager::apply_template(&template, app_path, &name, Some(json!(template_vars)))?;
        } else {
            // For non-data-science templates, use the original approach
            template_manager::apply_template(&template, app_path, &name, additional_vars)?;
        }
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
        // Disable strict mode to handle more complex templates
        handlebars.set_strict_mode(false);
        
        // Create template vars
        let mut template_vars = json!({
            "project_name": name,
            "crate_name": name.replace("-", "_"),
            "project_name_pascal_case": to_pascal_case(&name),
            "authors": "Your Name <your.email@example.com>"
        });
        
        // Merge additional variables if provided
        if let Some(add_vars) = additional_vars.as_ref() {
            if let Some(obj) = add_vars.as_object() {
                if let Some(obj_mut) = template_vars.as_object_mut() {
                    for (key, value) in obj {
                        obj_mut.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        
        // Apply templating with better error handling
        let rendered = match handlebars.render_template(&content, &template_vars) {
            Ok(result) => result,
            Err(e) => {
                println!("Warning: Template parsing error in lib.rs: {}", e);
                println!("Proceeding with unmodified template content");
                content
            }
        };
            
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
    // Disable strict mode to handle more complex templates
    handlebars.set_strict_mode(false);
    
    // Create template vars
    let mut template_vars = json!({
        "project_name": name,
        "crate_name": name.replace("-", "_"),
        "project_name_pascal_case": to_pascal_case(&name),
        "authors": "Your Name <your.email@example.com>"
    });
    
    // Merge additional variables if provided
    if let Some(add_vars) = additional_vars.as_ref() {
        if let Some(obj) = add_vars.as_object() {
            if let Some(obj_mut) = template_vars.as_object_mut() {
                for (key, value) in obj {
                    obj_mut.insert(key.clone(), value.clone());
                }
            }
        }
    }
    
    // Apply templating with better error handling
    let rendered = match handlebars.render_template(&content, &template_vars) {
        Ok(result) => result,
        Err(e) => {
            println!("Warning: Template parsing error in Cargo.toml: {}", e);
            println!("Proceeding with unmodified template content");
            content
        }
    };
        
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
        // Disable strict mode to handle more complex templates
        handlebars.set_strict_mode(false);
        
        // Create template vars
        let mut template_vars = json!({
            "project_name": name,
            "crate_name": name.replace("-", "_"),
            "project_name_pascal_case": to_pascal_case(&name),
            "authors": "Your Name <your.email@example.com>"
        });
        
        // Merge additional variables if provided
        if let Some(add_vars) = additional_vars.as_ref() {
            if let Some(obj) = add_vars.as_object() {
                if let Some(obj_mut) = template_vars.as_object_mut() {
                    for (key, value) in obj {
                        obj_mut.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        
        // Apply templating with better error handling
        let rendered = match handlebars.render_template(&content, &template_vars) {
            Ok(result) => result,
            Err(e) => {
                println!("Warning: Template parsing error in README.md: {}", e);
                println!("Proceeding with unmodified template content");
                content
            }
        };
            
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
            copy_directory(&source_path, &target_path)?;
        } else {
            // Read the source file
            let content = fs::read_to_string(&source_path)?;
            
            // Create handlebars instance for templating
            let mut handlebars = Handlebars::new();
            // Important: Disable escaping to preserve raw string literals and SVG content
            handlebars.register_escape_fn(handlebars::no_escape);
            // Disable strict mode to handle more complex templates
            handlebars.set_strict_mode(false);
            
            // Create template vars
            let mut template_vars = json!({
                "project_name": name,
                "crate_name": name.replace("-", "_"),
                "project_name_pascal_case": to_pascal_case(&name),
                "authors": "Your Name <your.email@example.com>"
            });
            
            // Merge additional variables if provided
            if let Some(add_vars) = additional_vars.as_ref() {
                if let Some(obj) = add_vars.as_object() {
                    if let Some(obj_mut) = template_vars.as_object_mut() {
                        for (key, value) in obj {
                            obj_mut.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
            
            // Apply templating with better error handling
            let rendered = match handlebars.render_template(&content, &template_vars) {
                Ok(result) => result,
                Err(e) => {
                    println!("Warning: Template parsing error in {}: {}", file_name_str, e);
                    println!("Proceeding with unmodified template content");
                    content
                }
            };
                
            // Write to target file
            fs::write(target_path, rendered)?;
        }
    }
    // Read template.json to get next steps
    let template_json_path = template_dir_path.join("template.json");
    if template_json_path.exists() {
        let template_json_content = fs::read_to_string(&template_json_path)?;
        if let Ok(template_config) = serde_json::from_str::<serde_json::Value>(&template_json_content) {
            if let Some(next_steps) = template_config.get("next_steps").and_then(|s| s.as_array()) {
                println!("\n✅ {} project created successfully!", name.green());
                println!("\n{}", "Next steps:".bold().green());
                
                // Create a handlebars registry for processing templates
                let mut handlebars = Handlebars::new();
                handlebars.register_escape_fn(handlebars::no_escape);
                
                // Create template vars
                let mut data = serde_json::Map::new();
                data.insert("project_name".to_string(), json!(name));
                
                // Add template variables to the data
                if let Some(ref vars) = additional_vars {
                    if let Some(obj) = vars.as_object() {
                        for (k, v) in obj {
                            data.insert(k.clone(), v.clone());
                        }
                    }
                }
                
                // Process each next step
                for step in next_steps {
                    if let Some(step_str) = step.as_str() {
                        // Replace {{project_name}} with the actual project name
                        let step_text = match handlebars.render_template(step_str, &json!(data)) {
                            Ok(rendered) => rendered,
                            Err(_) => step_str.replace("{{project_name}}", name),
                        };
                        println!("- {}", step_text);
                    }
                }
                return Ok(());
            }
        }
    }
    
    println!("\n🎉 Project {} created successfully!", name);
    
    Ok(())
}
