use anyhow::{Result, anyhow, Context};
use colored::Colorize;
use std::collections::HashMap;
use dialoguer::{Confirm, Select};
use crate::project::templates::{get_template, list_templates, find_template_directory};
use crate::core::Config;

/// Component structures for preview functionality
#[derive(Default, Debug)]
struct Components {
    client: Option<Client>,
    server: Option<Server>,
    database: Option<Database>,
    ai: Option<AI>,
    edge: Option<Edge>,
    embedded: Option<Embedded>,
}

#[derive(Default, Debug)]
struct Client {
    apps: Vec<String>,
    frameworks: Vec<String>,
}

#[derive(Default, Debug)]
struct Server {
    services: Vec<String>,
    frameworks: Vec<String>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct Database {
    enabled: bool,
    engines: Vec<String>,
    migration_tool: String,
    cache_engine: Option<String>,
    vector_engine: Option<String>,
    graph_engine: Option<String>,
}

#[derive(Default, Debug)]
struct AI {
    models: Vec<String>,
    frameworks: Vec<String>,
}

#[derive(Default, Debug)]
struct Edge {
    apps: Vec<String>,
    platforms: Vec<String>,
}

#[derive(Default, Debug)]
struct Embedded {
    devices: Vec<String>,
    platforms: Vec<String>,
}

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
    
    // Create a local components structure for preview
    let mut components = Components::default();
    
    // Set up different components based on template type
    match selected_template.as_str() {
        "full-stack" => {
            components.client = Some(Client {
                apps: vec!["web".to_string(), "desktop".to_string()],
                frameworks: vec!["dioxus".to_string(), "dioxus".to_string()],
            });
            
            components.server = Some(Server {
                services: vec!["api".to_string(), "auth".to_string()],
                frameworks: vec!["poem".to_string(), "poem".to_string()],
            });
            
            components.database = Some(Database {
                enabled: true,
                engines: vec!["postgresql".to_string()],
                migration_tool: "diesel".to_string(),
                cache_engine: Some("redis".to_string()),
                vector_engine: None,
                graph_engine: None,
            });
        },
        "gen-ai" => {
            components.client = Some(Client {
                apps: vec!["web".to_string()],
                frameworks: vec!["dioxus".to_string()],
            });
            
            components.server = Some(Server {
                services: vec!["inference".to_string(), "api".to_string()],
                frameworks: vec!["axum".to_string(), "axum".to_string()],
            });
            
            components.ai = Some(AI {
                models: vec!["llama".to_string(), "whisper".to_string()],
                frameworks: vec!["candle".to_string(), "tract".to_string()],
            });
        },
        "edge-app" => {
            components.client = Some(Client {
                apps: vec!["web".to_string()],
                frameworks: vec!["leptos".to_string()],
            });
            
            components.edge = Some(Edge {
                apps: vec!["worker".to_string()],
                platforms: vec!["cloudflare".to_string(), "deno".to_string()],
            });
            
            components.database = Some(Database {
                enabled: true,
                engines: vec!["dynamodb".to_string()],
                migration_tool: "aws-sdk".to_string(),
                cache_engine: None,
                vector_engine: None,
                graph_engine: None,
            });
        },
        "embedded" | "iot-device" => {
            components.embedded = Some(Embedded {
                devices: vec!["rp2040".to_string()],
                platforms: vec!["raspberry-pi-pico".to_string()],
            });
        },
        "serverless" => {
            components.server = Some(Server {
                services: vec!["function".to_string()],
                frameworks: vec!["lambda".to_string()],
            });
            
            components.database = Some(Database {
                enabled: true,
                engines: vec!["dynamodb".to_string()],
                migration_tool: "aws-sdk".to_string(),
                cache_engine: None,
                vector_engine: None,
                graph_engine: None,
            });
        },
        "ml-pipeline" => {
            components.server = Some(Server {
                services: vec!["pipeline".to_string(), "api".to_string()],
                frameworks: vec!["axum".to_string(), "axum".to_string()],
            });
            
            components.ai = Some(AI {
                models: vec!["custom".to_string()],
                frameworks: vec!["tract".to_string()],
            });
            
            components.database = Some(Database {
                enabled: true,
                engines: vec!["postgresql".to_string()],
                migration_tool: "sqlx".to_string(),
                cache_engine: None,
                vector_engine: None,
                graph_engine: None,
            });
        },
        "data-science" => {
            components.server = Some(Server {
                services: vec!["api".to_string()],
                frameworks: vec!["rocket".to_string()],
            });
            
            components.ai = Some(AI {
                models: vec!["notebook".to_string()],
                frameworks: vec!["polars".to_string()],
            });
            
            components.database = Some(Database {
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
    
    // Create a virtual configuration for the preview
    let config = Config::default();
    
    // Generate the project structure tree
    let tree = generate_project_tree(&components, &config);
    
    // Display the project structure tree
    println!("\n{}", "Project Structure:".bold());
    println!("{}", tree);
    
    // Display notable components and features
    println!("\n{}", "Notable Features:".bold());
    display_template_features(&selected_template, &components, &config);
    
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
        println!("Using template: {}", selected_template);
        
        // Create a temporary project with the selected template
        if let Err(e) = crate::commands::new::execute(None, Some(&selected_template), None, None, None, false, false, false, None) {
            return Err(anyhow!("Failed to create preview: {}", e));
        }
    }
    
    Ok(())
}

/// Generate a text-based tree representation of the project structure
fn generate_project_tree(components: &Components, _config: &Config) -> String {
    // Since Config no longer has project_name and template fields,
    // we'll use hardcoded values for the preview
    let project_name = "example_project";
    let template_type = "minimal"; // Default template type for preview
    
    let mut tree = format!("{}/\n", project_name);
    tree.push_str("├── Cargo.toml\n");
    
    // Different structure based on template type
    if template_type == "minimal" {
        tree.push_str("└── src/\n");
        tree.push_str("    └── main.rs\n");
    } else if template_type == "library" {
        tree.push_str("└── src/\n");
        tree.push_str("    └── lib.rs\n");
    } else if template_type == "full-stack" || template_type == "gen-ai" || template_type == "edge-app" {
        // Client
        if let Some(client) = &components.client {
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
        if let Some(server) = &components.server {
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
        if let Some(_database) = &components.database {
            tree.push_str("├── database/\n");
            tree.push_str("│   ├── Cargo.toml\n");
            tree.push_str("│   ├── migrations/\n");
            tree.push_str("│   └── src/\n");
            tree.push_str("│       ├── lib.rs\n");
            tree.push_str("│       ├── models.rs\n");
            tree.push_str("│       └── schema.rs\n");
        }
        
        // AI components
        if let Some(ai) = &components.ai {
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
        tree.push_str("└── ferrisup_common/\n");
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
fn display_template_features(template_name: &str, components: &Components, _config: &Config) {
    let mut features_from_metadata = Vec::new();
    
    if let Ok(template_dir) = find_template_directory(template_name) {
        let template_json_path = template_dir.join("template.json");
        if template_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&template_json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(features) = json.get("features").and_then(|f| f.as_array()) {
                        for feature in features {
                            if let Some(feature_str) = feature.as_str() {
                                features_from_metadata.push(feature_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    let features = if !features_from_metadata.is_empty() {
        features_from_metadata.iter().map(|s| s.as_str()).collect::<Vec<&str>>()
    } else {
        match template_name {
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
        }
    };
    
    for feature in features {
        println!("  ✓ {}", feature);
    }
    
    println!("\n{}", "Technology Stack:".bold());
    
    if let Some(client) = &components.client {
        println!("  ✓ Client Framework: {}", client.frameworks.join(", "));
    }
    
    if let Some(server) = &components.server {
        println!("  ✓ Server Framework: {}", server.frameworks.join(", "));
    }
    
    if let Some(db) = &components.database {
        if let Some(primary_db) = db.engines.first() {
            println!("  ✓ Database: {} with {}", primary_db, db.migration_tool);
        }
    }
    
    if let Some(ai) = &components.ai {
        println!("  ✓ AI Models: {}", ai.models.join(", "));
        println!("  ✓ AI Frameworks: {}", ai.frameworks.join(", "));
    }
    
    if let Some(edge) = &components.edge {
        println!("  ✓ Edge Platforms: {}", edge.platforms.join(", "));
        println!("  ✓ Edge Apps: {}", edge.apps.join(", "));
    }
    
    if let Some(embedded) = &components.embedded {
        println!("  ✓ Devices: {}", embedded.devices.join(", "));
        println!("  ✓ Embedded Platforms: {}", embedded.platforms.join(", "));
    }
}

/// Display sample files from the template
fn display_sample_files(template_name: &str) {
    println!("  📄 Sample files from template:");
    
    if let Ok(template_dir) = find_template_directory(template_name) {
        let key_files = vec![
            "src/main.rs",
            "src/lib.rs",
            "Cargo.toml",
            "README.md",
            "index.html",
            "style.css"
        ];
        
        let mut found_files = false;
        
        for file in key_files {
            let file_path = template_dir.join(file);
            if file_path.exists() && file_path.is_file() {
                found_files = true;
                
                println!("\n{} {}", "File:".cyan().bold(), file.cyan());
                println!("{}", "----------------------------------------".dimmed());
                
                match std::fs::read_to_string(&file_path) {
                    Ok(content) => {
                        let preview_content = if content.len() > 500 {
                            format!("{}...\n(Content truncated, showing first 500 characters)", &content[..500])
                        } else {
                            content
                        };
                        println!("{}", preview_content);
                    },
                    Err(_) => println!("(Unable to read file content)"),
                }
                
                println!("{}", "----------------------------------------".dimmed());
            }
        }
        
        if !found_files {
            println!("  (No sample files found in template)");
        }
    } else {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_preview_config() {
        let components = Components::default();
        let config = Config::default();
        let tree = generate_project_tree(&components, &config);
        
        assert!(tree.contains("example_project/"));
        assert!(tree.contains("Cargo.toml"));
        assert!(tree.contains("src/"));
        assert!(tree.contains("main.rs"));
    }
    
    #[test]
    fn test_display_template_features() {
        let components = Components::default();
        let config = Config::default();
        display_template_features("minimal", &components, &config);
        
        let components = Components::default();
        let config = Config::default();
        display_template_features("full-stack", &components, &config);
        
        let components = Components::default();
        let config = Config::default();
        display_template_features("gen-ai", &components, &config);
    }
    
    #[test]
    fn test_display_sample_files() {
        display_sample_files("minimal");
        display_sample_files("library");
        display_sample_files("full-stack");
    }
}
