use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::env;
use std::path::Path;
use std::process::Command;

use crate::config::{self, Config, read_config};
use crate::templates::{get_template, list_templates};
use crate::utils::{create_directory, write_cargo_toml, write_env_file};

pub fn execute(name: Option<&str>, template_name: Option<&str>, init_git: bool, build: bool, no_interactive: bool) -> Result<()> {
    println!("{}", "FerrisUp Interactive Project Creator".bold().green());
    println!("{}", "Create a new Rust project with the features you need".blue());
    
    // Interactive mode if name is not provided
    let project_name = match name {
        Some(n) => n.to_string(),
        None => {
            if no_interactive {
                // Use a default name in non-interactive mode
                "rust_project".to_string()
            } else {
                // Prompt for project name
                Input::new()
                    .with_prompt("Project name")
                    .interact_text()?
            }
        }
    };
    
    let project_path = Path::new(&project_name);
    
    // Check if directory already exists
    if project_path.exists() {
        println!("{} {} {}", 
            "Error:".red().bold(), 
            "Directory".red(), 
            format!("'{}' already exists", project_name).red());
        
        // Ask if user wants to use a different name (skip in non-interactive mode)
        if !no_interactive && Confirm::new()
            .with_prompt("Would you like to choose a different name?")
            .default(true)
            .interact()?
        {
            return execute(None, template_name, init_git, build, no_interactive);
        } else {
            return Ok(());
        }
    }
    
    // If template name is not provided, offer interactive selection or use default
    let selected_template = match template_name {
        Some(t) => t.to_string(),
        None => {
            if no_interactive {
                // Use "minimal" as the default template in non-interactive mode
                "minimal".to_string()
            } else {
                // Show available templates
                let templates = list_templates()?;
                let template_names: Vec<&str> = templates.iter().map(|(name, _)| name.as_str()).collect();
                let template_descriptions: Vec<&str> = templates.iter().map(|(_, desc)| desc.as_str()).collect();
                
                let selection = Select::new()
                    .with_prompt("Select a template")
                    .items(&template_names)
                    .interact()?;
                
                println!("Template description: {}", template_descriptions[selection].blue());
                
                template_names[selection].to_string()
            }
        }
    };
    
    // Get template configuration
    let template = get_template(&selected_template)
        .context(format!("Failed to find template '{}'", selected_template))?;
    
    println!("{} {}", "Using template:".blue(), template.cyan());
    
    // Create project directory
    create_directory(&project_name)?;
    
    // Create project structure based on template
    let config = read_config()?;
    let mut project_config = config.clone();
    project_config.project_name = project_name.to_string();
    project_config.template = selected_template.clone();
    
    // Ask if user wants to customize components
    let customize_components = if no_interactive {
        false
    } else {
        Confirm::new()
            .with_prompt("Would you like to customize project components?")
            .default(false)
            .interact()?
    };
    
    if customize_components {
        // Database selection
        let use_database = if no_interactive {
            false
        } else {
            Confirm::new()
                .with_prompt("Include database support?")
                .default(false)
                .interact()?
        };
        
        if use_database {
            let db_options = vec![
                "PostgreSQL",
                "MySQL",
                "SQLite",
                "MongoDB",
                "Redis",
                "DynamoDB",
                "None (will configure later)",
            ];
            
            let db_idx: usize = if no_interactive {
                0
            } else {
                Select::new()
                    .with_prompt("Select database type")
                    .items(&db_options)
                    .default(0)
                    .interact()?
            };
            
            let db_type = match db_idx {
                0 => "postgres",
                1 => "mysql",
                2 => "sqlite",
                3 => "mongodb",
                4 => "redis",
                5 => "dynamodb",
                _ => "none",
            };
            
            // Add database component if selected, but only for SQL databases for now
            if db_idx <= 5 {  // Any valid database selection (not "None")
                project_config.components.database = Some(config::Database {
                    enabled: true,
                    engines: vec![db_type.to_string()],
                    migration_tool: "sqlx".to_string(), // Default migration tool
                });
            }
        }
        
        // For full-stack or workspace templates, allow customization of client frameworks
        if selected_template == "full-stack" || selected_template == "minimal" {
            let customize_client = if no_interactive {
                false
            } else {
                Confirm::new()
                    .with_prompt("Would you like to customize client frameworks?")
                    .default(false)
                    .interact()?
            };
            
            if customize_client {
                let framework_options = vec![
                    "Dioxus (React-like, Web/Desktop/Mobile)",
                    "Tauri (Desktop with web technologies)",
                    "Leptos (Web with fine-grained reactivity)",
                    "Yew (Component-based framework)",
                    "Vanilla (No framework)",
                ];
                
                let selections: Vec<usize> = if no_interactive {
                    vec![0]
                } else {
                    MultiSelect::new()
                        .with_prompt("Select client frameworks to use")
                        .items(&framework_options)
                        .interact()?
                };
                
                let mut frameworks = Vec::new();
                for selection in selections {
                    match selection {
                        0 => frameworks.push("dioxus".to_string()),
                        1 => frameworks.push("tauri".to_string()),
                        2 => frameworks.push("leptos".to_string()),
                        3 => frameworks.push("yew".to_string()),
                        4 => frameworks.push("vanilla".to_string()),
                        _ => (),
                    }
                }
                
                if !frameworks.is_empty() {
                    project_config.components.client = Some(config::Client {
                        apps: vec!["app1".to_string(), "app2".to_string()], // Default apps
                        frameworks,
                    });
                }
            }
            
            // Server framework selection
            let customize_server = if no_interactive {
                false
            } else {
                Confirm::new()
                    .with_prompt("Would you like to customize server frameworks?")
                    .default(false)
                    .interact()?
            };
            
            if customize_server {
                let framework_options = vec![
                    "Poem (Simple and flexible)",
                    "Axum (Modular and performant)",
                    "Actix Web (Powerful and mature)",
                    "Rocket (Ergonomic and boilerplate-free)",
                    "Warp (Composable and fast)",
                ];
                
                let selections: Vec<usize> = if no_interactive {
                    vec![0]
                } else {
                    MultiSelect::new()
                        .with_prompt("Select server frameworks to use")
                        .items(&framework_options)
                        .interact()?
                };
                
                let mut frameworks = Vec::new();
                for selection in selections {
                    match selection {
                        0 => frameworks.push("poem".to_string()),
                        1 => frameworks.push("axum".to_string()),
                        2 => frameworks.push("actix-web".to_string()),
                        3 => frameworks.push("rocket".to_string()),
                        4 => frameworks.push("warp".to_string()),
                        _ => (),
                    }
                }
                
                if !frameworks.is_empty() {
                    project_config.components.server = Some(config::Server {
                        services: vec!["service1".to_string(), "service2".to_string()], // Default services
                        frameworks,
                    });
                }
            }
        }
        
        // For AI templates, allow customization
        if selected_template == "gen-ai" {
            let ai_options = vec![
                "Text Generation (LLaMA, GPT)",
                "Image Generation (Stable Diffusion)",
                "Speech Recognition (Whisper)",
                "Embeddings (BERT, Sentence transformers)",
                "Computer Vision (Object detection, classification)",
            ];
            
            let selections: Vec<usize> = if no_interactive {
                vec![0]
            } else {
                MultiSelect::new()
                    .with_prompt("Select AI capabilities to include")
                    .items(&ai_options)
                    .interact()?
            };
            
            let mut modules = Vec::new();
            for selection in selections {
                match selection {
                    0 => modules.push("text-generation".to_string()),
                    1 => modules.push("image-generation".to_string()),
                    2 => modules.push("speech-recognition".to_string()),
                    3 => modules.push("embeddings".to_string()),
                    4 => modules.push("computer-vision".to_string()),
                    _ => (),
                }
            }
            
            if !modules.is_empty() {
                project_config.components.ai = Some(config::AI {
                    models: vec!["llama".to_string(), "whisper".to_string()], // Default models
                    frameworks: modules, // Note: This matches the actual field name in the AI struct
                });
            }
        }
        
        // For edge-app templates, allow customization
        if selected_template == "edge-app" {
            let edge_options = vec![
                "WebAssembly (WASM)",
                "Cloudflare Workers",
                "Deno Deploy",
                "Netlify Functions",
                "Vercel Edge Functions",
            ];
            
            let selections: Vec<usize> = if no_interactive {
                vec![0]
            } else {
                MultiSelect::new()
                    .with_prompt("Select edge computing targets")
                    .items(&edge_options)
                    .interact()?
            };
            
            let mut platforms = Vec::new();
            for selection in selections {
                match selection {
                    0 => platforms.push("wasm".to_string()),
                    1 => platforms.push("cloudflare-workers".to_string()),
                    2 => platforms.push("deno-deploy".to_string()),
                    3 => platforms.push("netlify-functions".to_string()),
                    4 => platforms.push("vercel-edge".to_string()),
                    _ => (),
                }
            }
            
            if !platforms.is_empty() {
                project_config.components.edge = Some(config::Edge {
                    apps: vec!["edge-app".to_string()], // Default edge app
                    platforms,
                });
            }
        }
        
        // For embedded templates, allow customization
        if selected_template == "embedded" || selected_template == "iot-device" {
            let embedded_options = vec![
                "Raspberry Pi Pico (RP2040)",
                "ESP32",
                "STM32",
                "Arduino",
                "Generic Microcontroller",
            ];
            
            let selections: Vec<usize> = if no_interactive {
                vec![0]
            } else {
                MultiSelect::new()
                    .with_prompt("Select embedded targets")
                    .items(&embedded_options)
                    .interact()?
            };
            
            let mut platforms = Vec::new();
            for selection in selections {
                match selection {
                    0 => platforms.push("rp2040".to_string()),
                    1 => platforms.push("esp32".to_string()),
                    2 => platforms.push("stm32".to_string()),
                    3 => platforms.push("arduino".to_string()),
                    4 => platforms.push("generic".to_string()),
                    _ => (),
                }
            }
            
            if !platforms.is_empty() {
                project_config.components.embedded = Some(config::Embedded {
                    devices: vec!["device1".to_string()], // Default device
                    platforms,
                });
            }
        }
    }
    
    // Create the basic file structure
    create_project_structure(&project_name, &project_config, &template)?;
    
    // Ask about git initialization if not specified
    let should_init_git = if init_git {
        true
    } else {
        if no_interactive {
            false
        } else {
            Confirm::new()
                .with_prompt("Initialize git repository?")
                .default(true)
                .interact()?
        }
    };
    
    // Initialize git if requested
    if should_init_git {
        println!("{}", "Initializing git repository...".blue());
        Command::new("git")
            .arg("init")
            .current_dir(project_path)
            .output()
            .context("Failed to initialize git repository")?;
        
        // Create .gitignore
        let gitignore_path = project_path.join(".gitignore");
        let gitignore_content = r#"/target/
**/*.rs.bk
Cargo.lock
.env
.DS_Store
.idea/
.vscode/
*.pdb
*.exe
*.dll
*.so
*.dylib
*.iml
"#;
        std::fs::write(gitignore_path, gitignore_content)
            .context("Failed to create .gitignore file")?;
    }
    
    // Ask about building the project if not specified
    let should_build = if build {
        true
    } else {
        if no_interactive {
            false
        } else {
            Confirm::new()
                .with_prompt("Build the project?")
                .default(true)
                .interact()?
        }
    };
    
    // Build the project if requested
    if should_build {
        println!("{}", "Building project...".blue());
        Command::new("cargo")
            .arg("build")
            .current_dir(project_path)
            .output()
            .context("Failed to build project")?;
    }
    
    // Create a comprehensive README file
    create_readme(&project_name, &project_config)?;
    
    println!("\n{}", "Project successfully created!".bold().green());
    println!("Your new Rust project is ready at: {}", project_path.display().to_string().cyan());
    
    // Add instructions for running the project
    println!("\n{}", "Getting Started:".bold().yellow());
    println!("1. Navigate to your project directory:");
    println!("   cd {}", project_name);
    
    // Add instructions based on project components
    if let Some(client) = &project_config.components.client {
        if !client.apps.is_empty() {
            println!("\n2. Run client applications:");
            
            for (i, app) in client.apps.iter().enumerate() {
                let framework = if i < client.frameworks.len() {
                    &client.frameworks[i]
                } else if !client.frameworks.is_empty() {
                    // If we have at least one framework, use the first one
                    &client.frameworks[0]
                } else {
                    // Default to empty string if no frameworks are defined
                    ""
                };
                
                match framework {
                    "dioxus" => {
                        let port = 8080 + i;
                        println!("   # For {} (Dioxus web app):", app);
                        println!("   cargo install dioxus-cli  # Only needed once");
                        println!("   cd client/{} && dx serve --platform web --port {}", app, port);
                        println!("   # Then open http://localhost:{} in your browser", port);
                    },
                    "tauri" => {
                        println!("   # For {} (Tauri desktop app):", app);
                        println!("   cd client/{} && cargo run", app);
                    },
                    _ => {
                        println!("   # For {}:", app);
                        println!("   cargo run --bin {}", app);
                    }
                }
            }
        }
    }
    
    if let Some(server) = &project_config.components.server {
        if !server.services.is_empty() {
            println!("\n3. Run server services:");
            
            for service in &server.services {
                println!("   # For {}:", service);
                println!("   cargo run --bin {}", service);
                println!("   # Service will be available at http://localhost:3000");
            }
        }
    }
    
    println!("\nFor more detailed instructions, see the README.md file in your project directory.");
    
    Ok(())
}

fn create_project_structure(name: &str, config: &crate::config::Config, template: &str) -> Result<()> {
    let project_path = Path::new(name);
    
    // Create src directory
    let src_path = project_path.join("src");
    create_directory(src_path.to_str().unwrap())?;
    
    // Create Cargo.toml
    write_cargo_toml(project_path, config)?;
    
    // Create main.rs or lib.rs based on template
    match template {
        "minimal" | "binary" => {
            setup_minimal(project_path, config, &mut Vec::new())?;
        },
        "library" => {
            std::fs::write(
                src_path.join("lib.rs"), 
                include_str!("../../templates/library/lib.rs"),
            )?;
        },
        _ => {
            // For more complex templates, we need to create a workspace
            setup_workspace(name, config, template)?;
        }
    }
    
    // Create .env file if needed
    if template != "minimal" && template != "library" {
        write_env_file(project_path)?;
    }
    
    // Copy template README.md
    std::fs::write(
        project_path.join("README.md"),
        include_str!("../../README.md")
            .replace("FerrisUp", name)
            .replace("ferrisup", name.to_lowercase().as_str()),
    )?;
    
    Ok(())
}

fn setup_workspace(name: &str, config: &crate::config::Config, template: &str) -> Result<()> {
    let project_path = Path::new(name);
    
    // Create Cargo.toml with workspace configuration
    let mut workspace_members = Vec::new();
    
    // Create directories and add workspace members based on template
    setup_project(project_path, config, template, &mut workspace_members)?;
    
    // Update Cargo.toml with workspace members
    update_workspace_toml(project_path, &workspace_members)?;
    
    Ok(())
}

fn setup_project(
    project_path: &Path,
    config: &Config,
    template: &str,
    workspace_members: &mut Vec<String>
) -> Result<()> {
    // Create the basic file and directory structure based on template
    match template {
        "full-stack" => {
            // Setup client
            if let Some(_) = &config.components.client {
                setup_client(project_path, config, workspace_members)?;
            }
            
            // Setup server
            if let Some(_) = &config.components.server {
                setup_server(project_path, config, workspace_members)?;
            }
            
            // Setup shared libs
            if let Some(_) = &config.components.libs {
                setup_libs(project_path, config, workspace_members)?;
            }
        },
        "gen-ai" => {
            // Setup AI components
            if let Some(_) = &config.components.ai {
                setup_ai(project_path, config, workspace_members)?;
            }
            
            // Setup shared libs
            if let Some(_) = &config.components.libs {
                setup_libs(project_path, config, workspace_members)?;
            }
        },
        "edge-app" => {
            // Setup edge components
            if let Some(_) = &config.components.edge {
                setup_edge(project_path, config, workspace_members)?;
            }
        },
        "embedded" => {
            // Setup embedded components
            if let Some(_) = &config.components.embedded {
                setup_embedded(project_path, config, workspace_members)?;
            }
        },
        _ => {
            // For minimal or custom template, just setup workspace with a binary
            setup_minimal(project_path, config, workspace_members)?;
        }
    }
    
    Ok(())
}

fn setup_client(project_path: &Path, config: &crate::config::Config, workspace_members: &mut Vec<String>) -> Result<()> {
    if let Some(client) = &config.components.client {
        let client_path = project_path.join("client");
        create_directory(client_path.to_str().unwrap())?;
        
        for (i, app) in client.apps.iter().enumerate() {
            let app_path = client_path.join(app);
            create_directory(app_path.to_str().unwrap())?;
            
            // Create app directory structure
            std::fs::create_dir_all(app_path.join("src"))?;
            
            // Create app Cargo.toml
            // Fix for index out of bounds: safely get framework or use default
            let framework = if i < client.frameworks.len() {
                client.frameworks[i].as_str()
            } else if !client.frameworks.is_empty() {
                // If we have at least one framework, use the first one
                client.frameworks[0].as_str()
            } else {
                // Default to empty string if no frameworks are defined
                ""
            };
            
            let app_cargo = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
{}

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
                app,
                match framework {
                    "dioxus" => "dioxus = { version = \"0.5\" }\ndioxus-web = \"0.5\"\n\n# Optional: Uncomment to add the Dioxus CLI tools as a dev dependency\n# [dev-dependencies]\n# dioxus-cli = \"0.5\"",
                    "tauri" => "tauri = \"2.0\"\nserde = { version = \"1.0\", features = [\"derive\"] }",
                    _ => "",
                },
                app
            );
            
            std::fs::write(app_path.join("Cargo.toml"), app_cargo)?;
            
            // Create app README.md for Dioxus apps
            if framework == "dioxus" {
                let readme_content = r#"# Dioxus Web Application

This is a web application built with Dioxus, a React-like framework for Rust.

## Running the Application

To run this application in a browser, you'll need to install the Dioxus CLI:

```bash
cargo install dioxus-cli
```

Then you can run the development server:

```bash
# From this directory
dx serve

# Or from the workspace root
dx serve --path client/app1
```

## Building for Production

To build the application for production:

```bash
dx build --release
```

This will create optimized WebAssembly files in the `dist` directory.

## Running as a Native Binary

This application is designed to run in a web browser. If you try to run it directly with `cargo run`, you'll get a helpful message but it won't actually render the UI.

For the best development experience, use the Dioxus CLI as described above.
"#;
                std::fs::write(app_path.join("README.md"), readme_content)?;
            }
            
            // Create app main.rs based on framework
            let main_rs = match framework {
                "dioxus" => include_str!("../../templates/client/dioxus/main.rs"),
                "tauri" => include_str!("../../templates/client/tauri/main.rs"),
                _ => "fn main() {\n    println!(\"Hello from client!\");\n}",
            };
            
            std::fs::write(app_path.join("src").join("main.rs"), main_rs)?;
            
            // Create Dioxus configuration file
            if framework == "dioxus" {
                let dioxus_config = r#"[application]
name = "dioxus-app"
default_platform = "web"
out_dir = "dist"
asset_dir = "public"

[web.app]
title = "Dioxus App"

[web.watcher]
reload_html = true
watch_path = ["src", "public"]

[web.resource]
style = []
script = []

[web.resource.dev]
script = []
"#;
                std::fs::create_dir_all(app_path.join("public"))?;
                std::fs::write(app_path.join("Dioxus.toml"), dioxus_config)?;
            }
            
            workspace_members.push(format!("client/{}", app));
        }
    }
    
    Ok(())
}

fn setup_server(project_path: &Path, config: &crate::config::Config, workspace_members: &mut Vec<String>) -> Result<()> {
    if let Some(server) = &config.components.server {
        let server_path = project_path.join("server");
        create_directory(server_path.to_str().unwrap())?;
        
        for (i, service) in server.services.iter().enumerate() {
            let service_path = server_path.join(service);
            create_directory(service_path.to_str().unwrap())?;
            
            // Create service directories
            create_directory(service_path.join("src").to_str().unwrap())?;
            
            // Create service Cargo.toml
            // Fix for index out of bounds: safely get framework or use default
            let framework = if i < server.frameworks.len() {
                server.frameworks[i].as_str()
            } else if !server.frameworks.is_empty() {
                // If we have at least one framework, use the first one
                server.frameworks[0].as_str()
            } else {
                // Default to empty string if no frameworks are defined
                ""
            };
            
            let service_cargo = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
{}

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
                service,
                match framework {
                    "poem" => "poem = \"2.0\"\ntokio = { version = \"1.36\", features = [\"full\"] }\ntracing = \"0.1\"\ntracing-subscriber = \"0.3\"\nserde = { version = \"1.0\", features = [\"derive\"] }",
                    "axum" => "axum = \"0.7\"\ntokio = { version = \"1.36\", features = [\"full\"] }\ntracing = \"0.1\"\ntracing-subscriber = \"0.3\"\nserde = { version = \"1.0\", features = [\"derive\"] }",
                    _ => "",
                },
                service
            );
            
            std::fs::write(service_path.join("Cargo.toml"), service_cargo)?;
            
            // Create service main.rs based on framework
            let main_rs = match framework {
                "poem" => include_str!("../../templates/server/poem/main.rs"),
                "axum" => include_str!("../../templates/server/axum/main.rs"),
                "rocket" => "fn main() {
    println!(\"Hello from Rocket server!\");
    // TODO: Add Rocket server implementation
}",
                _ => "fn main() {\n    println!(\"Hello from server!\");\n}",
            };
            
            std::fs::write(service_path.join("src").join("main.rs"), main_rs)?;
            
            workspace_members.push(format!("server/{}", service));
        }
    }
    
    Ok(())
}

fn setup_libs(project_path: &Path, config: &crate::config::Config, workspace_members: &mut Vec<String>) -> Result<()> {
    if let Some(libs) = &config.components.libs {
        let libs_path = project_path.join("libs");
        create_directory(libs_path.to_str().unwrap())?;
        
        for lib in &libs.modules {
            let lib_path = libs_path.join(lib);
            create_directory(lib_path.to_str().unwrap())?;
            
            // Create lib directories
            create_directory(lib_path.join("src").to_str().unwrap())?;
            
            // Create lib Cargo.toml
            let lib_cargo = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
thiserror = "1.0"
"#,
                lib
            );
            
            std::fs::write(lib_path.join("Cargo.toml"), lib_cargo)?;
            
            // Create lib.rs
            std::fs::write(
                lib_path.join("src").join("lib.rs"),
                format!("pub fn hello_from_{}() {{\n    println!(\"Hello from {}!\");\n}}", lib, lib)
            )?;
            
            workspace_members.push(format!("libs/{}", lib));
        }
    }
    
    Ok(())
}

fn setup_ai(project_path: &Path, config: &crate::config::Config, workspace_members: &mut Vec<String>) -> Result<()> {
    if let Some(ai) = &config.components.ai {
        let ai_path = project_path.join("ai");
        create_directory(ai_path.to_str().unwrap())?;
        
        for model in &ai.models {
            let model_path = ai_path.join(model);
            create_directory(model_path.to_str().unwrap())?;
            
            // Create model directories
            create_directory(model_path.join("src").to_str().unwrap())?;
            
            // Create model Cargo.toml
            let model_cargo = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tch = "0.10"
tract-onnx = "0.19"
tokenizers = "0.13"
serde = {{ version = "1.0", features = ["derive"] }}
anyhow = "1.0"
"#,
                model
            );
            
            std::fs::write(model_path.join("Cargo.toml"), model_cargo)?;
            
            // Create lib.rs for the AI model
            std::fs::write(
                model_path.join("src").join("lib.rs"),
                include_str!("../../templates/ai/lib.rs")
                    .replace("MODEL_NAME", model)
            )?;
            
            workspace_members.push(format!("ai/{}", model));
        }
    }
    
    Ok(())
}

fn setup_edge(project_path: &Path, config: &crate::config::Config, workspace_members: &mut Vec<String>) -> Result<()> {
    if let Some(edge) = &config.components.edge {
        let edge_path = project_path.join("edge");
        create_directory(edge_path.to_str().unwrap())?;
        
        for app in &edge.apps {
            let app_path = edge_path.join(app);
            create_directory(app_path.to_str().unwrap())?;
            
            // Create app directories
            create_directory(app_path.join("src").to_str().unwrap())?;
            
            // Create app Cargo.toml
            let app_cargo = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = {{ version = "0.3", features = [
    "console", "Document", "Element", "HtmlElement", "Window"
] }}
wasm-bindgen-futures = "0.4"
serde = {{ version = "1.0", features = ["derive"] }}
serde-wasm-bindgen = "0.4"
"#,
                app
            );
            
            std::fs::write(app_path.join("Cargo.toml"), app_cargo)?;
            
            // Create lib.rs for the edge app
            std::fs::write(
                app_path.join("src").join("lib.rs"),
                include_str!("../../templates/edge/lib.rs")
                    .replace("APP_NAME", app)
            )?;
            
            workspace_members.push(format!("edge/{}", app));
        }
    }
    
    Ok(())
}

fn setup_embedded(project_path: &Path, config: &crate::config::Config, workspace_members: &mut Vec<String>) -> Result<()> {
    if let Some(embedded) = &config.components.embedded {
        let embedded_path = project_path.join("embedded");
        create_directory(embedded_path.to_str().unwrap())?;
        
        for device in &embedded.devices {
            let device_path = embedded_path.join(device);
            create_directory(device_path.to_str().unwrap())?;
            
            // Create device directories
            create_directory(device_path.join("src").to_str().unwrap())?;
            
            // Create device Cargo.toml
            let device_cargo = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
embedded-hal = "0.2"
panic-halt = "0.2"
cortex-m = "0.7"
cortex-m-rt = "0.7"

[[bin]]
name = "{}"
test = false
bench = false
"#,
                device, device
            );
            
            std::fs::write(device_path.join("Cargo.toml"), device_cargo)?;
            
            // Create main.rs for the embedded device
            std::fs::write(
                device_path.join("src").join("main.rs"),
                include_str!("../../templates/embedded/main.rs")
                    .replace("DEVICE_NAME", device)
            )?;
            
            workspace_members.push(format!("embedded/{}", device));
        }
    }
    
    Ok(())
}

fn setup_minimal(project_path: &Path, _config: &crate::config::Config, _workspace_members: &mut Vec<String>) -> Result<()> {
    // Create a basic project with src directory
    let src_path = project_path.join("src");
    create_directory(src_path.to_str().unwrap())?;
    
    // Create a simple main.rs
    let main_content = r#"fn main() {
    println!("Hello from FerrisUp minimal project!");
}
"#;
    std::fs::write(src_path.join("main.rs"), main_content)?;
    
    // Create a simple Cargo.toml
    let cargo_content = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_path.file_name().unwrap().to_str().unwrap());
    
    std::fs::write(project_path.join("Cargo.toml"), cargo_content)?;
    
    Ok(())
}

fn update_workspace_toml(project_path: &Path, workspace_members: &[String]) -> Result<()> {
    let mut cargo_toml = format!(
        r#"[workspace]
members = [
{}
]
resolver = "2"

[workspace.dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
thiserror = "1.0"
anyhow = "1.0"
"#,
        workspace_members.iter()
            .map(|m| format!("    \"{}\"", m))
            .collect::<Vec<_>>()
            .join(",\n")
    );
    
    if workspace_members.is_empty() {
        cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
"#,
            project_path.file_name().unwrap().to_str().unwrap()
        );
    }
    
    std::fs::write(project_path.join("Cargo.toml"), cargo_toml)?;
    
    Ok(())
}

fn create_readme(project_name: &str, config: &Config) -> Result<()> {
    let readme_path = Path::new(project_name).join("README.md");
    
    // Generate content based on project components
    let mut content = format!(
        r#"# {}

A Rust project created with [FerrisUp](https://github.com/Jitpomi/ferrisup).

## Project Structure

This is a full-stack Rust project with the following components:

"#,
        project_name
    );

    // Add client section if applicable
    if let Some(client) = &config.components.client {
        content.push_str("### Client Applications\n\n");
        
        for (i, app) in client.apps.iter().enumerate() {
            content.push_str(&format!("- `client/{}`", app));
            
            if let Some(i) = client.apps.iter().position(|a| a == app) {
                if i < client.frameworks.len() {
                    content.push_str(&format!(" ({})", client.frameworks[i]));
                }
            }
            content.push_str("\n");
        }
        content.push_str("\n");
    }

    // Add server section if applicable
    if let Some(server) = &config.components.server {
        content.push_str("### Server Services\n\n");
        
        for service in &server.services {
            content.push_str(&format!("- `server/{}`", service));
            
            if let Some(i) = server.services.iter().position(|s| s == service) {
                if i < server.frameworks.len() {
                    content.push_str(&format!(" ({})", server.frameworks[i]));
                }
            }
            content.push_str("\n");
        }
        content.push_str("\n");
    }

    // Add database section if applicable
    if let Some(database) = &config.components.database {
        content.push_str("### Database\n\n");
        if database.enabled {
            content.push_str(&format!("- Engines: {}\n", database.engines.join(", ")));
            if !database.migration_tool.is_empty() {
                content.push_str(&format!("- Migration Tool: {}\n", database.migration_tool));
            }
        } else {
            content.push_str("- Database is configured but not enabled\n");
        }
        content.push_str("\n");
    }

    // Add running instructions
    content.push_str(r#"## Running the Project

This project is set up as a Rust workspace with multiple binary targets. Here's how to run each component:

"#);

    // Add client running instructions
    if let Some(client) = &config.components.client {
        content.push_str("### Running Client Applications\n\n");
        
        for (i, app) in client.apps.iter().enumerate() {
            content.push_str(&format!("#### {}\n\n", app));
            
            let framework = if i < client.frameworks.len() {
                &client.frameworks[i]
            } else if !client.frameworks.is_empty() {
                // If we have at least one framework, use the first one
                &client.frameworks[0]
            } else {
                // Default to empty string if no frameworks are defined
                ""
            };
            
            match framework {
                "dioxus" => {
                    let port = 8080 + i;
                    content.push_str(&format!(r#"This is a Dioxus web application. To run it:

1. Install the Dioxus CLI if you haven't already:
   ```bash
   cargo install dioxus-cli
   ```

2. Run the development server:
   ```bash
   # From the project root
   cd client/{} && dx serve --platform web --port {}
   ```

3. Open your browser at http://localhost:{}
"#, app, port, port));
                },
                "tauri" => {
                    content.push_str(&format!(r#"This is a Tauri desktop application. To run it:

```bash
cd client/{} && cargo run
```
"#, app));
                },
                _ => {
                    content.push_str(&format!(r#"```bash
cargo run --bin {}
```
"#, app));
                }
            }
            content.push_str("\n");
        }
    }

    // Add server running instructions
    if let Some(server) = &config.components.server {
        content.push_str("### Running Server Services\n\n");
        
        for service in &server.services {
            content.push_str(&format!("#### {}\n\n", service));
            content.push_str(&format!(r#"```bash
cargo run --bin {}
```

The server will start and listen on http://localhost:3000 by default.

"#, service));
        }
    }

    // Add instructions for running multiple components
    content.push_str(r#"### Running Multiple Components

To run multiple components simultaneously, you can use separate terminal windows or use a tool like [Overmind](https://github.com/DarthSim/overmind) or [Foreman](https://github.com/ddollar/foreman).

#### Example Procfile for Foreman/Overmind

Create a `Procfile` in the project root with the following content:

```
"#);
    
    // Add server services to Procfile
    if let Some(server) = &config.components.server {
        for service in &server.services {
            content.push_str(&format!("{}: cd server/{} && cargo run\n", service, service));
        }
    }
    
    // Add client apps to Procfile
    if let Some(client) = &config.components.client {
        for (i, app) in client.apps.iter().enumerate() {
            let framework = if i < client.frameworks.len() {
                &client.frameworks[i]
            } else if !client.frameworks.is_empty() {
                // If we have at least one framework, use the first one
                &client.frameworks[0]
            } else {
                // Default to empty string if no frameworks are defined
                ""
            };
            
            if framework == "dioxus" {
                let port = 8080 + i;
                content.push_str(&format!("{}: cd client/{} && dx serve --platform web --port {}\n", 
                    app, 
                    app,
                    port
                ));
            } else {
                content.push_str(&format!("{}: cd client/{} && cargo run\n", app, app));
            }
        }
    }
    
    content.push_str("```\n\n");
    content.push_str("Then run:\n\n");
    content.push_str("```bash\n# If using Overmind\novermind start\n\n# If using Foreman\nforeman start\n```\n\n");
    
    content.push_str(r#"## Development

### Building All Components

To build all components at once:

```bash
cargo build
```

### Running Tests

To run tests for all components:

```bash
cargo test
```

## Deployment

Each component can be built for production using:

```bash
cargo build --release
```

For Dioxus web applications, use:

```bash
cd client/app1
dx build --release
```

This will generate optimized WebAssembly files in the `dist` directory.
"#);

    std::fs::write(readme_path, content)?;
    Ok(())
}
