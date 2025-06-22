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
    shared: Option<Shared>,
    edge: Option<Edge>,
    serverless: Option<Serverless>,
    data_science: Option<DataScience>,
    embedded: Option<Embedded>,
    library: Option<Library>,
    minimal: Option<Minimal>,
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
struct Shared {
    libraries: Vec<String>,
    utilities: Vec<String>,
}

#[derive(Default, Debug)]
struct Edge {
    apps: Vec<String>,
    platforms: Vec<String>,
}

#[derive(Default, Debug)]
struct Serverless {
    functions: Vec<String>,
    providers: Vec<String>,
}

#[derive(Default, Debug)]
struct DataScience {
    models: Vec<String>,
    frameworks: Vec<String>,
}

#[derive(Default, Debug)]
struct Embedded {
    devices: Vec<String>,
    platforms: Vec<String>,
}

#[derive(Default, Debug)]
struct Library {
    name: String,
    features: Vec<String>,
}

#[derive(Default, Debug)]
struct Minimal {
    name: String,
}

/// Options for customizing the preview output
#[derive(Default, Debug)]
struct PreviewOptions {
    /// Framework to use for client, server, or embedded components
    framework: Option<String>,
    
    /// Cloud provider for serverless components
    provider: Option<String>,
    
    /// Application type for edge components
    application_type: Option<String>,
}

/// Execute the preview command to visualize a template without actually creating files
/// 
/// # Work in Progress
/// 
/// This command is currently a work in progress with several known limitations:
/// 
/// * Template variable replacement is incomplete and may not accurately represent final output
/// * Framework-specific features and files may not be fully represented
/// * Some complex template combinations may not preview correctly
/// * File content previews are simplified and may differ from actual generated files
/// * Not all component options are fully supported
/// 
/// Future improvements will address these limitations to provide a more accurate preview experience.
pub fn execute(
    component_type: Option<&str>,
    framework: Option<&str>,
    provider: Option<&str>,
    application_type: Option<&str>
) -> Result<()> {
    println!("{}", "FerrisUp Template Preview".bold().green());
    println!("Preview template structure without creating files\n");
    
    // Get component type interactively if not provided
    let selected_template = if let Some(name) = component_type {
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
            .with_prompt("Select a component type to preview")
            .items(&display_items)
            .default(0)
            .interact()?;
        
        // Extract just the name from the selected template tuple
        templates[selection].0.clone()
    };
    
    // Store provided options for specialized previews
    let options = PreviewOptions {
        framework: framework.map(|f| f.to_string()),
        provider: provider.map(|p| p.to_string()),
        application_type: application_type.map(|a| a.to_string()),
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
        "client" => {
            // Use the specified framework if provided, otherwise show default options
            let frameworks = if let Some(framework) = &options.framework {
                vec![framework.clone()]
            } else {
                vec!["dioxus".to_string(), "tauri".to_string(), "leptos".to_string(), "yew".to_string()]
            };
            
            components.client = Some(Client {
                apps: vec!["web".to_string(), "desktop".to_string()],
                frameworks,
            });
            
            if let Some(framework) = &options.framework {
                println!("{}\n", format!("Using framework: {}", framework).blue());
            }
        },
        "server" => {
            // Use the specified framework if provided, otherwise show default options
            let frameworks = if let Some(framework) = &options.framework {
                vec![framework.clone()]
            } else {
                vec!["axum".to_string(), "poem".to_string(), "actix".to_string(), "rocket".to_string()]
            };
            
            components.server = Some(Server {
                services: vec!["api".to_string(), "auth".to_string()],
                frameworks,
            });
            
            if let Some(framework) = &options.framework {
                println!("{}\n", format!("Using framework: {}", framework).blue());
            }
        },
        "shared" => {
            components.shared = Some(Shared {
                libraries: vec!["common".to_string(), "models".to_string()],
                utilities: vec!["validation".to_string(), "helpers".to_string()],
            });
        },
        "edge" => {
            // Use the specified application type if provided, otherwise show default options
            let apps = if let Some(app_type) = &options.application_type {
                vec![app_type.clone()]
            } else {
                vec!["worker".to_string(), "function".to_string()]
            };
            
            // Use the specified provider if provided, otherwise show default options
            let platforms = if let Some(provider) = &options.provider {
                vec![provider.clone()]
            } else {
                vec!["cloudflare".to_string(), "deno".to_string(), "fastly".to_string()]
            };
            
            components.edge = Some(Edge {
                apps,
                platforms,
            });
            
            if let Some(app_type) = &options.application_type {
                println!("{}\n", format!("Using application type: {}", app_type).blue());
            }
            
            if let Some(provider) = &options.provider {
                println!("{}\n", format!("Using provider: {}", provider).blue());
            }
        },
        "serverless" => {
            // Use the specified provider if provided, otherwise show default options
            let providers = if let Some(provider) = &options.provider {
                vec![provider.clone()]
            } else {
                vec!["aws".to_string(), "vercel".to_string(), "azure".to_string(), "gcp".to_string()]
            };
            
            components.serverless = Some(Serverless {
                functions: vec!["api".to_string(), "processor".to_string()],
                providers,
            });
            
            if let Some(provider) = &options.provider {
                println!("{}\n", format!("Using provider: {}", provider).blue());
            }
        },
        "data-science" => {
            components.data_science = Some(DataScience {
                models: vec!["prediction".to_string(), "classification".to_string()],
                frameworks: vec!["linfa".to_string(), "smartcore".to_string()],
            });
        },
        "embedded" => {
            // Use the specified framework if provided, otherwise show default options
            let _frameworks = if let Some(framework) = &options.framework {
                vec![framework.clone()]
            } else {
                vec!["embassy".to_string(), "rtic".to_string(), "bare-metal".to_string()]
            };
            
            components.embedded = Some(Embedded {
                devices: vec!["rp2040".to_string(), "stm32".to_string(), "esp32".to_string()],
                platforms: vec!["raspberry-pi-pico".to_string(), "nucleo".to_string(), "esp-dev-kit".to_string()],
            });
            
            if let Some(framework) = &options.framework {
                println!("{}\n", format!("Using framework: {}", framework).blue());
            }
        },
        "library" => {
            components.library = Some(Library {
                name: "rust-lib".to_string(),
                features: vec!["async".to_string(), "serde".to_string()],
            });
        },
        "minimal" => {
            components.minimal = Some(Minimal {
                name: "hello-world".to_string(),
            });
        },
        _ => {
            return Err(anyhow!("Unknown component type: {}", selected_template));
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
    display_template_features(&selected_template, &components, &config, &options);
    
    // Show file previews
    println!("\n{}", "Sample Files:".bold());
    display_sample_files(&selected_template, &options);
    
    // Ask if user wants to create a project with this template
    if Confirm::new()
        .with_prompt("Create a new project with this template?")
        .default(false)
        .interact()?
    {
        // Call the new command with the selected template and options
        println!("Using component type: {}", selected_template);
        
        // Pass along any framework, provider, or application_type options
        if let Some(framework) = &options.framework {
            println!("Using framework: {}", framework);
        }
        
        if let Some(provider) = &options.provider {
            println!("Using provider: {}", provider);
        }
        
        if let Some(app_type) = &options.application_type {
            println!("Using application type: {}", app_type);
        }
        
        // Create a project with the selected template and options
        if let Err(e) = crate::commands::new::execute(
            None, 
            Some(&selected_template), 
            options.framework.as_deref(),
            options.provider.as_deref(), 
            options.application_type.as_deref(),
            false, false, false, None
        ) {
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
        // Client
        if let Some(client) = &components.client {
            tree.push_str("├── client/\n");
            for (_i, app) in client.apps.iter().enumerate() {
                tree.push_str(&format!("│   ├── {}\n", app));
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── main.rs\n");
            }
        }
        
        // Server
        if let Some(server) = &components.server {
            tree.push_str("├── server/\n");
            for service in &server.services {
                tree.push_str(&format!("│   ├── {}\n", service));
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── main.rs\n");
            }
        }
        
        // Shared
        if let Some(shared) = &components.shared {
            tree.push_str("├── shared/\n");
            for lib in &shared.libraries {
                tree.push_str(&format!("│   ├── {}\n", lib));
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── lib.rs\n");
            }
        }
        
        // Edge
        if let Some(edge) = &components.edge {
            tree.push_str("├── edge/\n");
            for app in &edge.apps {
                tree.push_str(&format!("│   ├── {}\n", app));
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── main.rs\n");
            }
        }
        
        // Serverless
        if let Some(serverless) = &components.serverless {
            tree.push_str("├── serverless/\n");
            for function in &serverless.functions {
                tree.push_str(&format!("│   ├── {}\n", function));
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── main.rs\n");
            }
        }
        
        // Data Science
        if let Some(data_science) = &components.data_science {
            tree.push_str("├── data-science/\n");
            for model in &data_science.models {
                tree.push_str(&format!("│   ├── {}\n", model));
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── lib.rs\n");
            }
        }
        
        // Embedded
        if let Some(embedded) = &components.embedded {
            tree.push_str("├── embedded/\n");
            for device in &embedded.devices {
                tree.push_str(&format!("│   ├── {}\n", device));
                tree.push_str("│   │   ├── Cargo.toml\n");
                tree.push_str("│   │   ├── memory.x\n");
                tree.push_str("│   │   └── src/\n");
                tree.push_str("│   │       └── main.rs\n");
            }
        }
        
        // Library
        if let Some(_library) = &components.library {
            tree.push_str("├── lib/\n");
            tree.push_str("│   ├── Cargo.toml\n");
            tree.push_str("│   └── src/\n");
            tree.push_str("│       └── lib.rs\n");
        }
        
        // Minimal
        if let Some(_minimal) = &components.minimal {
            tree.push_str("└── src/\n");
            tree.push_str("    └── main.rs\n");
        }
    }
    
    tree
}

/// Display notable features for a given template
/// 
/// Note: This is a work in progress and may not accurately represent all features
/// of the selected template, especially for complex template combinations.
fn display_template_features(template_name: &str, components: &Components, _config: &Config, options: &PreviewOptions) {
    // TODO: Improve feature detection to better match actual template capabilities
    // TODO: Add support for more complex feature combinations based on framework/provider/application-type
    // TODO: Integrate with actual template rendering to show more accurate features
    let mut features_from_metadata = Vec::new();
    
    // Add framework-specific features if a framework is specified
    if let Some(framework) = &options.framework {
        match framework.as_str() {
            "dioxus" => {
                features_from_metadata.push("Dioxus reactive web framework".to_string());
                features_from_metadata.push("Hot-reloading for development".to_string());
            },
            "leptos" => {
                features_from_metadata.push("Leptos reactive web framework".to_string());
                features_from_metadata.push("Server-side rendering support".to_string());
            },
            "axum" => {
                features_from_metadata.push("Axum web server framework".to_string());
                features_from_metadata.push("Async request handling".to_string());
            },
            "tauri" => {
                features_from_metadata.push("Tauri desktop application framework".to_string());
                features_from_metadata.push("Cross-platform desktop support".to_string());
            },
            _ => {}
        }
    }
    
    // Add provider-specific features if a provider is specified
    if let Some(provider) = &options.provider {
        match provider.as_str() {
            "aws" => {
                features_from_metadata.push("AWS Lambda integration".to_string());
                features_from_metadata.push("AWS SAM deployment support".to_string());
            },
            "cloudflare" => {
                features_from_metadata.push("Cloudflare Workers support".to_string());
                features_from_metadata.push("Edge deployment capabilities".to_string());
            },
            _ => {}
        }
    }
    
    // Add application-type specific features
    if let Some(app_type) = &options.application_type {
        match app_type.as_str() {
            "wasm" => {
                features_from_metadata.push("WebAssembly compilation target".to_string());
                features_from_metadata.push("Browser integration".to_string());
            },
            "worker" => {
                features_from_metadata.push("Background worker processing".to_string());
                features_from_metadata.push("Async task handling".to_string());
            },
            _ => {}
        }
    }
    
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
            "client" => vec![
                "Client-side applications",
                "Web and desktop UI components",
                "Dioxus and Tauri integration",
                "State management patterns",
            ],
            "server" => vec![
                "API server with Axum/Poem",
                "Authentication and authorization",
                "Request validation",
                "Middleware support",
            ],
            "shared" => vec![
                "Common code libraries",
                "Shared models and types",
                "Validation utilities",
                "Cross-component helpers",
            ],
            "edge" => vec![
                "Edge computing with Cloudflare Workers",
                "Serverless functions",
                "Web API endpoints",
                "Lightweight deployment",
            ],
            "serverless" => vec![
                "AWS Lambda functions",
                "Vercel serverless integration",
                "API Gateway setup",
                "Event-driven architecture",
            ],
            "data-science" => vec![
                "Data processing pipelines",
                "Machine learning models",
                "Statistical analysis tools",
                "Visualization utilities",
            ],
            "embedded" => vec![
                "Embedded Rust for microcontrollers",
                "No-std environment setup",
                "Hardware abstraction layers",
                "Memory-safe device drivers",
            ],
            "library" => vec![
                "Library crate with lib.rs",
                "Documentation setup",
                "Test infrastructure",
                "Feature flags support",
            ],
            "minimal" => vec![
                "Single binary application",
                "Clean workspace structure",
                "Ready for expansion",
                "Minimal dependencies",
            ],
            _ => vec!["Custom component features"],
        }
    };
    
    // Print features based on template type
    println!("{}", "\nFeatures:".bold());
    for feature in features {
        println!("  • {}", feature);
    }
    
    // Print tech stack based on components
    println!("{}", "\nTech Stack:".bold());
    
    if let Some(client) = &components.client {
        println!("  • Client: {}", client.frameworks.join(", "));
    }
    
    if let Some(server) = &components.server {
        println!("  • Server: {}", server.frameworks.join(", "));
    }
    
    if let Some(shared) = &components.shared {
        println!("  • Shared Libraries: {}", shared.libraries.join(", "));
        println!("  • Utilities: {}", shared.utilities.join(", "));
    }
    
    if let Some(edge) = &components.edge {
        println!("  • Edge Platforms: {}", edge.platforms.join(", "));
    }
    
    if let Some(serverless) = &components.serverless {
        println!("  • Serverless Functions: {}", serverless.functions.join(", "));
        println!("  • Cloud Providers: {}", serverless.providers.join(", "));
    }
    
    if let Some(data_science) = &components.data_science {
        println!("  • Data Science Models: {}", data_science.models.join(", "));
        println!("  • Frameworks: {}", data_science.frameworks.join(", "));
    }
    
    if let Some(embedded) = &components.embedded {
        println!("  • Embedded Devices: {}", embedded.devices.join(", "));
        println!("  • Platforms: {}", embedded.platforms.join(", "));
    }
    
    if let Some(library) = &components.library {
        println!("  • Library: {}", library.name);
        println!("  • Features: {}", library.features.join(", "));
    }
    
    if let Some(minimal) = &components.minimal {
        println!("  • Minimal Application: {}", minimal.name);
    }
}

/// Display sample files from the template
/// 
/// Note: This is a work in progress and may not show all files that would be generated.
/// The content of displayed files may also differ from actual generated content.
fn display_sample_files(template_name: &str, options: &PreviewOptions) {
    // TODO: Improve sample file display to show more accurate content
    // TODO: Better integrate with actual template rendering system
    // TODO: Add support for showing framework-specific file content
    // TODO: Handle complex template combinations more accurately
    println!("  📄 Sample files from template:");
    
    if let Ok(template_dir) = find_template_directory(template_name) {
        // Determine which files to show based on template and options
        let mut key_files = vec![
            "src/main.rs",
            "src/lib.rs",
            "Cargo.toml",
            "README.md"
        ];
        
        // Add framework-specific files if a framework is specified
        if let Some(framework) = &options.framework {
            match framework.as_str() {
                "dioxus" => {
                    key_files.push("index.html");
                    key_files.push("dioxus.toml");
                },
                "leptos" => {
                    key_files.push("index.html");
                    key_files.push("leptos.config.json");
                },
                "axum" => {
                    key_files.push(".env");
                    key_files.push("src/routes/mod.rs");
                },
                "tauri" => {
                    key_files.push("tauri.conf.json");
                    key_files.push("src-tauri/tauri.conf.json");
                },
                _ => {
                    // Default web files for other frameworks
                    key_files.push("index.html");
                    key_files.push("style.css");
                }
            }
        } else if template_name == "client" {
            // Default web files if no framework specified for client
            key_files.push("index.html");
            key_files.push("style.css");
        }
        
        // Add provider-specific files if a provider is specified
        if let Some(provider) = &options.provider {
            match provider.as_str() {
                "aws" => {
                    key_files.push("template.yaml");
                    key_files.push(".aws-sam/build.toml");
                },
                "cloudflare" => {
                    key_files.push("wrangler.toml");
                },
                _ => {}
            }
        }
        
        let mut found_files = false;
        
        for file in key_files {
            let file_path = template_dir.join(file);
            if file_path.exists() && file_path.is_file() {
                found_files = true;
                
                println!("\n{} {}", "File:".cyan().bold(), file.cyan());
                println!("{}", "----------------------------------------".dimmed());
                
                match std::fs::read_to_string(&file_path) {
                    Ok(content) => {
                        // Replace template variables with placeholder values
                        let mut processed_content = content
                            .replace("{{project_name}}", "example-project")
                            .replace("{{crate_name}}", "example_project")
                            .replace("{{description}}", "Example project created with FerrisUp")
                            .replace("{{author}}", "FerrisUp User")
                            .replace("{{mcu_target}}", "rp2040");
                            
                        // Handle Handlebars conditionals for preview purposes
                        processed_content = processed_content
                            .replace("{{#if (eq mcu_target \"rp2040\")}}", "")
                            .replace("{{/if}}", "")
                            .replace("{{#if (eq mcu_target \"stm32\")}}", "<!-- Not selected: ")
                            .replace("{{#if (eq mcu_target \"esp32\")}}", "<!-- Not selected: ")
                            .replace("{{#if (eq mcu_target \"nrf52\")}}", "<!-- Not selected: ");
                        
                        // If content is too long, truncate it
                        let preview_content = if processed_content.len() > 500 {
                            format!("{}...\n(Content truncated, showing first 500 characters)", &processed_content[..500])
                        } else {
                            processed_content
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
            "client" => {
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
                files
            },
            "server" => {
                let mut files = HashMap::new();
                files.insert("server/api/src/main.rs", r#"use axum::{routing::get, Router};

async fn hello() -> &'static str {
    "Hello from Axum server!"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello));
    
    println!("Server starting at http://localhost:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}"#);
                files
            },
            "shared" => {
                let mut files = HashMap::new();
                files.insert("shared/models/src/lib.rs", r#"//! Shared models for the application

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}"#);
                files
            },
            "edge" => {
                let mut files = HashMap::new();
                files.insert("edge/worker/src/main.rs", r#"use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::ok("Hello from Cloudflare Workers!"))
        .get("/api", |_, _| Response::ok("API endpoint"))
        .run(req, env)
        .await
}"#);
                files
            },
            "serverless" => {
                let mut files = HashMap::new();
                files.insert("serverless/api/src/main.rs", r#"use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(handler)).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    
    Ok(json!({
        "statusCode": 200,
        "body": json!({
            "message": "Hello from AWS Lambda!",
            "event": event
        }).to_string()
    }))
}"#);
                files
            },
            "data-science" => {
                let mut files = HashMap::new();
                files.insert("data-science/prediction/src/lib.rs", r#"use ndarray::Array2;
use linfa::prelude::*;

pub struct Model {
    // Model state would go here
}

impl Model {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn predict(&self, data: &Array2<f64>) -> Array2<f64> {
        // Prediction logic would go here
        println!("Predicting with data shape: {:?}", data.shape());
        Array2::zeros((data.nrows(), 1))
    }
}"#);
                files
            },
            "embedded" => {
                let mut files = HashMap::new();
                files.insert("embedded/rp2040/src/main.rs", r#"#![no_std]
#![no_main]

use panic_halt as _;
use rp2040_hal as hal;

#[rp2040_hal::entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let core = hal::pac::CorePeripherals::take().unwrap();
    
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let sio = hal::Sio::new(pac.SIO);
    
    let clocks = hal::clocks::init_clocks_and_plls(
        rp2040_hal::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    ).ok().unwrap();
    
    // Infinite loop
    loop {}
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
            
            // Replace template variables with placeholder values
            // TODO: Improve template variable replacement to handle more variables and complex patterns
            // TODO: Add support for framework-specific variables based on PreviewOptions
            // TODO: Improve template variable replacement to handle more variables and complex patterns
            // TODO: Add support for framework-specific variables based on PreviewOptions
            let mut processed_content = content
                .replace("{{project_name}}", "example-project")
                .replace("{{crate_name}}", "example_project")
                .replace("{{description}}", "Example project created with FerrisUp")
                .replace("{{author}}", "FerrisUp User")
                .replace("{{mcu_target}}", "rp2040");
                
            // Handle Handlebars conditionals for preview purposes
            // TODO: Improve conditional handling for more complex templates
            // TODO: Support framework-specific conditionals based on PreviewOptions
            processed_content = processed_content
                .replace("{{#if (eq mcu_target \"rp2040\")}}", "")
                .replace("{{/if}}", "")
                .replace("{{#if (eq mcu_target \"stm32\")}}", "<!-- Not selected: ")
                .replace("{{#if (eq mcu_target \"esp32\")}}", "<!-- Not selected: ")
                .replace("{{#if (eq mcu_target \"nrf52\")}}", "<!-- Not selected: ");
                
            println!("{}", processed_content);
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
        let options = PreviewOptions::default();
        display_template_features("minimal", &components, &config, &options);
        
        let components = Components::default();
        let config = Config::default();
        let options = PreviewOptions::default();
        display_template_features("full-stack", &components, &config, &options);
        
        let components = Components::default();
        let config = Config::default();
        let options = PreviewOptions::default();
        display_template_features("gen-ai", &components, &config, &options);
    }
    
    #[test]
    fn test_display_sample_files() {
        let options = PreviewOptions::default();
        display_sample_files("minimal", &options);
        display_sample_files("library", &options);
        display_sample_files("full-stack", &options);
    }
}
