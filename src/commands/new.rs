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
    
    // For client template, prompt for Rust client framework selection
    if selected_template == "client" && !no_interactive {
        let rust_client_framework_options = get_rust_client_framework_options();
        
        let selection = Select::new()
            .with_prompt("Select Rust client framework")
            .items(&rust_client_framework_options)
            .default(0)
            .interact()?;
        
        let framework = match selection {
            0 => "dioxus",
            1 => "yew",
            2 => "leptos",
            3 => "tauri",
            4 => "sycamoreui",
            5 => "moon-zoon",
            6 => "percy",
            7 => "seed",
            _ => "dioxus", // Default to dioxus
        };
        
        // Set up a single app with the selected framework
        let mut apps = Vec::new();
        apps.push("app1".to_string());
        
        let mut frameworks = Vec::new();
        frameworks.push(framework.to_string());
        
        // If Tauri is selected, provide Tauri configuration options
        if framework == "tauri" {
            // First, prompt for frontend language
            let frontend_languages = vec![
                "TypeScript/JavaScript",
                "Rust (Experimental)"
            ];
            
            let language_selection = Select::new()
                .with_prompt("Choose which language to use for your Tauri frontend")
                .items(&frontend_languages)
                .default(0)
                .interact()?;
            
            let frontend_language = match language_selection {
                0 => "js",
                1 => "rust",
                _ => "js",
            };
            
            frameworks.push(format!("frontend-{}", frontend_language));
            
            // For JavaScript frontend, prompt for package manager and UI framework
            if frontend_language == "js" {
                // Prompt for package manager
                let package_managers = vec![
                    "pnpm",
                    "yarn",
                    "npm",
                    "bun"
                ];
                
                let package_manager_selection = Select::new()
                    .with_prompt("Choose your package manager")
                    .items(&package_managers)
                    .default(0)
                    .interact()?;
                
                let package_manager = package_managers[package_manager_selection];
                frameworks.push(format!("package-manager-{}", package_manager));
                
                // Prompt for UI framework
                let js_framework_options = vec![
                    "React (JavaScript/TypeScript)",
                    "Vue (JavaScript/TypeScript)",
                    "Svelte (JavaScript/TypeScript)",
                    "Preact (JavaScript/TypeScript)",
                    "Solid (JavaScript/TypeScript)",
                    "Qwik (JavaScript/TypeScript)",
                    "Angular (JavaScript/TypeScript)",
                    "Vanilla (JavaScript/TypeScript)",
                    "None"
                ];
                
                let js_selection = Select::new()
                    .with_prompt("Choose your UI template")
                    .items(&js_framework_options)
                    .default(0)
                    .interact()?;
                
                let js_framework = match js_selection {
                    0 => "react",
                    1 => "vue",
                    2 => "svelte",
                    3 => "preact",
                    4 => "solid",
                    5 => "qwik",
                    6 => "angular",
                    7 => "vanilla",
                    _ => "vanilla",
                };
                
                frameworks.push(js_framework.to_string());
                
                // Finally, prompt for TypeScript or JavaScript
                let ui_flavors = vec![
                    "TypeScript",
                    "JavaScript"
                ];
                
                let flavor_selection = Select::new()
                    .with_prompt("Choose your UI flavor")
                    .items(&ui_flavors)
                    .default(0)
                    .interact()?;
                
                let ui_flavor = match flavor_selection {
                    0 => "typescript",
                    _ => "javascript",
                };
                
                frameworks.push(ui_flavor.to_string());
            } else {
                // For Rust frontend, prompt for Rust UI framework
                let rust_ui_frameworks = vec![
                    "Vanilla",
                    "Yew",
                    "Leptos",
                    "Sycamore"
                ];
                
                let rust_ui_selection = Select::new()
                    .with_prompt("Choose your Rust UI framework")
                    .items(&rust_ui_frameworks)
                    .default(0)
                    .interact()?;
                
                let rust_ui_framework = match rust_ui_selection {
                    1 => "yew",
                    2 => "leptos",
                    3 => "sycamore",
                    _ => "vanilla",
                };
                
                frameworks.push(rust_ui_framework.to_string());
            }
            
            // Add desktop/mobile capabilities
            frameworks.push("desktop".to_string());
            
            // Set the template to "tauri" instead of "client"
            project_config.template = "tauri".to_string();
        }
        
        project_config.components.client = Some(config::Client {
            apps,
            frameworks,
        });
    }
    
    // Only show JavaScript framework options for Tauri projects that are directly selected
    // (not when selected via client template)
    if selected_template == "tauri" && !no_interactive && project_config.components.client.is_none() {
        let framework_options = get_client_framework_options(no_interactive);
        
        let selections: Vec<usize> = MultiSelect::new()
            .with_prompt("Select client frameworks to use")
            .items(&framework_options)
            .interact()?;
        
        let mut frameworks = Vec::new();
        for selection in selections {
            match selection {
                0 => frameworks.push("react".to_string()),
                1 => frameworks.push("vue".to_string()),
                2 => frameworks.push("svelte".to_string()),
                3 => frameworks.push("preact".to_string()),
                4 => frameworks.push("solid".to_string()),
                5 => frameworks.push("qwik".to_string()),
                6 => frameworks.push("angular".to_string()),
                7 => frameworks.push("vanilla".to_string()),
                8 => frameworks.push("none".to_string()),
                _ => (),
            }
        }
        
        if !frameworks.is_empty() {
            project_config.components.client = Some(config::Client {
                apps: vec!["app".to_string()], // Default app
                frameworks,
            });
        }
    }
    
    // Ask if user wants to customize components
    let customize_components = if no_interactive {
        false
    } else if selected_template == "client" {
        // For client template, we've already handled framework selection
        // No need to ask about database support for client templates
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
                let rust_client_framework_options = get_rust_client_framework_options();
                let selections: Vec<usize> = if no_interactive {
                    vec![0]
                } else {
                    MultiSelect::new()
                        .with_prompt("Select Rust client frameworks to use")
                        .items(&rust_client_framework_options)
                        .interact()?
                };
                
                let mut frameworks = Vec::new();
                for selection in selections {
                    match selection {
                        0 => frameworks.push("dioxus".to_string()),
                        1 => frameworks.push("yew".to_string()),
                        2 => frameworks.push("leptos".to_string()),
                        3 => frameworks.push("tauri".to_string()),
                        4 => frameworks.push("sycamoreui".to_string()),
                        5 => frameworks.push("moon-zoon".to_string()),
                        6 => frameworks.push("percy".to_string()),
                        7 => frameworks.push("seed".to_string()),
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
    if let Some(_client) = &project_config.components.client {
        if !project_config.components.client.as_ref().unwrap().apps.is_empty() {
            println!("\n2. Run client applications:");
            
            for app in &project_config.components.client.as_ref().unwrap().apps {
                let framework = if let Some(framework) = project_config.components.client.as_ref().unwrap().frameworks.first() {
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
    
    if let Some(_server) = &project_config.components.server {
        if !project_config.components.server.as_ref().unwrap().services.is_empty() && project_config.template != "client" && project_config.template != "tauri" {
            println!("\n3. Run server services:");
            
            for service in &project_config.components.server.as_ref().unwrap().services {
                println!("   # For {}:", service);
                println!("   cargo run --bin {}", service);
                println!("   # Service will be available at http://localhost:3000");
            }
        }
    }
    
    println!("\nFor more detailed instructions, see the README.md file in your project directory.");
    
    Ok(())
}

fn get_client_framework_options(_no_interactive: bool) -> Vec<&'static str> {
    // Return a list of available JavaScript client frameworks for Tauri
    vec![
        "React (JavaScript/TypeScript)",
        "Vue (JavaScript/TypeScript)",
        "Svelte (JavaScript/TypeScript)",
        "Preact (JavaScript/TypeScript)",
        "Solid (JavaScript/TypeScript)",
        "Qwik (JavaScript/TypeScript)",
        "Angular (JavaScript/TypeScript)",
        "Vanilla (JavaScript/TypeScript)",
        "None"
    ]
}

fn get_rust_client_framework_options() -> Vec<&'static str> {
    // Return a list of available Rust client frameworks
    vec![
        "Dioxus",
        "Yew",
        "Leptos",
        "Tauri",
        "SycamoreUI",
        "MoonZoon",
        "Percy",
        "Seed",
        "None"
    ]
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
            if let Some(_client) = &config.components.client {
                setup_client(project_path, config, workspace_members)?;
            }
            
            // Setup server
            if let Some(_server) = &config.components.server {
                setup_server(project_path, config, workspace_members)?;
            }
            
            // Setup shared libs
            if let Some(_libs) = &config.components.libs {
                setup_libs(project_path, config, workspace_members)?;
            }
        },
        "client" | "tauri" => {
            // For client templates, only setup client components
            if let Some(_client) = &config.components.client {
                setup_client(project_path, config, workspace_members)?;
            }
        },
        "gen-ai" => {
            // Setup AI components
            if let Some(_ai) = &config.components.ai {
                setup_ai(project_path, config, workspace_members)?;
            }
            
            // Setup shared libs
            if let Some(_libs) = &config.components.libs {
                setup_libs(project_path, config, workspace_members)?;
            }
        },
        "edge-app" => {
            // Setup edge components
            if let Some(_edge) = &config.components.edge {
                setup_edge(project_path, config, workspace_members)?;
            }
        },
        "embedded" => {
            // Setup embedded components
            if let Some(_embedded) = &config.components.embedded {
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
    if let Some(_client) = &config.components.client {
        let client_dir = project_path.join("client");
        create_directory(&client_dir)?;
        
        for app in &config.components.client.as_ref().unwrap().apps {
            let app_path = client_dir.join(app);
            create_directory(&app_path)?;
            create_directory(&app_path.join("src"))?;
            
            // Create app Cargo.toml
            let framework = if let Some(framework) = config.components.client.as_ref().unwrap().frameworks.first() {
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
            
            // Handle different frameworks
            if framework == "dioxus" {
                // Print debug information
                println!("📦 Using Dioxus CLI to bootstrap the project");
                
                // First, ensure the Dioxus CLI is installed
                println!("🔧 Checking if Dioxus CLI is installed...");
                let dx_check = std::process::Command::new("dx")
                    .arg("--version")
                    .output();
                
                let dx_installed = match dx_check {
                    Ok(output) => output.status.success(),
                    Err(_) => false,
                };
                
                if !dx_installed {
                    println!("⚠️ Dioxus CLI not found. Installing it...");
                    let install_status = std::process::Command::new("cargo")
                        .args(["install", "dioxus-cli", "--locked"])
                        .status();
                        
                    if let Err(e) = install_status {
                        println!("⚠️ Failed to install Dioxus CLI: {}", e);
                        println!("⚠️ Falling back to manual project creation");
                        // Fall back to manual creation
                        create_manual_dioxus_project(&app_path)?;
                    } else {
                        // Remove the app directory to allow dx new to create it
                        std::fs::remove_dir_all(&app_path).ok();
                        
                        // Change to the client directory and run dx new
                        println!("🔧 Creating new Dioxus project: {}", app);
                        let mut command = std::process::Command::new("dx");
                        command.args(["new", app])
                            .arg("--name").arg(app)
                            .current_dir(&client_dir);
                        
                        // Print the full command for debugging
                        println!("🔧 Executing: {:?}", command);
                        
                        // Execute the command
                        let status = command.status();
                        
                        match status {
                            Ok(exit_status) if exit_status.success() => {
                                println!("✅ Successfully created Dioxus app");
                                
                                // Ensure WASM target is installed
                                println!("🔧 Ensuring WASM target is installed...");
                                let _ = std::process::Command::new("rustup")
                                    .args(["target", "add", "wasm32-unknown-unknown"])
                                    .status();
                            },
                            _ => {
                                println!("⚠️ Failed to create Dioxus project using CLI. Falling back to manual creation.");
                                // Fall back to manual creation
                                create_manual_dioxus_project(&app_path)?;
                            }
                        }
                    }
                } else {
                    // Remove the app directory to allow dx new to create it
                    std::fs::remove_dir_all(&app_path).ok();
                    
                    // Change to the client directory and run dx new
                    println!("🔧 Creating new Dioxus project: {}", app);
                    let mut command = std::process::Command::new("dx");
                    command.args(["new", app])
                        .arg("--name").arg(app)
                        .current_dir(&client_dir);
                    
                    // Print the full command for debugging
                    println!("🔧 Executing: {:?}", command);
                    
                    // Execute the command
                    let status = command.status();
                    
                    match status {
                        Ok(exit_status) if exit_status.success() => {
                            println!("✅ Successfully created Dioxus app");
                            
                            // Ensure WASM target is installed
                            println!("🔧 Ensuring WASM target is installed...");
                            let _ = std::process::Command::new("rustup")
                                .args(["target", "add", "wasm32-unknown-unknown"])
                                .status();
                        },
                        _ => {
                            println!("⚠️ Failed to create Dioxus project using CLI. Falling back to manual creation.");
                            // Fall back to manual creation
                            create_manual_dioxus_project(&app_path)?;
                        }
                    }
                }
                
                // Create main.rs with Dioxus template if it doesn't exist
                let src_main_rs = app_path.join("src").join("main.rs");
                if !src_main_rs.exists() {
                    std::fs::write(src_main_rs, include_str!("../../templates/client/dioxus/main.rs"))?;
                }
                
                // Create Dioxus configuration file if it doesn't exist
                let dioxus_config_path = app_path.join("Dioxus.toml");
                if !dioxus_config_path.exists() {
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
                    std::fs::write(dioxus_config_path, dioxus_config)?;
                }
            } else if framework == "tauri" {
                // Get frontend language, package manager, and UI framework from config
                let frontend_language = config.components.client.as_ref().unwrap().frameworks.iter()
                    .find(|f| f.starts_with("frontend-"))
                    .map(|s| s.replace("frontend-", ""))
                    .unwrap_or_else(|| "js".to_string());
                
                // For JS/TS frontend, get package manager and UI framework
                if frontend_language == "js" {
                    let package_manager = config.components.client.as_ref().unwrap().frameworks.iter()
                        .find(|f| f.starts_with("package-manager-"))
                        .map(|s| s.replace("package-manager-", ""))
                        .unwrap_or_else(|| "npm".to_string());
                    
                    let js_framework = config.components.client.as_ref().unwrap().frameworks.iter()
                        .find(|f| ["react", "vue", "svelte", "preact", "solid", "qwik", "angular", "vanilla"].contains(&f.as_str()))
                        .map(|s| s.as_str())
                        .unwrap_or("vanilla");
                    
                    let is_typescript = config.components.client.as_ref().unwrap().frameworks.iter().any(|f| f == "typescript");
                    let template = format!("{}{}", js_framework, if is_typescript { "-ts" } else { "" });
                    
                    // Remove the app directory first to allow create-tauri-app to create it
                    std::fs::remove_dir_all(&app_path).ok();
                    
                    // Print debug information
                    println!("📦 Using create-tauri-app CLI to bootstrap the project");
                    println!("   Package Manager: {}", package_manager);
                    println!("   Template: {}", template);
                    
                    // Construct the command based on the package manager
                    let mut command = std::process::Command::new(&package_manager);
                    
                    // Add appropriate arguments based on package manager
                    match package_manager.as_str() {
                        "npm" => {
                            command.args(["exec", "--", "create-tauri-app@latest"]);
                        },
                        "yarn" => {
                            command.args(["dlx", "create-tauri-app@latest"]);
                        },
                        "pnpm" => {
                            command.args(["dlx", "create-tauri-app@latest"]);
                        },
                        "bun" => {
                            command.args(["x", "create-tauri-app@latest"]);
                        },
                        _ => {
                            command.args(["exec", "--", "create-tauri-app@latest"]);
                        }
                    };
                    
                    // Add common arguments
                    command.arg(app)
                        .arg("--template")
                        .arg(&template)
                        .arg("--manager")
                        .arg(&package_manager)
                        .arg("--yes") // Non-interactive mode
                        .current_dir(&client_dir);
                    
                    // Print the full command for debugging
                    println!("🔧 Executing: {:?}", command);
                    
                    // Execute the command
                    let status = command.status();
                    
                    match status {
                        Ok(exit_status) if exit_status.success() => {
                            println!("✅ Successfully created Tauri app with {} using {}", template, package_manager);
                        },
                        Ok(_) => {
                            eprintln!("❌ Failed to create Tauri app with create-tauri-app CLI");
                            
                            // Fallback to manual creation if CLI fails
                            create_directory(&app_path)?;
                            create_directory(&app_path.join("src"))?;
                            create_manual_tauri_project(&app_path, config.components.client.as_ref().unwrap(), app)?;
                        },
                        Err(e) => {
                            eprintln!("❌ Failed to execute create-tauri-app CLI: {}", e);
                            
                            // Fallback to manual creation if CLI fails
                            create_directory(&app_path)?;
                            create_directory(&app_path.join("src"))?;
                            create_manual_tauri_project(&app_path, config.components.client.as_ref().unwrap(), app)?;
                        }
                    }
                } else if frontend_language == "rust" {
                    // For Rust frontend, get the Rust UI framework
                    let rust_ui_framework = config.components.client.as_ref().unwrap().frameworks.iter()
                        .find(|f| ["yew", "leptos", "sycamore", "dioxus"].contains(&f.as_str()))
                        .map(|s| s.as_str())
                        .unwrap_or("yew"); // Default to yew if no specific framework is selected
                    
                    // Check for required dependencies
                    println!("🔧 Checking for required Tauri Rust frontend dependencies...");
                    
                    // Check for Tauri CLI
                    let tauri_cli_installed = Command::new("cargo")
                        .args(["install", "--list"])
                        .output()
                        .map(|output| String::from_utf8_lossy(&output.stdout).contains("tauri-cli"))
                        .unwrap_or(false);
                    
                    if !tauri_cli_installed {
                        println!("📦 Installing Tauri CLI...");
                        let install_result = Command::new("cargo")
                            .args(["install", "tauri-cli", "--version", "^2.0.0", "--locked"])
                            .status();
                        
                        match install_result {
                            Ok(status) if status.success() => println!("✅ Tauri CLI installed successfully"),
                            _ => println!("⚠️ Failed to install Tauri CLI. You'll need to install it manually: cargo install tauri-cli --version '^2.0.0' --locked"),
                        }
                    } else {
                        println!("✅ Tauri CLI is already installed");
                    }
                    
                    // Check for Trunk
                    let trunk_installed = Command::new("cargo")
                        .args(["install", "--list"])
                        .output()
                        .map(|output| String::from_utf8_lossy(&output.stdout).contains("trunk"))
                        .unwrap_or(false);
                    
                    if !trunk_installed {
                        println!("📦 Installing Trunk...");
                        let install_result = Command::new("cargo")
                            .args(["install", "trunk", "--locked"])
                            .status();
                        
                        match install_result {
                            Ok(status) if status.success() => println!("✅ Trunk installed successfully"),
                            _ => println!("⚠️ Failed to install Trunk. You'll need to install it manually: cargo install trunk --locked"),
                        }
                    } else {
                        println!("✅ Trunk is already installed");
                    }
                    
                    // Check for wasm32 target
                    let wasm_target_installed = Command::new("rustup")
                        .args(["target", "list", "--installed"])
                        .output()
                        .map(|output| String::from_utf8_lossy(&output.stdout).contains("wasm32-unknown-unknown"))
                        .unwrap_or(false);
                    
                    if !wasm_target_installed {
                        println!("📦 Adding wasm32-unknown-unknown target...");
                        let add_target_result = Command::new("rustup")
                            .args(["target", "add", "wasm32-unknown-unknown"])
                            .status();
                        
                        match add_target_result {
                            Ok(status) if status.success() => println!("✅ wasm32-unknown-unknown target added successfully"),
                            _ => println!("⚠️ Failed to add wasm32-unknown-unknown target. You'll need to add it manually: rustup target add wasm32-unknown-unknown"),
                        }
                    } else {
                        println!("✅ wasm32-unknown-unknown target is already installed");
                    }
                    
                    // Remove the app directory first to allow create-tauri-app to create it
                    std::fs::remove_dir_all(&app_path).ok();
                    
                    // Print debug information
                    println!("📦 Using create-tauri-app CLI to bootstrap the project with Rust frontend");
                    println!("   Rust UI Framework: {}", rust_ui_framework);
                    
                    // Construct the command based on a default package manager (npm)
                    let mut command = std::process::Command::new("npm");
                    
                    // Add appropriate arguments
                    command.args(["exec", "--", "create-tauri-app@latest"]);
                    
                    // Add common arguments
                    command.arg(app)
                        .arg("--template")
                        .arg(rust_ui_framework) // Use the specific Rust framework as template
                        .arg("--yes") // Non-interactive mode
                        .current_dir(&client_dir);
                    
                    // Print the full command for debugging
                    println!("🔧 Executing: {:?}", command);
                    
                    // Execute the command
                    let status = command.status();
                    
                    match status {
                        Ok(exit_status) if exit_status.success() => {
                            println!("✅ Successfully created Tauri app with Rust frontend");
                            
                            // If a specific Rust UI framework was selected, we need to modify the generated project
                            if rust_ui_framework != "yew" {
                                // TODO: Modify the generated project to use the selected Rust UI framework
                                println!("⚠️ Support for {} with Tauri is experimental. You may need to manually configure it.", rust_ui_framework);
                            }
                        },
                        Ok(_) => {
                            eprintln!("❌ Failed to create Tauri app with create-tauri-app CLI");
                            
                            // Fallback to manual creation if CLI fails
                            create_directory(&app_path)?;
                            create_directory(&app_path.join("src"))?;
                            create_manual_tauri_project(&app_path, config.components.client.as_ref().unwrap(), app)?;
                        },
                        Err(e) => {
                            eprintln!("❌ Failed to execute create-tauri-app CLI: {}", e);
                            
                            // Fallback to manual creation if CLI fails
                            create_directory(&app_path)?;
                            create_directory(&app_path.join("src"))?;
                            create_manual_tauri_project(&app_path, config.components.client.as_ref().unwrap(), app)?;
                        }
                    }
                }
                
                // The src-tauri directory should be added to workspace members if it exists
                let src_tauri_dir = app_path.join("src-tauri");
                if src_tauri_dir.exists() {
                    workspace_members.push(format!("client/{}/src-tauri", app));
                }
            } else {
                // For other frameworks, create a basic main.rs
                let main_rs = "fn main() {\n    println!(\"Hello from client!\");\n}";
                std::fs::write(app_path.join("src").join("main.rs"), main_rs)?;
            }
            
            workspace_members.push(format!("client/{}", app));
        }
    }
    
    Ok(())
}

// Helper function to create a manual Dioxus project when the CLI approach fails
fn create_manual_dioxus_project(app_path: &Path) -> Result<()> {
    println!("⚠️ Falling back to manual Dioxus project creation");
    
    // Create src directory
    let src_dir = app_path.join("src");
    create_directory(&src_dir)?;
    
    // Create main.rs
    std::fs::write(
        src_dir.join("main.rs"),
        r#"use dioxus::prelude::*;

fn main() {
    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            h1 { "Welcome to Dioxus!" }
            p { "Your Dioxus app is ready." }
        }
    })
}
"#,
    )?;
    
    // Create Dioxus configuration file
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
    
    Ok(())
}

fn setup_server(project_path: &Path, config: &crate::config::Config, workspace_members: &mut Vec<String>) -> Result<()> {
    if let Some(_server) = &config.components.server {
        let server_path = project_path.join("server");
        create_directory(&server_path)?;
        
        for service in &config.components.server.as_ref().unwrap().services {
            let service_path = server_path.join(service);
            create_directory(&service_path)?;
            
            // Create service directories
            create_directory(&service_path.join("src"))?;
            
            // Create service Cargo.toml
            let framework = if let Some(framework) = config.components.server.as_ref().unwrap().frameworks.first() {
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
    if let Some(_libs) = &config.components.libs {
        let libs_path = project_path.join("libs");
        create_directory(&libs_path)?;
        
        for lib in &config.components.libs.as_ref().unwrap().modules {
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
    if let Some(_ai) = &config.components.ai {
        let ai_path = project_path.join("ai");
        create_directory(&ai_path)?;
        
        for model in &config.components.ai.as_ref().unwrap().models {
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
    if let Some(_edge) = &config.components.edge {
        let edge_path = project_path.join("edge");
        create_directory(&edge_path)?;
        
        for app in &config.components.edge.as_ref().unwrap().apps {
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
    if let Some(_embedded) = &config.components.embedded {
        let embedded_path = project_path.join("embedded");
        create_directory(&embedded_path)?;
        
        for device in &config.components.embedded.as_ref().unwrap().devices {
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
    let project_path = Path::new(project_name);
    
    // Determine if this is a Tauri project
    let is_tauri = if let Some(_client) = &config.components.client {
        config.components.client.as_ref().unwrap().frameworks.iter().any(|f| f == "tauri")
    } else {
        false
    };
    
    // For Tauri projects, create a specialized README
    if is_tauri {
        let readme_content = format!(
            r#"# {project_name}

A Tauri desktop application.

## Project Structure

```
{project_name}/
├── client/           # Frontend application
│   └── app/          # Tauri application
│       ├── src/      # Frontend source code
│       └── src-tauri/ # Tauri backend code
├── Cargo.toml        # Rust workspace configuration
└── README.md         # This file
```

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable)
- [Node.js](https://nodejs.org/) (LTS version recommended)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Running the Application

Navigate to the client app directory:

```bash
cd client/app
```

Then run the development server:

```bash
cargo tauri dev
```

### Building for Production

To build the application for production:

```bash
cargo tauri build
```

This will create optimized binaries in the `target/release` directory.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
"#
        );
        
        std::fs::write(project_path.join("README.md"), readme_content)?;
        return Ok(());
    }
    
    // For other projects, create a standard README with appropriate sections
    let mut readme_content = format!(
        r#"# {project_name}

## Project Structure

```
{project_name}/
"#
    );
    
    // Add client section if client components exist
    if let Some(_client) = &config.components.client {
        readme_content.push_str(
            r#"├── client/           # Client applications
"#
        );
        
        for app in &config.components.client.as_ref().unwrap().apps {
            readme_content.push_str(&format!(
                r#"│   ├── {}/          # Client application
"#,
                app
            ));
        }
    }
    
    // Add server section if server components exist and this is not a client-only template
    if config.template != "client" {
        if let Some(_server) = &config.components.server {
            readme_content.push_str(
                r#"├── server/           # Server applications
"#
            );
            
            for app in &config.components.server.as_ref().unwrap().services {
                readme_content.push_str(&format!(
                    r#"│   ├── {}/          # Server application
"#,
                    app
                ));
            }
        }
        
        // Add database section if database components exist
        if let Some(database) = &config.components.database {
            if database.enabled {
                readme_content.push_str(
                    r#"├── database/         # Database configuration
"#
                );
                
                for engine in &database.engines {
                    readme_content.push_str(&format!(
                        r#"│   ├── {}/          # {} database configuration
"#,
                        engine, engine
                    ));
                }
            }
        }
    }
    
    // Add common files
    readme_content.push_str(
        r#"├── Cargo.toml        # Rust workspace configuration
└── README.md         # This file
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable)
"#
    );
    
    // Add client-specific instructions
    if let Some(_client) = &config.components.client {
        let has_dioxus = config.components.client.as_ref().unwrap().frameworks.iter().any(|f| f == "dioxus");
        
        if has_dioxus {
            readme_content.push_str(
                r#"- [Dioxus CLI](https://dioxuslabs.com/docs/0.5/guide/en/getting_started/cli.html) (for Dioxus applications)

### Running Client Applications

For Dioxus applications:

```bash
# Navigate to the client app directory
cd client/app1

# Run the development server
dx serve
```
"#
            );
        }
    }
    
    // Add server-specific instructions if not a client-only template
    if config.template != "client" {
        if let Some(_server) = &config.components.server {
            readme_content.push_str(
                r#"
### Running Server Applications

```bash
# Navigate to the server app directory
cd server/app1

# Run the server
cargo run
```
"#
            );
        }
        
        // Add database-specific instructions
        if let Some(database) = &config.components.database {
            if database.enabled {
                readme_content.push_str(
                    r#"
### Database Setup

"#
                );
                
                for engine in &database.engines {
                    match engine.as_str() {
                        "postgres" => {
                            readme_content.push_str(
                                r#"#### PostgreSQL

1. Install PostgreSQL
2. Create a database
3. Update the `.env` file with your database credentials
"#
                            );
                        }
                        "mysql" => {
                            readme_content.push_str(
                                r#"#### MySQL

1. Install MySQL
2. Create a database
3. Update the `.env` file with your database credentials
"#
                            );
                        }
                        "sqlite" => {
                            readme_content.push_str(
                                r#"#### SQLite

The SQLite database file will be created automatically when the application runs.
"#
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    // Add common closing sections
    readme_content.push_str(
        r#"
## License

This project is licensed under the MIT License - see the LICENSE file for details.
"#
    );
    
    std::fs::write(project_path.join("README.md"), readme_content)?;
    
    Ok(())
}

fn create_manual_tauri_project(app_path: &Path, _client: &crate::config::Client, app: &str) -> Result<()> {
    println!("⚠️ Falling back to manual Tauri project creation");
    
    // Create src directory
    let src_dir = app_path.join("src");
    create_directory(&src_dir)?;
    
    // Create main.rs
    std::fs::write(
        src_dir.join("main.rs"),
        r#"use tauri::{Builder, TauriApp};
use tauri::WindowBuilder;

fn main() {
    let app = Builder::default()
        .setup(|_app| Ok(()))
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    
    let window = WindowBuilder::new(&app, "main")
        .title("Tauri App")
        .build()
        .unwrap();
    
    app.run(|_app_handle, event| match event {
        tauri::Event::WindowEvent { event, .. } => match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                println!("Close requested");
            }
            _ => {}
        },
        _ => {}
    });
}
"#,
    )?;
    
    // Create index.html
    let index_html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tauri App</title>
</head>
<body>
    <h1>Welcome to Tauri!</h1>
</body>
</html>
"#;
    std::fs::write(app_path.join("index.html"), index_html)?;
    
    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = "2.0"
serde = {{ version = "1.0", features = ["derive"] }}
"#,
        app
    );
    std::fs::write(app_path.join("Cargo.toml"), cargo_toml)?;
    
    Ok(())
}
