use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Select};
use std::collections::HashMap;
use std::path::Path;

use crate::templates::{get_template, list_templates};
use crate::config::{Config, Components, ClientComponents, ServerComponents, DatabaseComponents};
use crate::utils::create_directory;

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
        
        let selection = Select::new()
            .with_prompt("Select a template to preview")
            .items(&templates)
            .default(0)
            .interact()?;
        
        templates[selection].clone()
    };
    
    // Get template metadata
    let template_content = get_template(&selected_template)
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
        crate::commands::new::execute(None, Some(&selected_template), false, false)?;
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
            config.components.client = Some(ClientComponents {
                framework: "dioxus".to_string(),
                apps: vec!["web".to_string(), "desktop".to_string()],
            });
            
            config.components.server = Some(ServerComponents {
                framework: "poem".to_string(),
                services: vec!["api".to_string(), "auth".to_string()],
            });
            
            config.components.database = Some(DatabaseComponents {
                db_type: "postgresql".to_string(),
                orm: "diesel".to_string(),
            });
        },
        "gen-ai" => {
            config.components.client = Some(ClientComponents {
                framework: "dioxus".to_string(), 
                apps: vec!["web".to_string()],
            });
            
            config.components.server = Some(ServerComponents {
                framework: "axum".to_string(),
                services: vec!["inference".to_string(), "api".to_string()],
            });
            
            config.components.ai = Some(crate::config::AIComponents {
                models: vec!["llama".to_string(), "whisper".to_string()],
                tasks: vec!["text-generation".to_string(), "speech-recognition".to_string()],
            });
        },
        "edge-app" => {
            config.components.client = Some(ClientComponents {
                framework: "leptos".to_string(),
                apps: vec!["web".to_string()],
            });
            
            config.components.edge = Some(crate::config::EdgeComponents {
                platforms: vec!["cloudflare-workers".to_string(), "deno-deploy".to_string()],
                features: vec!["wasm".to_string(), "serverless".to_string()],
            });
        },
        "embedded" | "iot-device" => {
            config.components.embedded = Some(crate::config::EmbeddedComponents {
                mcu: "rp2040".to_string(),
                platforms: vec!["raspberry-pi-pico".to_string()],
            });
        },
        "serverless" => {
            config.components.server = Some(ServerComponents {
                framework: "lambda".to_string(),
                services: vec!["function".to_string()],
            });
            
            config.components.database = Some(DatabaseComponents {
                db_type: "dynamodb".to_string(),
                orm: "aws-sdk".to_string(),
            });
        },
        "ml-pipeline" => {
            config.components.server = Some(ServerComponents {
                framework: "axum".to_string(),
                services: vec!["pipeline".to_string(), "api".to_string()],
            });
            
            config.components.ai = Some(crate::config::AIComponents {
                models: vec!["custom".to_string()],
                tasks: vec!["training".to_string(), "inference".to_string()],
            });
            
            config.components.database = Some(DatabaseComponents {
                db_type: "postgresql".to_string(),
                orm: "sqlx".to_string(),
            });
        },
        "data-science" => {
            config.components.server = Some(ServerComponents {
                framework: "rocket".to_string(),
                services: vec!["api".to_string()],
            });
            
            config.components.ai = Some(crate::config::AIComponents {
                models: vec!["custom".to_string()],
                tasks: vec!["analysis".to_string()],
            });
            
            config.components.database = Some(DatabaseComponents {
                db_type: "postgresql".to_string(),
                orm: "sqlx".to_string(),
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
    let mut tree = format!("example_project/\n");
    tree.push_str("├── Cargo.toml\n");
    
    match config.template.as_str() {
        "minimal" => {
            tree.push_str("└── src/\n");
            tree.push_str("    └── main.rs\n");
        },
        "library" => {
            tree.push_str("└── src/\n");
            tree.push_str("    └── lib.rs\n");
        },
        "full-stack" => {
            tree.push_str("├── client/\n");
            if let Some(client) = &config.components.client {
                for app in &client.apps {
                    tree.push_str(&format!("│   ├── {}/\n", app));
                    tree.push_str("│   │   ├── Cargo.toml\n");
                    tree.push_str("│   │   └── src/\n");
                    tree.push_str("│   │       └── main.rs\n");
                }
            }
            
            tree.push_str("├── server/\n");
            if let Some(server) = &config.components.server {
                for service in &server.services {
                    tree.push_str(&format!("│   ├── {}/\n", service));
                    tree.push_str("│   │   ├── Cargo.toml\n");
                    tree.push_str("│   │   └── src/\n");
                    tree.push_str("│   │       └── main.rs\n");
                }
            }
            
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
        },
        "gen-ai" => {
            tree.push_str("├── client/\n");
            tree.push_str("│   └── web/\n");
            tree.push_str("│       ├── Cargo.toml\n");
            tree.push_str("│       └── src/\n");
            tree.push_str("│           └── main.rs\n");
            
            tree.push_str("├── server/\n");
            tree.push_str("│   ├── inference/\n");
            tree.push_str("│   │   ├── Cargo.toml\n");
            tree.push_str("│   │   └── src/\n");
            tree.push_str("│   │       └── main.rs\n");
            tree.push_str("│   └── api/\n");
            tree.push_str("│       ├── Cargo.toml\n");
            tree.push_str("│       └── src/\n");
            tree.push_str("│           └── main.rs\n");
            
            tree.push_str("├── ai/\n");
            tree.push_str("│   ├── models/\n");
            tree.push_str("│   │   ├── Cargo.toml\n");
            tree.push_str("│   │   └── src/\n");
            tree.push_str("│   │       └── lib.rs\n");
            tree.push_str("│   └── inference/\n");
            tree.push_str("│       ├── Cargo.toml\n");
            tree.push_str("│       └── src/\n");
            tree.push_str("│           └── lib.rs\n");
            
            tree.push_str("└── shared/\n");
            tree.push_str("    ├── core/\n");
            tree.push_str("    │   ├── Cargo.toml\n");
            tree.push_str("    │   └── src/\n");
            tree.push_str("    │       └── lib.rs\n");
            tree.push_str("    └── models/\n");
            tree.push_str("        ├── Cargo.toml\n");
            tree.push_str("        └── src/\n");
            tree.push_str("            └── lib.rs\n");
        },
        "edge-app" => {
            tree.push_str("├── client/\n");
            tree.push_str("│   └── web/\n");
            tree.push_str("│       ├── Cargo.toml\n");
            tree.push_str("│       └── src/\n");
            tree.push_str("│           └── main.rs\n");
            
            tree.push_str("├── edge/\n");
            tree.push_str("│   ├── cloudflare/\n");
            tree.push_str("│   │   ├── Cargo.toml\n");
            tree.push_str("│   │   └── src/\n");
            tree.push_str("│   │       └── lib.rs\n");
            tree.push_str("│   └── deno/\n");
            tree.push_str("│       ├── Cargo.toml\n");
            tree.push_str("│       └── src/\n");
            tree.push_str("│           └── lib.rs\n");
            
            tree.push_str("└── shared/\n");
            tree.push_str("    └── core/\n");
            tree.push_str("        ├── Cargo.toml\n");
            tree.push_str("        └── src/\n");
            tree.push_str("            └── lib.rs\n");
        },
        "embedded" | "iot-device" => {
            tree.push_str("├── firmware/\n");
            tree.push_str("│   ├── Cargo.toml\n");
            tree.push_str("│   └── src/\n");
            tree.push_str("│       └── main.rs\n");
            
            tree.push_str("├── memory.x\n");
            tree.push_str("├── .cargo/\n");
            tree.push_str("│   └── config.toml\n");
            
            if config.template == "iot-device" {
                tree.push_str("├── server/\n");
                tree.push_str("│   └── api/\n");
                tree.push_str("│       ├── Cargo.toml\n");
                tree.push_str("│       └── src/\n");
                tree.push_str("│           └── main.rs\n");
                
                tree.push_str("└── shared/\n");
                tree.push_str("    └── protocol/\n");
                tree.push_str("        ├── Cargo.toml\n");
                tree.push_str("        └── src/\n");
                tree.push_str("            └── lib.rs\n");
            } else {
                tree.push_str("└── shared/\n");
                tree.push_str("    └── hal/\n");
                tree.push_str("        ├── Cargo.toml\n");
                tree.push_str("        └── src/\n");
                tree.push_str("            └── lib.rs\n");
            }
        },
        "serverless" => {
            tree.push_str("├── functions/\n");
            tree.push_str("│   ├── api/\n");
            tree.push_str("│   │   ├── Cargo.toml\n");
            tree.push_str("│   │   └── src/\n");
            tree.push_str("│   │       └── main.rs\n");
            tree.push_str("│   └── worker/\n");
            tree.push_str("│       ├── Cargo.toml\n");
            tree.push_str("│       └── src/\n");
            tree.push_str("│           └── main.rs\n");
            
            tree.push_str("├── shared/\n");
            tree.push_str("│   └── core/\n");
            tree.push_str("│       ├── Cargo.toml\n");
            tree.push_str("│       └── src/\n");
            tree.push_str("│           └── lib.rs\n");
            
            tree.push_str("└── deployment/\n");
            tree.push_str("    ├── template.yaml\n");
            tree.push_str("    └── scripts/\n");
            tree.push_str("        └── deploy.sh\n");
        },
        "ml-pipeline" | "data-science" => {
            tree.push_str("├── server/\n");
            tree.push_str("│   └── api/\n");
            tree.push_str("│       ├── Cargo.toml\n");
            tree.push_str("│       └── src/\n");
            tree.push_str("│           └── main.rs\n");
            
            if config.template == "ml-pipeline" {
                tree.push_str("├── pipeline/\n");
                tree.push_str("│   ├── ingest/\n");
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── lib.rs\n");
                tree.push_str("│   ├── transform/\n");
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── lib.rs\n");
                tree.push_str("│   └── train/\n");
                tree.push_str("│       ├── Cargo.toml\n");
                tree.push_str("│       └── src/\n");
                tree.push_str("│           └── lib.rs\n");
            } else {
                tree.push_str("├── analysis/\n");
                tree.push_str("│   ├── Cargo.toml\n");
                tree.push_str("│   └── src/\n");
                tree.push_str("│       └── lib.rs\n");
                
                tree.push_str("├── visualization/\n");
                tree.push_str("│   ├── Cargo.toml\n");
                tree.push_str("│   └── src/\n");
                tree.push_str("│       └── lib.rs\n");
            }
            
            tree.push_str("└── shared/\n");
            tree.push_str("    └── models/\n");
            tree.push_str("        ├── Cargo.toml\n");
            tree.push_str("        └── src/\n");
            tree.push_str("            └── lib.rs\n");
        },
        _ => {
            tree.push_str("└── src/\n");
            tree.push_str("    └── main.rs\n");
        }
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
        println!("  ✓ Client Framework: {}", client.framework);
    }
    
    if let Some(server) = &config.components.server {
        println!("  ✓ Server Framework: {}", server.framework);
    }
    
    if let Some(db) = &config.components.database {
        println!("  ✓ Database: {} with {}", db.db_type, db.orm);
    }
    
    if let Some(ai) = &config.components.ai {
        println!("  ✓ AI Models: {}", ai.models.join(", "));
        println!("  ✓ AI Tasks: {}", ai.tasks.join(", "));
    }
    
    if let Some(edge) = &config.components.edge {
        println!("  ✓ Edge Platforms: {}", edge.platforms.join(", "));
        println!("  ✓ Edge Features: {}", edge.features.join(", "));
    }
    
    if let Some(embedded) = &config.components.embedded {
        println!("  ✓ MCU: {}", embedded.mcu);
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
            files.insert("README.md", format!(r#"# {} Project

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
```"#, template_name));
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
