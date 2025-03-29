use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Input, MultiSelect, Select};
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
    create_directory(project_path)?;
    
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
            // Primary database selection
            let primary_db_options = vec![
                "PostgreSQL",
                "MySQL",
                "SQLite",
                "MongoDB",
                "TypeDB",
                "CockroachDB",
                "TimescaleDB",
                "ScyllaDB",
                "None (will configure later)",
            ];
            
            let primary_db_idx: usize = if no_interactive {
                0
            } else {
                Select::new()
                    .with_prompt("Select primary database type")
                    .items(&primary_db_options)
                    .default(0)
                    .interact()?
            };
            
            let primary_db_type = match primary_db_idx {
                0 => "postgres",
                1 => "mysql",
                2 => "sqlite",
                3 => "mongodb",
                4 => "typedb",
                5 => "cockroachdb",
                6 => "timescaledb",
                7 => "scylladb",
                _ => "none",
            };
            
            // Cache database selection
            let cache_db_options = vec![
                "None",
                "Redis",
                "Memcached",
                "Hazelcast",
                "Aerospike",
                "Ignite",
            ];
            
            let cache_db_idx: usize = if no_interactive {
                0
            } else {
                Select::new()
                    .with_prompt("Select cache database")
                    .items(&cache_db_options)
                    .default(0)
                    .interact()?
            };
            
            let cache_db_type = match cache_db_idx {
                1 => "redis",
                2 => "memcached",
                3 => "hazelcast",
                4 => "aerospike",
                5 => "ignite",
                _ => "none",
            };
            
            // Vector database selection
            let vector_db_options = vec![
                "None",
                "Pinecone",
                "Qdrant",
                "Milvus",
                "Chroma",
                "Weaviate",
                "Vespa",
                "Faiss",
                "OpenSearch",
            ];
            
            let vector_db_idx: usize = if no_interactive {
                0
            } else {
                Select::new()
                    .with_prompt("Select vector database for LLM embeddings")
                    .items(&vector_db_options)
                    .default(0)
                    .interact()?
            };
            
            let vector_db_type = match vector_db_idx {
                1 => "pinecone",
                2 => "qdrant",
                3 => "milvus",
                4 => "chroma",
                5 => "weaviate",
                6 => "vespa",
                7 => "faiss",
                8 => "opensearch",
                _ => "none",
            };
            
            // Graph database selection
            let graph_db_options = vec![
                "None",
                "Neo4j",
                "TypeDB",
                "ArangoDB",
                "JanusGraph",
                "DGraph",
                "TigerGraph",
                "Amazon Neptune",
            ];
            
            let graph_db_idx: usize = if no_interactive {
                0
            } else {
                Select::new()
                    .with_prompt("Select graph database (optional)")
                    .items(&graph_db_options)
                    .default(0)
                    .interact()?
            };
            
            let graph_db_type = match graph_db_idx {
                1 => "neo4j",
                2 => "typedb",
                3 => "arangodb",
                4 => "janusgraph",
                5 => "dgraph",
                6 => "tigergraph",
                7 => "neptune",
                _ => "none",
            };
            
            // Add database component if any database was selected
            if primary_db_idx < 8 || cache_db_idx > 0 || vector_db_idx > 0 || graph_db_idx > 0 {
                let mut engines = Vec::new();
                if primary_db_idx < 8 {
                    engines.push(primary_db_type.to_string());
                }
                
                project_config.components.database = Some(config::Database {
                    enabled: true,
                    engines,
                    migration_tool: "sqlx".to_string(), // Default migration tool
                    cache_engine: if cache_db_idx > 0 { Some(cache_db_type.to_string()) } else { None },
                    vector_engine: if vector_db_idx > 0 { Some(vector_db_type.to_string()) } else { None },
                    graph_engine: if graph_db_idx > 0 { Some(graph_db_type.to_string()) } else { None },
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
                    // Create the same number of apps as frameworks to avoid index out of bounds errors
                    let mut apps = Vec::new();
                    for _ in frameworks.iter() {
                        apps.push("app".to_string());
                    }
                    
                    project_config.components.client = Some(config::Client {
                        apps,
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
                        services: vec!["service".to_string()], // Default services
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
                    devices: vec!["device".to_string()], // Default device
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
    } else if no_interactive {
        false
    } else {
        Confirm::new()
            .with_prompt("Initialize git repository?")
            .default(true)
            .interact()?
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
    } else if no_interactive {
        false
    } else {
        Confirm::new()
            .with_prompt("Build the project?")
            .default(true)
            .interact()?
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
            
            for app in &client.apps {
                let framework = if let Some(framework) = client.frameworks.first() {
                    framework
                } else {
                    // Default to empty string if no frameworks are defined
                    ""
                };
                
                match framework {
                    "dioxus" => {
                        let port = 8080;
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
    create_directory(&src_path)?;
    
    // Create Cargo.toml
    write_cargo_toml(project_path, config)?;
    
    // Create main.rs or lib.rs based on template
    match template {
        "minimal" | "binary" => {
            setup_minimal(project_path, config)?;
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
            if config.components.client.is_some() {
                setup_client(project_path, config, workspace_members)?;
            }
            
            // Setup server
            if config.components.server.is_some() {
                setup_server(project_path, config, workspace_members)?;
            }
            
            // Setup shared libs
            if config.components.libs.is_some() {
                setup_libs(project_path, config, workspace_members)?;
            }
        },
        "gen-ai" => {
            // Setup AI components
            if config.components.ai.is_some() {
                setup_ai(project_path, config, workspace_members)?;
            }
            
            // Setup shared libs
            if config.components.libs.is_some() {
                setup_libs(project_path, config, workspace_members)?;
            }
        },
        "edge-app" => {
            // Setup edge components
            if config.components.edge.is_some() {
                setup_edge(project_path, config, workspace_members)?;
            }
        },
        "embedded" => {
            // Setup embedded components
            if config.components.embedded.is_some() {
                setup_embedded(project_path, config, workspace_members)?;
            }
        },
        _ => {
            // For minimal or custom template, just setup workspace with a binary
            setup_minimal(project_path, config)?;
        }
    }
    
    Ok(())
}

fn setup_client(project_path: &Path, config: &crate::config::Config, workspace_members: &mut Vec<String>) -> Result<()> {
    if let Some(client) = &config.components.client {
        let client_path = project_path.join("client");
        create_directory(&client_path)?;
        
        for app in &client.apps {
            let app_path = client_path.join(app);
            create_directory(&app_path)?;
            
            // Create app directory structure
            create_directory(&app_path.join("src"))?;
            
            // Create app Cargo.toml
            let framework = if let Some(framework) = client.frameworks.first() {
                framework
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
        create_directory(&server_path)?;
        
        for service in &server.services {
            let service_path = server_path.join(service);
            create_directory(&service_path)?;
            
            // Create service directories
            create_directory(&service_path.join("src"))?;
            
            // Create service Cargo.toml
            let framework = if let Some(framework) = server.frameworks.first() {
                framework
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
        create_directory(&libs_path)?;
        
        for lib in &libs.modules {
            let lib_path = libs_path.join(lib);
            create_directory(&lib_path)?;
            
            // Create lib directories
            create_directory(&lib_path.join("src"))?;
            
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
        create_directory(&ai_path)?;
        
        for model in &ai.models {
            let model_path = ai_path.join(model);
            create_directory(&model_path)?;
            
            // Create model directories
            create_directory(&model_path.join("src"))?;
            
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
        create_directory(&edge_path)?;
        
        for app in &edge.apps {
            let app_path = edge_path.join(app);
            create_directory(&app_path)?;
            
            // Create app directories
            create_directory(&app_path.join("src"))?;
            
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
        create_directory(&embedded_path)?;
        
        for device in &embedded.devices {
            let device_path = embedded_path.join(device);
            create_directory(&device_path)?;
            
            // Create device directories
            create_directory(&device_path.join("src"))?;
            
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

fn setup_minimal(project_path: &Path, _config: &crate::config::Config) -> Result<()> {
    // Create a basic project with src directory
    let src_path = project_path.join("src");
    create_directory(&src_path)?;
    
    // Create a simple main.rs
    let main_rs = src_path.join("main.rs");
    let main_content = r#"fn main() {
    println!("Hello from FerrisUp!");
}
"#;
    std::fs::write(main_rs, main_content)?;
    
    // Create a basic Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    let toml_content = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rust_project"));
    
    std::fs::write(cargo_toml, toml_content)?;
    
    Ok(())
}

fn update_workspace_toml(project_path: &Path, workspace_members: &[String]) -> Result<()> {
    let mut cargo_toml = format!(
        r#"[workspace]
members = [
    ".",
{}
]

[workspace.dependencies]
anyhow = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
tokio = {{ version = "1.0", features = ["full"] }}
"#,
        workspace_members
            .iter()
            .map(|member| format!("    \"{}\",", member))
            .collect::<Vec<String>>()
            .join("\n")
    );

    // Add resolver for Rust 2021
    cargo_toml.push_str("resolver = \"2\"\n");

    std::fs::write(project_path.join("Cargo.toml"), cargo_toml)?;
    Ok(())
}

fn create_readme(project_name: &str, config: &Config) -> Result<()> {
    let mut content = String::new();
    
    content.push_str("# FerrisUp Workspace\n\n");
    content.push_str("This is a Rust workspace created with FerrisUp.\n\n");
    
    content.push_str("## Components\n\n");
    
    if let Some(client) = &config.components.client {
        content.push_str("### Client Applications\n\n");
        
        for app in &client.apps {
            content.push_str(&format!("- {}\n", app));
        }
        
        content.push('\n');
    }
    
    if let Some(server) = &config.components.server {
        content.push_str("### Server Applications\n\n");
        
        for service in &server.services {
            content.push_str(&format!("- {}\n", service));
        }
        
        content.push('\n');
    }
    
    if let Some(database) = &config.components.database {
        if database.enabled {
            content.push_str("### Database Engines\n\n");
            
            for engine in &database.engines {
                content.push_str(&format!("- {}\n", engine));
            }
            
            content.push('\n');
        }
    }
    
    // Write the README.md file
    std::fs::write(Path::new(project_name).join("README.md"), content)?;
    
    Ok(())
}
