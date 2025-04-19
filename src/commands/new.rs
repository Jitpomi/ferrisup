use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, anyhow};
use dialoguer::{Select, Input};
use crate::template_manager;
use crate::utils::create_directory;
use serde_json::{self, json};
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
            let templates: Vec<&str> = templates_with_desc.iter().map(|(name, _)| name.as_str()).collect();
            
            let selection = Select::new()
                .with_prompt("Select a template")
                .items(&templates)
                .default(0)
                .interact()?;
                
            templates[selection].to_string()
        }
    };

    // Declare additional_vars here
    let mut additional_vars = None;

    // Get template configuration to check for options
    let template_config = template_manager::get_template_config(&template)?;
    
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
                        
                        // Display help text if available
                        if let Some(help) = template_config.get("help").and_then(|h| h.as_object()) {
                            if let Some(help_text) = help.get(selected_value).and_then(|t| t.as_str()) {
                                println!("ℹ️  {}", help_text);
                            }
                        }
                        
                        println!("Using {} as the {}", selected_value, name);
                    }
                }
            }
        }
        
        if !vars.is_empty() {
            additional_vars = Some(serde_json::Value::Object(vars));
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
            println!("  dx serve");
            
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

    // Apply the template using the template_manager
    if template.starts_with("client/") {
        // Template path is already fully qualified
        template_manager::apply_template(&template, app_path, &name, additional_vars.clone())?;
    } else if template == "server" {
        // For server templates, we'll handle everything manually to avoid the duplicate prompt issue
        
        // First prompt for framework selection
        let framework_options = vec![
            "axum",
            "actix", 
            "poem"
        ];
        
        let selection = Select::new()
            .with_prompt("Server framework")
            .default(0)
            .items(&framework_options)
            .interact()?;
            
        let framework_selected = framework_options[selection];
        println!("Using {} as the server_framework", framework_selected);
        
        // Create target directories
        if !app_path.exists() {
            fs::create_dir_all(app_path)?;
        }
        
        // Create src directory
        let src_dir = app_path.join("src");
        if !src_dir.exists() {
            fs::create_dir_all(&src_dir)?;
        }
        
        // Define paths for source files based on selected framework
        let template_dir = format!("{}/templates/server/{}", env!("CARGO_MANIFEST_DIR"), framework_selected);
        let template_dir_path = PathBuf::from(&template_dir);
        
        // Copy main.rs
        let main_rs_src = template_dir_path.join("src").join("main.rs");
        if main_rs_src.exists() {
            let content = fs::read_to_string(&main_rs_src)?;
            
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
            fs::write(src_dir.join("main.rs"), rendered)?;
        } else {
            return Err(anyhow::anyhow!("Could not find main.rs for {} framework", framework_selected));
        }
        
        // Copy Cargo.toml
        let cargo_toml_src = template_dir_path.join("Cargo.toml.template");
        if cargo_toml_src.exists() {
            let content = fs::read_to_string(&cargo_toml_src)?;
            
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
            let rendered = handlebars.render_template(&content, &template_vars)
                .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
                
            // Write to target file
            fs::write(app_path.join("Cargo.toml"), rendered)?;
        } else {
            return Err(anyhow::anyhow!("Could not find Cargo.toml.template for {} framework", framework_selected));
        }
        
        // Copy README.md from the framework-specific template
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
        
        // Check for wasm target 
        println!("🔍 Checking for wasm32-unknown-unknown target...");
        let output = Command::new("rustup")
            .args([
                "target",
                "list",
                "--installed"
            ])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        if !output_str.contains("wasm32-unknown-unknown") {
            println!("⚠️ wasm32-unknown-unknown target not found, installing...");
            let install_output = Command::new("rustup")
                .args([
                    "target",
                    "add",
                    "wasm32-unknown-unknown"
                ])
                .output()?;
            
            if install_output.status.success() {
                println!("✅ wasm32-unknown-unknown target installed successfully");
            } else {
                println!("❌ Failed to install wasm32-unknown-unknown target");
                println!("{}", String::from_utf8_lossy(&install_output.stderr));
            }
        } else {
            println!("✅ wasm32-unknown-unknown target is already installed");
        }
        
        // Show next steps
        let next_steps_template_path = template_dir_path.join("template.json");
        if next_steps_template_path.exists() {
            let next_steps_json = fs::read_to_string(&next_steps_template_path)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&next_steps_json) {
                if let Some(next_steps) = json.get("next_steps").and_then(|ns| ns.as_array()) {
                    println!("\nNext steps:");
                    for step in next_steps {
                        if let Some(step_str) = step.as_str() {
                            // Apply template substitution to next steps
                            let mut handlebars = Handlebars::new();
                            handlebars.register_escape_fn(handlebars::no_escape);
                            let template_vars = json!({
                                "project_name": name
                            });
                            
                            let rendered = handlebars.render_template(step_str, &template_vars)
                                .unwrap_or_else(|_| step_str.to_string());
                                
                            println!("  {}", rendered);
                        }
                    }
                }
            }
        }
        
        println!("\n🎉 Project {} created successfully!", name);
        
        return Ok(());
    } else if template == "embedded" {
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
    } else if template == "edge" {
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
        
        // Check for wasm-pack
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
        
        template_manager::apply_template(&template, app_path, &name, additional_vars.clone())?;
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
