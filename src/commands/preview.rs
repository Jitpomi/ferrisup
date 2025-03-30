use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Select};
use std::collections::HashMap;

use crate::template_manager::{get_template, list_templates};
use crate::config::{Config, Components, Client, Server, Database, AI, Edge, Embedded};

/// Execute the preview command to visualize a template without actually creating files
pub fn execute(template_name: Option<&str>) -> Result<()> {
    println!("{}", "FerrisUp Template Preview".bold().green());
    println!("Preview template structure without creating files\n");
    
    // Get template name interactively if not provided
    let selected_template = if let Some(name) = template_name {
        name.to_string()
    } else {
        // List available templates for selection
        let templates = list_templates()?;
        
        // Create formatted display items with template name and description
        let display_items: Vec<String> = templates
            .iter()
            .map(|(name, desc)| format!("{} - {}", name, desc))
            .collect();
        
        let selection = Select::new()
            .with_prompt("Select a template to preview")
            .items(&display_items)
            .default(0)
            .interact()?;
        
        // Extract just the name from the selected template tuple
        templates[selection].0.clone()
    };
    
    // Get template metadata
    let _template_content = get_template(&selected_template)
        .context(format!("Failed to find template '{}'", selected_template))?;
    
    // Create a temporary representation of the project structure
    println!("\n{} {}", "Template:".bold(), selected_template.green());
    
    // Create a virtual configuration for the preview
    let config = create_preview_config(&selected_template)?;
    
    // Generate the project structure tree
    let tree = generate_project_tree(&config);
    
    // Display the project structure tree
    println!("\n{}", "Project Structure:".bold());
    println!("{}", tree);
    
    // Display notable components and features
    println!("\n{}", "Notable Features:".bold());
    display_template_features(&selected_template, &config);
    
    // Show file previews
    println!("\n{}", "Sample Files:".bold());
    display_sample_files(&selected_template);
    
    // Ask if user wants to create a project with this template
    if Confirm::new()
        .with_prompt("Create a new project with this template?")
        .default(false)
        .interact()?
    {
        // Call the new command with the selected template
        crate::commands::new::execute(None, Some(&selected_template), false, false, false)?;
    }
    
    Ok(())
}

/// Create a preview configuration based on template name
fn create_preview_config(template_name: &str) -> Result<Config> {
    let mut config = Config {
        project_name: "example_project".to_string(),
        template: template_name.to_string(),
        components: Components::default(),
    };
    
    // Set up different components based on template type
    match template_name {
        "full-stack" => {
            config.components.client = Some(Client {
                apps: vec!["web".to_string(), "desktop".to_string()],
                frameworks: vec!["dioxus".to_string(), "dioxus".to_string()],
            });
            
            config.components.server = Some(Server {
                services: vec!["api".to_string(), "auth".to_string()],
                frameworks: vec!["poem".to_string(), "poem".to_string()],
            });
            
            config.components.database = Some(Database {
                enabled: true,
                engines: vec!["postgresql".to_string()],
                migration_tool: "diesel".to_string(),
                cache_engine: Some("redis".to_string()),
                vector_engine: None,
                graph_engine: None,
            });
        },
        "gen-ai" => {
            config.components.client = Some(Client {
                apps: vec!["web".to_string()],
                frameworks: vec!["dioxus".to_string()],
            });
            
            config.components.server = Some(Server {
                services: vec!["inference".to_string(), "api".to_string()],
                frameworks: vec!["axum".to_string(), "axum".to_string()],
            });
            
            config.components.ai = Some(AI {
                models: vec!["llama".to_string(), "whisper".to_string()],
                frameworks: vec!["candle".to_string(), "tract".to_string()],
            });
        },
        "edge-app" => {
            config.components.client = Some(Client {
                apps: vec!["web".to_string()],
                frameworks: vec!["leptos".to_string()],
            });
            
            config.components.edge = Some(Edge {
                apps: vec!["worker".to_string()],
                platforms: vec!["cloudflare".to_string(), "deno".to_string()],
            });
            
            config.components.database = Some(Database {
                enabled: true,
                engines: vec!["dynamodb".to_string()],
                migration_tool: "aws-sdk".to_string(),
                cache_engine: None,
                vector_engine: None,
                graph_engine: None,
            });
        },
        "embedded" | "iot-device" => {
            config.components.embedded = Some(Embedded {
                devices: vec!["rp2040".to_string()],
                platforms: vec!["raspberry-pi-pico".to_string()],
            });
        },
        "serverless" => {
            config.components.server = Some(Server {
                services: vec!["function".to_string()],
                frameworks: vec!["lambda".to_string()],
            });
            
            config.components.database = Some(Database {
                enabled: true,
                engines: vec!["dynamodb".to_string()],
                migration_tool: "aws-sdk".to_string(),
                cache_engine: None,
                vector_engine: None,
                graph_engine: None,
            });
        },
        "ml-pipeline" => {
            config.components.server = Some(Server {
                services: vec!["pipeline".to_string(), "api".to_string()],
                frameworks: vec!["axum".to_string(), "axum".to_string()],
            });
            
            config.components.ai = Some(AI {
                models: vec!["custom".to_string()],
                frameworks: vec!["tract".to_string()],
            });
            
            config.components.database = Some(Database {
                enabled: true,
                engines: vec!["postgresql".to_string()],
                migration_tool: "sqlx".to_string(),
                cache_engine: None,
                vector_engine: None,
                graph_engine: None,
            });
        },
        "data-science" => {
            config.components.server = Some(Server {
                services: vec!["api".to_string()],
                frameworks: vec!["rocket".to_string()],
            });
            
            config.components.ai = Some(AI {
                models: vec!["notebook".to_string()],
                frameworks: vec!["polars".to_string()],
            });
            
            config.components.database = Some(Database {
                enabled: true,
                engines: vec!["postgresql".to_string(), "duckdb".to_string()],
                migration_tool: "sqlx".to_string(),
                cache_engine: None,
                vector_engine: None,
                graph_engine: None,
            });
        },
        _ => {
            // Minimal or library templates don't need special configuration
        }
    }
    
    Ok(config)
}

/// Generate a text-based tree representation of the project structure
fn generate_project_tree(config: &Config) -> String {
    let mut tree = format!("{}/\n", config.project_name);
    tree.push_str("├── Cargo.toml\n");
    
    if config.template == "minimal" {
        tree.push_str("├── src/\n");
        tree.push_str("│   └── main.rs\n");
    } else if config.template == "library" {
        tree.push_str("├── src/\n");
        tree.push_str("│   └── lib.rs\n");
    } else if config.template == "full-stack" || config.template == "gen-ai" || config.template == "edge-app" {
        // Client
        if let Some(client) = &config.components.client {
            tree.push_str("├── client/\n");
            if client.apps.contains(&"web".to_string()) {
                tree.push_str("│   ├── web/\n");
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── main.rs\n");
            }
            if client.apps.contains(&"desktop".to_string()) {
                tree.push_str("│   ├── desktop/\n");
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── main.rs\n");
            }
        }
        
        // Server
        if let Some(server) = &config.components.server {
            tree.push_str("├── server/\n");
            tree.push_str("│   ├── api/\n");
            tree.push_str("│   │   ├── Cargo.toml\n");
            tree.push_str("│   │   └── src/\n");
            tree.push_str("│   │       └── main.rs\n");
            if server.services.contains(&"auth".to_string()) {
                tree.push_str("│   ├── auth/\n");
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── main.rs\n");
            }
        }
        
        // Database
        if let Some(_database) = &config.components.database {
            tree.push_str("├── database/\n");
            tree.push_str("│   ├── Cargo.toml\n");
            tree.push_str("│   ├── migrations/\n");
            tree.push_str("│   └── src/\n");
            tree.push_str("│       ├── lib.rs\n");
            tree.push_str("│       ├── models.rs\n");
            tree.push_str("│       └── schema.rs\n");
        }
        
        // AI components
        if let Some(ai) = &config.components.ai {
            tree.push_str("├── ai/\n");
            tree.push_str("│   ├── models/\n");
            for model in &ai.models {
                tree.push_str(&format!("│   │   ├── {}/\n", model));
            }
            tree.push_str("│   ├── inference/\n");
            tree.push_str("│   │   ├── Cargo.toml\n");
            tree.push_str("│   │   └── src/\n");
            tree.push_str("│   │       └── main.rs\n");
        }
        
        // Shared libraries
        tree.push_str("└── shared/\n");
        tree.push_str("    ├── core/\n");
        tree.push_str("    │   ├── Cargo.toml\n");
        tree.push_str("    │   └── src/\n");
        tree.push_str("    │       └── lib.rs\n");
        tree.push_str("    ├── models/\n");
        tree.push_str("    │   ├── Cargo.toml\n");
        tree.push_str("    │   └── src/\n");
        tree.push_str("    │       └── lib.rs\n");
        tree.push_str("    └── auth/\n");
        tree.push_str("        ├── Cargo.toml\n");
        tree.push_str("        └── src/\n");
        tree.push_str("            └── lib.rs\n");
    }
    
    tree
}

/// Display notable features for a given template
fn display_template_features(template_name: &str, config: &Config) {
    let features = match template_name {
        "minimal" => vec![
            "Single binary application",
            "Clean workspace structure",
            "Ready for expansion",
        ],
        "library" => vec![
            "Library crate with lib.rs",
            "Documentation setup",
            "Test infrastructure",
        ],
        "full-stack" => vec![
            "Client applications using Dioxus/Tauri",
            "Server services with Poem/Axum",
            "Shared libraries for code reuse",
            "Database integration",
            "Workspace structure for efficient development",
        ],
        "gen-ai" => vec![
            "AI model integration (LLaMA, BERT, Whisper, Stable Diffusion)",
            "Inference server architecture",
            "Model management and serving",
            "Web UI for interaction",
        ],
        "edge-app" => vec![
            "WebAssembly compilation targets",
            "Edge deployment configurations", 
            "Cloudflare Workers integration",
            "Deno Deploy support",
        ],
        "embedded" => vec![
            "No-std Rust configuration",
            "Hardware abstraction layers",
            "Memory-safe peripheral management",
            "Support for RP2040, ESP32, STM32, Arduino",
        ],
        "iot-device" => vec![
            "Connectivity protocols (MQTT, HTTP, BLE)",
            "Secure boot and OTA updates",
            "Power management utilities",
            "Server integration for device management",
        ],
        "serverless" => vec![
            "AWS Lambda/Azure Functions compatible",
            "Deployment configurations",
            "Local development environment",
            "Database connectors for serverless",
        ],
        "ml-pipeline" => vec![
            "Data ingestion components",
            "Transformation pipeline",
            "Model training infrastructure",
            "Inference API endpoints",
        ],
        "data-science" => vec![
            "Data loading and processing utilities",
            "Statistical analysis components",
            "Visualization and reporting tools",
            "Integration with popular data science libraries",
        ],
        _ => vec![
            "Basic Rust project structure",
            "Command-line interface",
        ],
    };
    
    for feature in features {
        println!("  ✓ {}", feature);
    }
    
    // Show tech stack based on config components
    println!("\n{}", "Technology Stack:".bold());
    
    if let Some(client) = &config.components.client {
        println!("  ✓ Client Framework: {}", client.frameworks.join(", "));
    }
    
    if let Some(server) = &config.components.server {
        println!("  ✓ Server Framework: {}", server.frameworks.join(", "));
    }
    
    if let Some(db) = &config.components.database {
        if let Some(primary_db) = db.engines.first() {
            println!("  ✓ Database: {} with {}", primary_db, db.migration_tool);
        }
    }
    
    if let Some(ai) = &config.components.ai {
        println!("  ✓ AI Models: {}", ai.models.join(", "));
        println!("  ✓ AI Frameworks: {}", ai.frameworks.join(", "));
    }
    
    if let Some(edge) = &config.components.edge {
        println!("  ✓ Edge Platforms: {}", edge.platforms.join(", "));
        println!("  ✓ Edge Apps: {}", edge.apps.join(", "));
    }
    
    if let Some(embedded) = &config.components.embedded {
        println!("  ✓ Devices: {}", embedded.devices.join(", "));
        println!("  ✓ Embedded Platforms: {}", embedded.platforms.join(", "));
    }
}

/// Display sample files from the template
fn display_sample_files(template_name: &str) {
    let sample_files = match template_name {
        "minimal" => {
            let mut files = HashMap::new();
            files.insert("src/main.rs", r#"fn main() {
    println!("Hello from FerrisUp minimal project!");
}"#);
            files
        },
        "library" => {
            let mut files = HashMap::new();
            files.insert("src/lib.rs", r#"//! Library crate
/// Example function
pub fn hello() -> &'static str {
    "Hello from FerrisUp library!"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        assert_eq!(hello(), "Hello from FerrisUp library!");
    }
}"#);
            files
        },
        "full-stack" => {
            let mut files = HashMap::new();
            files.insert("client/web/src/main.rs", r#"use dioxus::prelude::*;

fn main() {
    dioxus::web::launch(App);
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div { "Hello from Dioxus!" }
    })
}"#);
            
            files.insert("server/api/src/main.rs", r#"use poem::{get, handler, Route, Server};
use poem::web::{Html, Path};

#[handler]
fn hello(Path(name): Path<String>) -> Html<String> {
    Html(format!("<h1>Hello, {}!</h1>", name))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("/hello/:name", get(hello));
    
    println!("Server starting at http://localhost:3000");
    Server::new(([0, 0, 0, 0], 3000))
        .run(app)
        .await
}"#);
            files
        },
        _ => {
            let mut files = HashMap::new();
            files.insert("README.md", r#"# Custom Project

A Rust project created with FerrisUp.

## Features

- Modern Rust architecture
- Workspace organization for maintainability
- Comprehensive test suite
- Documentation

## Getting Started

```bash
cargo build
cargo test
cargo run
```"#);
            files
        }
    };
    
    for (path, content) in sample_files {
        println!("\n{} {}", "File:".cyan().bold(), path.cyan());
        println!("{}", "----------------------------------------".dimmed());
        println!("{}", content);
        println!("{}", "----------------------------------------".dimmed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_preview_config() {
        // Test minimal template
        let config = create_preview_config("minimal").expect("Should create minimal preview config");
        assert_eq!(config.project_name, "example_project");
        assert_eq!(config.template, "minimal");
        assert!(config.components.client.is_none());
        assert!(config.components.server.is_none());
        
        // Test full-stack template
        let config = create_preview_config("full-stack").expect("Should create full-stack preview config");
        assert_eq!(config.project_name, "example_project");
        assert_eq!(config.template, "full-stack");
        assert!(config.components.client.is_some());
        assert!(config.components.server.is_some());
        assert!(config.components.database.is_some());
        
        if let Some(client) = config.components.client {
            assert!(client.apps.contains(&"web".to_string()));
            assert!(client.frameworks.contains(&"dioxus".to_string()));
        }
        
        // Test gen-ai template
        let config = create_preview_config("gen-ai").expect("Should create gen-ai preview config");
        assert_eq!(config.template, "gen-ai");
        assert!(config.components.ai.is_some());
        
        if let Some(ai) = config.components.ai {
            assert!(ai.models.contains(&"llama".to_string()) || ai.models.contains(&"whisper".to_string()));
        }
    }
    
    #[test]
    fn test_generate_project_tree() {
        // Test minimal template tree generation
        let config = create_preview_config("minimal").expect("Should create minimal preview config");
        let tree = generate_project_tree(&config);
        
        assert!(tree.contains("example_project/"));
        assert!(tree.contains("Cargo.toml"));
        assert!(tree.contains("src/"));
        assert!(tree.contains("main.rs"));
        
        // Test library template tree generation
        let config = create_preview_config("library").expect("Should create library preview config");
        let tree = generate_project_tree(&config);
        
        assert!(tree.contains("example_project/"));
        assert!(tree.contains("Cargo.toml"));
        assert!(tree.contains("src/"));
        assert!(tree.contains("lib.rs"));
        
        // Test full-stack template tree generation
        let config = create_preview_config("full-stack").expect("Should create full-stack preview config");
        let tree = generate_project_tree(&config);
        
        // Print the tree for debugging
        println!("Full-stack tree structure:\n{}", tree);
        
        assert!(tree.contains("client/"));
        assert!(tree.contains("server/"));
        
        // Look for database-related content in the tree
        if let Some(db) = &config.components.database {
            println!("Database config: enabled={}, engines={:?}, migration_tool={}", db.enabled, db.engines, db.migration_tool);
            
            // Re-enable the database check now that we've fixed the tree generator
            assert!(
                tree.contains("database/"),
                "Tree should contain database directory"
            );
            
            // Check for database schema/models
            assert!(
                tree.contains("migrations/") || 
                tree.contains("models.rs") || 
                tree.contains("schema.rs"),
                "Tree should contain database schema components"
            );
        }
    }
    
    #[test]
    fn test_display_template_features() {
        // This is mainly a visual function, so we're just testing it doesn't crash
        let config = create_preview_config("minimal").expect("Should create minimal preview config");
        display_template_features("minimal", &config);
        
        let config = create_preview_config("full-stack").expect("Should create full-stack preview config");
        display_template_features("full-stack", &config);
        
        let config = create_preview_config("gen-ai").expect("Should create gen-ai preview config");
        display_template_features("gen-ai", &config);
        
        // No assertions needed - we're just making sure it executes without panicking
    }
    
    #[test]
    fn test_display_sample_files() {
        // Similar to the previous test, we're just ensuring it runs without errors
        display_sample_files("minimal");
        display_sample_files("library");
        display_sample_files("full-stack");
        
        // No assertions needed - we're just making sure it executes without panicking
    }
}
