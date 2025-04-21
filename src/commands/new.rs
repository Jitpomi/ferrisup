use anyhow::{anyhow, Result};
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::Command;
use colored::*;
use dialoguer::{Input, Select};
use serde_json::{json, Value};
use handlebars::Handlebars;

// Import from our new architecture
use crate::project::templates;
use crate::utils::create_directory;

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
            
            let templates_with_desc = templates::list_templates()?;
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
    let mut _additional_vars = None;

    // Get template configuration to check for options
    let template_config = templates::get_template_config(&template)?;
    
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
        println!("Using {} as the framework", framework_selected);
        
        // Check for wasm32-unknown-unknown target
        check_dependencies("wasm32-unknown-unknown")?;
        
        // Create additional vars to use our template system's conditional files
        _additional_vars = Some(json!({
            "framework": framework_selected
        }));
        
        // Let the template system handle all the conditional file selection
        templates::apply_template("server", app_path, &name, _additional_vars.clone())?;
        
        // Print success message and return
        println!("\n🎉 Project {} created successfully!", name);
        return Ok(());
    } else if template == "serverless" {
        // Handle serverless template options
        let provider_options = ["aws", "gcp", "azure", "vercel", "netlify"];
        
        let selection = Select::new()
            .with_prompt("Which cloud provider would you like to target for your serverless function?")
            .items(&provider_options)
            .default(0)
            .interact()?;
            
        let provider_selected = provider_options[selection];
        
        // Display info about the selected provider
        let provider_info = match provider_selected {
            "aws" => "AWS Lambda is a serverless compute service that runs your code in response to events and automatically manages the underlying compute resources.",
            "gcp" => "Google Cloud Functions is a serverless execution environment for building and connecting cloud services.",
            "azure" => "Azure Functions is a serverless solution that allows you to write less code, maintain less infrastructure, and save on costs.",
            "vercel" => "Vercel Functions provide a serverless platform for deploying functions that run on-demand and scale automatically.",
            "netlify" => "Netlify Functions let you deploy server-side code that works alongside your static website or application.",
            _ => ""
        };
        
        if !provider_info.is_empty() {
            println!("ℹ️  {}", provider_info);
        }
        
        println!("Using {} as the cloud_provider", provider_selected);
        
        // For serverless, we need to handle the provider-specific files manually
        // because the conditional_files feature is not working properly with subdirectories
        let provider_dir = PathBuf::from(format!("{}/templates/serverless/{}", env!("CARGO_MANIFEST_DIR"), provider_selected));
        
        if !provider_dir.exists() {
            return Err(anyhow::anyhow!("Provider directory not found: {}", provider_selected));
        }
        
        // Copy provider-specific files directly to the target directory
        for entry in fs::read_dir(&provider_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Skip template.json and other specific files that shouldn't be copied
            if file_name_str == "template.json" {
                continue;
            }
            
            let source_path = entry.path();
            
            if source_path.is_dir() {
                let target_path = app_path.join(&file_name);
                if source_path.file_name().unwrap() == "src" {
                    // Handle src directory specially to ensure it copies to the right place
                    fs::create_dir_all(app_path.join("src"))?;
                    copy_dir_all(&source_path, &app_path.join("src"))?;
                } else {
                    copy_dir_all(&source_path, &target_path)?;
                }
            } else {
                // Read the source file
                let content = fs::read_to_string(&source_path)?;
                
                // Create handlebars instance for templating
                let mut handlebars = Handlebars::new();
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
                    
                // Determine the target file name (remove .template extension if present)
                let mut target_file_name = file_name_str.to_string();
                if target_file_name.ends_with(".template") {
                    target_file_name = target_file_name.trim_end_matches(".template").to_string();
                }
                
                // Write to target file with corrected name
                let target_path = app_path.join(target_file_name);
                fs::write(target_path, rendered)?;
            }
        }
        
        // Show serverless next steps for the selected provider
        println!("\n🎉 Project {} created successfully!", name);
        
        println!("\nNext steps:");
        println!("- cd {}", name);
        
        match provider_selected {
            "aws" => {
                println!("- # Install cargo-lambda: cargo install cargo-lambda");
                println!("- # Test locally: cargo lambda watch");
                println!("- # Deploy: cargo lambda deploy");
            },
            "gcp" => {
                println!("- # Build: docker build -t gcr.io/your-project/function .");
                println!("- # Deploy: gcloud functions deploy function-name --runtime=rust --trigger-http");
            },
            "azure" => {
                println!("- # Install Azure Functions Core Tools");
                println!("- # Test locally: func start");
                println!("- # Deploy: func azure functionapp publish your-function-app");
            },
            "vercel" => {
                println!("- # Install Vercel CLI: npm i -g vercel");
                println!("- # Deploy: vercel");
            },
            "netlify" => {
                println!("- # Install Netlify CLI: npm i -g netlify-cli");
                println!("- # Deploy: netlify deploy");
            },
            _ => {}
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
                                            _additional_vars = Some(json!(vars_map));
                                            
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
                                            }
                                            
                                            // Handle the edge template explicitly
                                            handle_edge_template(&template, app_path, &name, _additional_vars.clone())?;
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
                    _additional_vars = Some(json!(vars));
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
            templates::apply_template(&template_path, app_path, &name, _additional_vars.clone())?;
            
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
            println!("- cd {}", name);
            // Detect if this is a Dioxus workspace (web, desktop, mobile all exist)
            let web_exists = std::path::Path::new(&format!("{}/web", name)).exists();
            let desktop_exists = std::path::Path::new(&format!("{}/desktop", name)).exists();
            let mobile_exists = std::path::Path::new(&format!("{}/mobile", name)).exists();
            if web_exists || desktop_exists || mobile_exists {
                println!("- dx serve --package web    # For web application");
                println!("- dx serve --package desktop    # For desktop application");
                println!("- dx serve --package mobile    # For mobile application");
            } else {
                println!("- dx serve");
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
    } else if template.starts_with("data-science") {
        // Handle data-science template options
        let mut selected_approach = String::new();
        
        if template == "data-science" {
            // Let the user select a data science approach
            let approach_options = [
                "Data Analysis with Polars", 
                "Machine Learning with Linfa",
                "Computer Vision", 
                "Natural Language Processing",
                "Burn Neural Networks"
            ];
            
            let approach_selection = Select::new()
                .with_prompt("📊 Select a Data Science approach")
                .items(&approach_options)
                .default(0)
                .interact()?;
                
            selected_approach = approach_options[approach_selection].to_string();
            println!("📈 Selected: {}", selected_approach);
            
            match approach_selection {
                // Polars for data analysis
                0 => {
                    template = "data-science/polars-cli".to_string();
                },
                // Linfa for ML
                1 => {
                    template = "data-science/linfa-examples".to_string();
                },
                // Computer Vision
                2 => {
                    let cv_options = [
                        "Image Classification (Digit Recognition)",
                        "Image Classification (Custom CNN)"
                    ];
                    
                    let cv_selection = Select::new()
                        .with_prompt("🖼️ Select Computer Vision Task")
                        .items(&cv_options)
                        .default(0)
                        .interact()?;
                        
                    let selected_cv = cv_options[cv_selection];
                    println!("🔍 Selected: {}", selected_cv);
                    
                    match cv_selection {
                        0 => template = "data-science/burn-image-recognition".to_string(),
                        1 => template = "data-science/burn-image-classifier".to_string(),
                        _ => template = "data-science/burn-image-recognition".to_string()
                    }
                },
                // NLP
                3 => {
                    template = "data-science/polars-cli".to_string();
                    println!("⚠️ NLP templates are coming soon. Using Polars for now.");
                },
                // Burn Neural Networks
                4 => {
                    template = "data-science/burn-image-recognition".to_string();
                },
                _ => {
                    template = "data-science/polars-cli".to_string();
                }
            }
        } else {
            selected_approach = template.replace("data-science/", "");
        }
        
        println!("\n📊 Setting up {} data science project...", template.replace("data-science/", ""));
        
        // Check for required dependencies
        check_dependencies(&template)?;
        
        // For data-science/polars-cli template, we need to handle the data format selection
        if template == "data-science/polars-cli" {
            // Ask user what data format they want to use
            let data_format_options = ["CSV files", "Parquet files", "JSON data"];
            
            let format_selection = Select::new()
                .with_prompt("📊 Select the primary data format you'll be working with")
                .items(&data_format_options)
                .default(0)
                .interact()?;
                
            let selected_format = data_format_options[format_selection];
            println!("📄 Selected data format: {}", selected_format);
            
            // Create the target directory if it doesn't exist
            if !app_path.exists() {
                fs::create_dir_all(app_path)?;
            }
            
            // Get the template directory path
            let template_dir = PathBuf::from(format!("{}/templates/{}", env!("CARGO_MANIFEST_DIR"), template));
            
            // Copy only the selected data format file
            let data_file_source = match selected_format {
                "CSV files" => template_dir.join("data/example_data_csv.csv"),
                "JSON data" => template_dir.join("data/example_data_json.json"),
                "Parquet files" => template_dir.join("data/example_data_parquet.parquet"),
                _ => template_dir.join("data/example_data_csv.csv") // Default to CSV
            };
            
            let target_ext = match selected_format {
                "CSV files" => "csv",
                "JSON data" => "json",
                "Parquet files" => "parquet",
                _ => "csv" // Default to CSV
            };
            
            let data_file_target = app_path.join("data").join(format!("example_data.{}", target_ext));
            
            // Create additional vars for template substitution
            _additional_vars = Some(json!({
                "data_source": selected_format,
                "data_format": target_ext
            }));
            
            // Apply the template using the normal template system
            templates::apply_template(&template, app_path, &name, _additional_vars.clone())?;
            
            // After the template is applied, delete the unneeded data files
            // and keep only the selected one
            if selected_format != "CSV files" {
                let csv_file = app_path.join("data").join("example_data.csv");
                if csv_file.exists() {
                    let _ = fs::remove_file(&csv_file);
                }
            }
            
            if selected_format != "JSON data" {
                let json_file = app_path.join("data").join("example_data.json");
                if json_file.exists() {
                    let _ = fs::remove_file(&json_file);
                }
            }
            
            if selected_format != "Parquet files" {
                let parquet_file = app_path.join("data").join("example_data.parquet");
                if parquet_file.exists() {
                    let _ = fs::remove_file(&parquet_file);
                }
            }
            
            // If the data file hasn't been copied (which can happen with the template system),
            // copy the selected data file manually
            if !data_file_target.exists() && data_file_source.exists() {
                let _ = fs::copy(&data_file_source, &data_file_target);
            }
            
        } else {
            // For other data science templates, use the standard template system
            templates::apply_template(&template, app_path, &name, _additional_vars.clone())?;
        }
        
        println!("\n✅ {} project created successfully!", name);
    } else if template == "counter" || template == "router" || template == "todo" {
        // For Leptos templates, prepend "client/leptos/"
        let template_path = format!("client/leptos/{}", template);
        templates::apply_template(&template_path, app_path, &name, _additional_vars.clone())?;
    } else {
        // For other templates, use as is
        templates::apply_template(&template, app_path, &name, _additional_vars.clone())?;
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
pub fn handle_edge_template(template: &str, app_path: &Path, name: &str, _additional_vars: Option<serde_json::Value>) -> Result<()> {
    // Handle edge template creation manually
    let template_dir_path = PathBuf::from(format!("{}/templates/{}", env!("CARGO_MANIFEST_DIR"), template));
    
    if !template_dir_path.exists() {
        return Err(anyhow::anyhow!("Could not find template directory for {} template", template));
    }

    // Get edge template configuration to extract next steps
    let template_json_path = PathBuf::from(format!("{}/templates/edge/template.json", env!("CARGO_MANIFEST_DIR")));
    let mut next_steps = Vec::new();
    
    if template_json_path.exists() {
        let template_json_str = fs::read_to_string(&template_json_path)?;
        let template_json: Value = serde_json::from_str(&template_json_str)?;
        
        // Extract selected edge_type and provider from template path
        // Extract parts from template path like "edge/api-function/fastly"
        let parts: Vec<&str> = template.split('/').collect();
        let edge_type = if parts.len() > 1 { parts[1] } else { "api-function" };
        let provider = if parts.len() > 2 { parts[2] } else { "fastly" };
        
        println!("Using edge_type: {}, provider: {}", edge_type, provider);
        
        // Extract next steps from the redirect structure
        if let Some(redirect) = template_json.get("redirect") {
            if let Some(type_config) = redirect.get(edge_type) {
                if let Some(provider_config) = type_config.get(provider) {
                    if let Some(steps) = provider_config.get("next_steps").and_then(|s| s.as_array()) {
                        for step in steps {
                            if let Some(step_str) = step.as_str() {
                                // Replace project_name placeholder with actual name
                                let step_str = step_str.replace("{{project_name}}", name);
                                next_steps.push(step_str);
                            }
                        }
                    }
                }
            }
        }
        
        // Create .ferrisup_next_steps.json file so template manager can find it
        if !next_steps.is_empty() {
            let next_steps_json = json!({
                "next_steps": next_steps
            });
            fs::write(
                app_path.join(".ferrisup_next_steps.json"),
                serde_json::to_string_pretty(&next_steps_json)?
            )?;
            
            // Display next steps to the user directly since edge templates bypass normal template flow
            println!("\n✅ {} project created successfully!", name);
            println!("\nNext steps:");
            for step in &next_steps {
                println!("- {}", step);
            }
        }
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
            "project_name_pascal_case": to_pascal_case(&name)
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
        "project_name_pascal_case": to_pascal_case(&name)
    });
    
    // Apply templating
    let rendered = handlebars.render_template(&content, &template_vars)
        .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
        
    // Determine the target file name (remove .template extension if present)
    let mut target_file_name = "Cargo.toml";
    if cargo_toml_path.file_name().unwrap().to_string_lossy().ends_with(".template") {
        target_file_name = "Cargo.toml";
    }
    
    // Write to target file with corrected name
    let target_path = app_path.join(target_file_name);
    fs::write(target_path, rendered)?;
    
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
            "project_name_pascal_case": to_pascal_case(&name)
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
                "project_name_pascal_case": to_pascal_case(&name)
            });
            
            // Apply templating
            let rendered = handlebars.render_template(&content, &template_vars)
                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                
            // Determine the target file name (remove .template extension if present)
            let mut target_file_name = file_name_str.to_string();
            if target_file_name.ends_with(".template") {
                target_file_name = target_file_name.trim_end_matches(".template").to_string();
            }
            
            // Write to target file with corrected name
            let target_path = app_path.join(target_file_name);
            fs::write(target_path, rendered)?;
        }
    }
    
    Ok(())
}

// Helper function to handle embedded projects
pub fn handle_embedded_project(name: &str, app_path: &Path, _template: &str, _additional_vars: Option<serde_json::Value>) -> Result<()> {
    // Handle embedded template creation manually
    let template_dir_path = PathBuf::from(format!("{}/templates/embedded", env!("CARGO_MANIFEST_DIR")));
    
    if !template_dir_path.exists() {
        return Err(anyhow::anyhow!("Could not find template directory for embedded template"));
    }
    
    // Copy src directory
    let src_dir = template_dir_path.join("src");
    if src_dir.exists() {
        copy_dir_all(&src_dir, &app_path.join("src"))?;
    } else {
        return Err(anyhow::anyhow!("Could not find src directory for embedded template"));
    }
    
    // Copy Cargo.toml.template and process it
    let cargo_toml_template = template_dir_path.join("Cargo.toml.template");
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
        return Err(anyhow::anyhow!("Could not find Cargo.toml.template for embedded template"));
    }
    
    // Copy README.md from the template
    let readme_src = template_dir_path.join("README.md");
    
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
    
    Ok(())
}
