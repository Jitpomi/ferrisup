use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::path::{Path, PathBuf};
use std::fs;

use crate::utils::{create_directory, read_cargo_toml, update_workspace_members};

/// Execute the component command for adding/removing components
pub fn execute(action: Option<&str>, component_type: Option<&str>, project_path: Option<&str>) -> Result<()> {
    println!("{}", "FerrisUp Component Manager".bold().green());
    
    // Get project path
    let project_dir = if let Some(path) = project_path {
        PathBuf::from(path)
    } else {
        // Prompt for project path
        let use_current = Confirm::new()
            .with_prompt("Use current directory?")
            .default(true)
            .interact()?;
        
        if use_current {
            std::env::current_dir()?
        } else {
            let path = Input::<String>::new()
                .with_prompt("Enter project path")
                .interact()?;
            
            PathBuf::from(path)
        }
    };
    
    // Verify it's a Rust project
    if !project_dir.join("Cargo.toml").exists() {
        return Err(anyhow::anyhow!("Not a Rust project (Cargo.toml not found)"));
    }
    
    // Get action (add/remove)
    let action_str = if let Some(act) = action {
        act.to_string()
    } else {
        let options = vec!["add", "remove", "list"];
        let selection = Select::new()
            .with_prompt("Select action")
            .items(&options)
            .default(0)
            .interact()?;
        
        options[selection].to_string()
    };
    
    // Execute the selected action
    match action_str.as_str() {
        "add" => add_component(&project_dir, component_type)?,
        "remove" => remove_component(&project_dir, component_type)?,
        "list" => list_components(&project_dir)?,
        _ => return Err(anyhow::anyhow!("Invalid action. Use 'add', 'remove', or 'list'")),
    }
    
    Ok(())
}

/// Add a component to an existing project
fn add_component(project_dir: &Path, component_type: Option<&str>) -> Result<()> {
    // Get component type
    let component = if let Some(ctype) = component_type {
        ctype.to_string()
    } else {
        let options = vec![
            "client", "server", "database", "ai", "edge", "embedded",
            "library", "test", "documentation", "deployment",
        ];
        
        let selection = Select::new()
            .with_prompt("Select component type to add")
            .items(&options)
            .default(0)
            .interact()?;
        
        options[selection].to_string()
    };
    
    // Get workspace structure
    let cargo_content = read_cargo_toml(project_dir)?;
    let is_workspace = cargo_content.contains("[workspace]");
    
    match component.as_str() {
        "client" => add_client_component(project_dir, is_workspace)?,
        "server" => add_server_component(project_dir, is_workspace)?,
        "database" => add_database_component(project_dir, is_workspace)?,
        "ai" => add_ai_component(project_dir, is_workspace)?,
        "edge" => add_edge_component(project_dir, is_workspace)?,
        "embedded" => add_embedded_component(project_dir, is_workspace)?,
        "library" => add_library_component(project_dir, is_workspace)?,
        _ => return Err(anyhow::anyhow!("Component type not supported yet")),
    }
    
    // Update workspace if needed
    if is_workspace {
        update_workspace_members(project_dir)?;
    }
    
    println!("{} {} {}", 
        "Successfully added".green(),
        component.green(),
        "component".green());
    
    Ok(())
}

/// Remove a component from an existing project
fn remove_component(project_dir: &Path, component_type: Option<&str>) -> Result<()> {
    // List existing components
    let components = discover_components(project_dir)?;
    
    if components.is_empty() {
        println!("{}", "No components found to remove".yellow());
        return Ok(());
    }
    
    // Get component to remove
    let component_path = if let Some(ctype) = component_type {
        // Find the component by type
        let matching: Vec<String> = components.iter()
            .filter(|c| c.contains(ctype))
            .cloned()
            .collect();
        
        if matching.is_empty() {
            return Err(anyhow::anyhow!("No matching component found"));
        } else if matching.len() == 1 {
            matching[0].clone()
        } else {
            // Multiple matches, ask user to select one
            let selection = Select::new()
                .with_prompt("Select component to remove")
                .items(&matching)
                .default(0)
                .interact()?;
            
            matching[selection].clone()
        }
    } else {
        // Let user select from all components
        let selection = Select::new()
            .with_prompt("Select component to remove")
            .items(&components)
            .default(0)
            .interact()?;
        
        components[selection].clone()
    };
    
    // Confirm removal
    let confirm = Confirm::new()
        .with_prompt(format!("Remove component {}?", component_path))
        .default(false)
        .interact()?;
    
    if !confirm {
        println!("Operation cancelled");
        return Ok(());
    }
    
    // Remove the component
    let full_path = project_dir.join(&component_path);
    fs::remove_dir_all(&full_path)
        .context(format!("Failed to remove {}", full_path.display()))?;
    
    println!("{} {}", "Successfully removed component:".green(), component_path);
    
    // Update workspace if needed
    let cargo_content = read_cargo_toml(project_dir)?;
    if cargo_content.contains("[workspace]") {
        update_workspace_members(project_dir)?;
    }
    
    Ok(())
}

/// List components in a project
fn list_components(project_dir: &Path) -> Result<()> {
    let components = discover_components(project_dir)?;
    
    if components.is_empty() {
        println!("{}", "No components found".yellow());
        return Ok(());
    }
    
    println!("\n{}", "Project Components:".bold());
    for (i, component) in components.iter().enumerate() {
        println!("  {}. {}", i + 1, component);
    }
    
    Ok(())
}

/// Discover components in a project
fn discover_components(project_dir: &Path) -> Result<Vec<String>> {
    let mut components = Vec::new();
    
    let dirs = vec!["client", "server", "shared", "ai", "edge", "embedded", "libs"];
    
    for dir in dirs {
        let dir_path = project_dir.join(dir);
        if dir_path.exists() && dir_path.is_dir() {
            // Check for subcomponents
            if let Ok(entries) = fs::read_dir(&dir_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_dir() {
                        let rel_path = entry.path().strip_prefix(project_dir)?.to_string_lossy().into_owned();
                        components.push(rel_path);
                    }
                }
            }
        }
    }
    
    Ok(components)
}

// Implement component addition functions
fn add_client_component(project_dir: &Path, is_workspace: bool) -> Result<()> {
    // Get client framework
    let frameworks = vec!["dioxus", "tauri", "leptos", "yew"];
    let selection = Select::new()
        .with_prompt("Select client framework")
        .items(&frameworks)
        .default(0)
        .interact()?;
    
    let framework = frameworks[selection];
    
    // Get client name
    let name = Input::<String>::new()
        .with_prompt("Enter client app name")
        .default("web".to_string())
        .interact()?;
    
    // Create client directory structure
    let client_dir = if is_workspace {
        project_dir.join("client").join(&name)
    } else {
        project_dir.join("src").join("client")
    };
    
    create_directory(&client_dir)?;
    
    // Create src directory
    let src_dir = client_dir.join("src");
    create_directory(&src_dir)?;
    
    // Create main.rs with appropriate content
    let main_content = match framework {
        "dioxus" => r#"use dioxus::prelude::*;

fn main() {
    dioxus::web::launch(App);
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div { "Hello from Dioxus!" }
    })
}"#,
        "tauri" => r#"fn main() {
    println!("Hello from Tauri app!");
}"#,
        "leptos" => r#"use leptos::*;

fn main() {
    mount_to_body(|| view! { <App/> });
}

#[component]
fn App() -> impl IntoView {
    view! { <div>"Hello from Leptos!"</div> }
}"#,
        _ => r#"fn main() {
    println!("Hello from client app!");
}"#,
    };
    
    fs::write(src_dir.join("main.rs"), main_content)?;
    
    // Create Cargo.toml if workspace
    if is_workspace {
        let cargo_content = format!(r#"[package]
name = "{}-{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_dir.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project directory name"))?
        .replace('-', "_"), name);
        
        fs::write(client_dir.join("Cargo.toml"), cargo_content)?;
    }
    
    println!("{} {} {}", 
        "Added".green(),
        framework.green(),
        "client component".green());
    
    Ok(())
}

fn add_server_component(project_dir: &Path, is_workspace: bool) -> Result<()> {
    // Get server framework
    let frameworks = vec!["poem", "axum", "actix-web", "rocket", "warp"];
    let selection = Select::new()
        .with_prompt("Select server framework")
        .items(&frameworks)
        .default(0)
        .interact()?;
    
    let framework = frameworks[selection];
    
    // Get service name
    let name = Input::<String>::new()
        .with_prompt("Enter service name")
        .default("api".to_string())
        .interact()?;
    
    // Create server directory structure
    let server_dir = if is_workspace {
        project_dir.join("server").join(&name)
    } else {
        project_dir.join("src").join("server")
    };
    
    create_directory(&server_dir)?;
    
    // Create src directory
    let src_dir = server_dir.join("src");
    create_directory(&src_dir)?;
    
    // Create Cargo.toml if workspace
    if is_workspace {
        let cargo_content = format!(r#"[package]
name = "{}-{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_dir.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project directory name"))?
        .replace('-', "_"), name);
        
        fs::write(server_dir.join("Cargo.toml"), cargo_content)?;
    }
    
    println!("{} {} {}", 
        "Added".green(),
        framework.green(),
        "server component".green());
    
    Ok(())
}

fn add_database_component(project_dir: &Path, is_workspace: bool) -> Result<()> {
    // Use the enhanced database selection
    let selections = _select_database_components()?;
    
    // Parse the selections to determine which database types were selected
    let mut primary_db = None;
    let mut cache_db = None;
    let mut vector_db = None;
    let mut graph_db = None;
    let mut custom_db = false;
    
    for selection in &selections {
        if selection.starts_with("Primary: ") {
            primary_db = Some(selection.trim_start_matches("Primary: ").to_lowercase());
        } else if selection.starts_with("Cache: ") {
            cache_db = Some(selection.trim_start_matches("Cache: ").to_lowercase());
        } else if selection.starts_with("Vector: ") {
            vector_db = Some(selection.trim_start_matches("Vector: ").to_lowercase());
        } else if selection.starts_with("Graph: ") {
            graph_db = Some(selection.trim_start_matches("Graph: ").to_lowercase());
        } else if selection == "Custom..." {
            custom_db = true;
        }
    }
    
    // If custom is selected, prompt for the database name
    if custom_db || (primary_db.is_none() && cache_db.is_none() && vector_db.is_none() && graph_db.is_none()) {
        let db_name = Input::<String>::new()
            .with_prompt("Enter custom database name")
            .interact()?;
        
        primary_db = Some(db_name.to_lowercase());
    }
    
    // Get ORM/driver selection based on the primary database
    let mut orm = String::new();
    
    if let Some(primary) = &primary_db {
        let orm_options = match primary.as_str() {
            "postgresql" | "postgres" | "mysql" | "sqlite" | "cockroachdb" | "timescaledb" => {
                vec!["diesel", "sqlx", "sea-orm", "tokio-postgres", "sqlalchemy-rs"]
            },
            "mongodb" => {
                vec!["mongodb", "wither"]
            },
            "typedb" => {
                vec!["typedb-client", "custom"]
            },
            "scylladb" => {
                vec!["scylla-rs", "cassandra-rs"]
            },
            _ => {
                vec!["custom"]
            }
        };
        
        let selection = Select::new()
            .with_prompt("Select ORM/driver for primary database")
            .items(&orm_options)
            .default(0)
            .interact()?;
        
        orm = orm_options[selection].to_string();
    }
    
    // Create database directory structure
    let db_dir = if is_workspace {
        project_dir.join("database")
    } else {
        project_dir.join("src").join("database")
    };
    
    create_directory(&db_dir)?;
    
    // Create subdirectories for different database types
    if primary_db.is_some() {
        create_directory(&db_dir.join("primary"))?;
        
        // Create migrations directory if using Diesel or Sea-ORM
        if orm == "diesel" || orm == "sea-orm" {
            create_directory(&db_dir.join("migrations"))?;
        }
    }
    
    if cache_db.is_some() {
        create_directory(&db_dir.join("cache"))?;
    }
    
    if vector_db.is_some() {
        create_directory(&db_dir.join("vector"))?;
    }
    
    if graph_db.is_some() {
        create_directory(&db_dir.join("graph"))?;
    }
    
    // Create schema file
    fs::write(
        db_dir.join("schema.rs"), 
        "// Database schema will be generated here\n"
    )?;
    
    // Create Cargo.toml if workspace
    if is_workspace {
        let mut dependencies = String::new();
        
        // Add dependencies based on selected databases
        if let Some(primary) = &primary_db {
            match primary.as_str() {
                "postgresql" | "postgres" => {
                    if orm == "diesel" {
                        dependencies.push_str("diesel = { version = \"2.1.0\", features = [\"postgres\"] }\n");
                    } else if orm == "sqlx" {
                        dependencies.push_str("sqlx = { version = \"0.7.3\", features = [\"runtime-tokio-rustls\", \"postgres\"] }\n");
                    } else if orm == "sea-orm" {
                        dependencies.push_str("sea-orm = { version = \"0.12.10\", features = [\"sqlx-postgres\", \"runtime-tokio-rustls\"] }\n");
                    }
                },
                "mysql" => {
                    if orm == "diesel" {
                        dependencies.push_str("diesel = { version = \"2.1.0\", features = [\"mysql\"] }\n");
                    } else if orm == "sqlx" {
                        dependencies.push_str("sqlx = { version = \"0.7.3\", features = [\"runtime-tokio-rustls\", \"mysql\"] }\n");
                    } else if orm == "sea-orm" {
                        dependencies.push_str("sea-orm = { version = \"0.12.10\", features = [\"sqlx-mysql\", \"runtime-tokio-rustls\"] }\n");
                    }
                },
                "sqlite" => {
                    if orm == "diesel" {
                        dependencies.push_str("diesel = { version = \"2.1.0\", features = [\"sqlite\"] }\n");
                    } else if orm == "sqlx" {
                        dependencies.push_str("sqlx = { version = \"0.7.3\", features = [\"runtime-tokio-rustls\", \"sqlite\"] }\n");
                    } else if orm == "sea-orm" {
                        dependencies.push_str("sea-orm = { version = \"0.12.10\", features = [\"sqlx-sqlite\", \"runtime-tokio-rustls\"] }\n");
                    }
                },
                "mongodb" => {
                    dependencies.push_str("mongodb = \"2.8.0\"\n");
                    dependencies.push_str("bson = \"2.9.0\"\n");
                },
                "typedb" => {
                    if !dependencies.contains("typedb-client") {
                        dependencies.push_str("typedb-client = \"2.24.0\"\n");
                    }
                },
                "cockroachdb" => {
                    // CockroachDB uses the PostgreSQL driver
                    dependencies.push_str("sqlx = { version = \"0.7.3\", features = [\"runtime-tokio-rustls\", \"postgres\"] }\n");
                },
                "timescaledb" => {
                    // TimescaleDB uses the PostgreSQL driver
                    dependencies.push_str("sqlx = { version = \"0.7.3\", features = [\"runtime-tokio-rustls\", \"postgres\"] }\n");
                },
                "scylladb" => {
                    dependencies.push_str("scylla = \"0.12.0\"\n");
                },
                _ => {}
            }
        }
        
        if let Some(cache) = &cache_db {
            match cache.as_str() {
                "redis" => {
                    dependencies.push_str("redis = { version = \"0.24.0\", features = [\"tokio-comp\"] }\n");
                },
                "memcached" => {
                    dependencies.push_str("memcache = \"0.17.0\"\n");
                },
                "hazelcast" => {
                    dependencies.push_str("hazelcast-client = \"0.3.0\"\n");
                },
                "aerospike" => {
                    dependencies.push_str("aerospike = \"1.0.1\"\n");
                },
                "ignite" => {
                    dependencies.push_str("ignite-rs = \"0.1.0\"\n");
                },
                _ => {}
            }
        }
        
        if let Some(vector) = &vector_db {
            match vector.as_str() {
                "pinecone" => {
                    dependencies.push_str("pinecone-client = \"0.2.0\"\n");
                    dependencies.push_str("reqwest = { version = \"0.11.27\", features = [\"json\"] }\n");
                },
                "qdrant" => {
                    dependencies.push_str("qdrant-client = \"1.8.0\"\n");
                },
                "milvus" => {
                    dependencies.push_str("milvus-sdk-rust = \"0.1.0\"\n");
                },
                "chroma" => {
                    dependencies.push_str("chromadb = \"0.1.0\"\n");
                    dependencies.push_str("reqwest = { version = \"0.11.27\", features = [\"json\"] }\n");
                },
                "weaviate" => {
                    dependencies.push_str("weaviate-client = \"0.1.0\"\n");
                    dependencies.push_str("reqwest = { version = \"0.11.27\", features = [\"json\"] }\n");
                },
                "vespa" => {
                    dependencies.push_str("vespa-rs = \"0.1.0\"\n");
                    dependencies.push_str("reqwest = { version = \"0.11.27\", features = [\"json\"] }\n");
                },
                "faiss" => {
                    dependencies.push_str("faiss-rs = \"0.1.0\"\n");
                },
                "opensearch" => {
                    dependencies.push_str("opensearch = \"2.1.0\"\n");
                },
                _ => {}
            }
        }
        
        if let Some(graph) = &graph_db {
            match graph.as_str() {
                "neo4j" => {
                    dependencies.push_str("neo4rs = \"0.8.0\"\n");
                },
                "typedb" => {
                    if !dependencies.contains("typedb-client") {
                        dependencies.push_str("typedb-client = \"2.24.0\"\n");
                    }
                },
                "arangodb" => {
                    dependencies.push_str("arangors = \"0.5.4\"\n");
                },
                "janusgraph" => {
                    dependencies.push_str("gremlin-client = \"0.8.1\"\n");
                },
                "dgraph" => {
                    dependencies.push_str("dgraph-rs = \"0.1.0\"\n");
                },
                "tigergraph" => {
                    dependencies.push_str("reqwest = { version = \"0.11.27\", features = [\"json\"] }\n");
                },
                "neptune" => {
                    dependencies.push_str("gremlin-client = \"0.8.1\"\n");
                    dependencies.push_str("aws-sdk-neptune = \"0.40.0\"\n");
                },
                _ => {}
            }
        }
        
        // Common dependencies for all database types
        dependencies.push_str("tokio = { version = \"1.36.0\", features = [\"full\"] }\n");
        dependencies.push_str("serde = { version = \"1.0.197\", features = [\"derive\"] }\n");
        dependencies.push_str("anyhow = \"1.0.80\"\n");
        
        let cargo_content = format!(r#"[package]
name = "{}-database"
version = "0.1.0"
edition = "2021"

[dependencies]
{}
"#, project_dir.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project directory name"))?
        .replace('-', "_"), dependencies);
        
        fs::write(db_dir.join("Cargo.toml"), cargo_content)?;
    } else {
        // If not a workspace, add dependencies to the main Cargo.toml file
        let cargo_path = project_dir.join("Cargo.toml");
        println!("Adding dependencies to: {:?}", cargo_path);
        
        // Read the current content
        let content = match fs::read_to_string(&cargo_path) {
            Ok(content) => content,
            Err(e) => {
                println!("Error reading Cargo.toml: {:?}", e);
                return Ok(());
            }
        };
        
        // Build the dependencies string
        let mut deps = String::new();
        
        // Add primary database dependencies
        if let Some(primary) = &primary_db {
            match primary.as_str() {
                "typedb" => {
                    deps.push_str("typedb-client = \"2.24.0\"\n");
                },
                "postgresql" | "postgres" => {
                    if orm == "diesel" {
                        deps.push_str("diesel = { version = \"2.1.0\", features = [\"postgres\"] }\n");
                    } else if orm == "sqlx" {
                        deps.push_str("sqlx = { version = \"0.7.3\", features = [\"runtime-tokio-rustls\", \"postgres\"] }\n");
                    } else if orm == "sea-orm" {
                        deps.push_str("sea-orm = { version = \"0.12.10\", features = [\"sqlx-postgres\", \"runtime-tokio-rustls\"] }\n");
                    }
                },
                // Add other cases as needed
                _ => {}
            }
        }
        
        // Add cache database dependencies
        if let Some(cache) = &cache_db {
            match cache.as_str() {
                "redis" => {
                    deps.push_str("redis = \"0.24.0\"\n");
                },
                // Add other cases as needed
                _ => {}
            }
        }
        
        // Add vector database dependencies
        if let Some(vector) = &vector_db {
            match vector.as_str() {
                "qdrant" => {
                    deps.push_str("qdrant-client = \"1.7.0\"\n");
                },
                // Add other cases as needed
                _ => {}
            }
        }
        
        // Add common dependencies
        deps.push_str("anyhow = \"1.0.80\"\n");
        deps.push_str("tokio = { version = \"1.36.0\", features = [\"full\"] }\n");
        
        // Create the new content with dependencies
        let new_content = if content.contains("[dependencies]") {
            // Replace the empty dependencies section with our dependencies
            content.replace("[dependencies]", &format!("[dependencies]\n{}", deps))
        } else {
            // Add a new dependencies section
            format!("{}\n[dependencies]\n{}", content, deps)
        };
        
        // Write the new content back to the file
        match fs::write(&cargo_path, new_content) {
            Ok(_) => println!("Dependencies added successfully to Cargo.toml"),
            Err(e) => println!("Error writing to Cargo.toml: {:?}", e),
        }
    }
    
    // Print summary of what was added
    println!("{} Database components:", "Added".green());
    
    if let Some(primary) = &primary_db {
        println!("  - {} {} {}", "Primary:".green(), primary.green(), format!("with {}", orm).green());
    }
    
    if let Some(cache) = &cache_db {
        println!("  - {} {}", "Cache:".green(), cache.green());
    }
    
    if let Some(vector) = &vector_db {
        println!("  - {} {}", "Vector:".green(), vector.green());
    }
    
    if let Some(graph) = &graph_db {
        println!("  - {} {}", "Graph:".green(), graph.green());
    }
    
    Ok(())
}

fn add_ai_component(project_dir: &Path, is_workspace: bool) -> Result<()> {
    // Get AI model types
    let models = vec!["llama", "bert", "whisper", "stable-diffusion", "custom"];
    let mut selections = MultiSelect::new()
        .with_prompt("Select AI models (space to select, enter to confirm)")
        .items(&models)
        .interact()?;
    
    if selections.is_empty() {
        println!("{}", "No models selected, using 'custom'".yellow());
        selections.push(4); // custom
    }
    
    let selected_models: Vec<String> = selections.iter()
        .map(|&i| models[i].to_string())
        .collect();
    
    // Create AI directory structure
    let ai_dir = if is_workspace {
        project_dir.join("ai")
    } else {
        project_dir.join("src").join("ai")
    };
    
    create_directory(&ai_dir)?;
    
    // Create model and inference subdirectories
    create_directory(&ai_dir.join("models"))?;
    create_directory(&ai_dir.join("inference"))?;
    
    // Create lib.rs
    let lib_content = r#"//! AI components for inference and model management

/// Initialize models
pub fn init_models() {
    println!("Initializing AI models...");
}

/// Run inference on input data
pub fn run_inference(input: &str) -> String {
    format!("Inference result for: {}", input)
}
"#;
    
    fs::write(ai_dir.join("lib.rs"), lib_content)?;
    
    // Create Cargo.toml if workspace
    if is_workspace {
        let cargo_content = format!(r#"[package]
name = "{}-ai"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_dir.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project directory name"))?
        .replace('-', "_"));
        
        fs::write(ai_dir.join("Cargo.toml"), cargo_content)?;
    }
    
    println!("{} {} {}", 
        "Added AI component with models:".green(),
        selected_models.join(", ").green(),
        "".green());
    
    Ok(())
}

fn add_edge_component(project_dir: &Path, is_workspace: bool) -> Result<()> {
    // Get edge platforms
    let platforms = vec!["cloudflare-workers", "deno-deploy", "browser-wasm", "edge-lambda"];
    let mut selections = MultiSelect::new()
        .with_prompt("Select edge platforms (space to select, enter to confirm)")
        .items(&platforms)
        .interact()?;
    
    if selections.is_empty() {
        println!("{}", "No platforms selected, using 'browser-wasm'".yellow());
        selections.push(2); // browser-wasm
    }
    
    let selected_platforms: Vec<String> = selections.iter()
        .map(|&i| platforms[i].to_string())
        .collect();
    
    // Create edge directory structure
    let edge_dir = if is_workspace {
        project_dir.join("edge")
    } else {
        project_dir.join("src").join("edge")
    };
    
    create_directory(&edge_dir)?;
    
    // Create lib.rs
    let lib_content = r#"//! Edge components for WebAssembly and edge deployment

/// Initialize the edge runtime
pub fn init() {
    println!("Initializing edge runtime...");
}

/// Run the edge handler
pub fn handler(request: &str) -> String {
    format!("Edge response for: {}", request)
}
"#;
    
    fs::write(edge_dir.join("lib.rs"), lib_content)?;
    
    // Create Cargo.toml if workspace
    if is_workspace {
        let cargo_content = format!(r#"[package]
name = "{}-edge"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
"#, project_dir.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project directory name"))?
        .replace('-', "_"));
        
        fs::write(edge_dir.join("Cargo.toml"), cargo_content)?;
    }
    
    println!("{} {} {}", 
        "Added edge component with platforms:".green(),
        selected_platforms.join(", ").green(),
        "".green());
    
    Ok(())
}

fn add_embedded_component(project_dir: &Path, is_workspace: bool) -> Result<()> {
    // Get microcontroller type
    let mcus = vec!["rp2040", "esp32", "stm32", "arduino"];
    let selection = Select::new()
        .with_prompt("Select microcontroller")
        .items(&mcus)
        .default(0)
        .interact()?;
    
    let mcu = mcus[selection];
    
    // Create embedded directory structure
    let embedded_dir = if is_workspace {
        project_dir.join("embedded")
    } else {
        project_dir.join("src").join("embedded")
    };
    
    create_directory(&embedded_dir)?;
    
    // Create src directory
    let src_dir = embedded_dir.join("src");
    create_directory(&src_dir)?;
    
    // Create main.rs
    let main_content = r#"#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize hardware
    
    loop {
        // Main embedded loop
    }
}
"#;
    
    fs::write(src_dir.join("main.rs"), main_content)?;
    
    // Create memory.x file for microcontroller
    fs::write(
        embedded_dir.join("memory.x"),
        "/* Memory layout */\n"
    )?;
    
    // Create .cargo directory and config
    create_directory(&embedded_dir.join(".cargo"))?;
    
    fs::write(
        embedded_dir.join(".cargo").join("config.toml"),
        "[build]\ntarget = \"thumbv6m-none-eabi\"\n"
    )?;
    
    // Create Cargo.toml if workspace
    if is_workspace {
        let cargo_content = format!(r#"[package]
name = "{}-embedded"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_dir.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project directory name"))?
        .replace('-', "_"));
        
        fs::write(embedded_dir.join("Cargo.toml"), cargo_content)?;
    }
    
    println!("{} {} {}", 
        "Added embedded component for".green(),
        mcu.green(),
        "microcontroller".green());
    
    Ok(())
}

fn add_library_component(project_dir: &Path, is_workspace: bool) -> Result<()> {
    // Get library name
    let name = Input::<String>::new()
        .with_prompt("Enter library name")
        .default("core".to_string())
        .interact()?;
    
    // Create library directory structure
    let lib_dir = if is_workspace {
        if project_dir.join("shared").exists() {
            project_dir.join("shared").join(&name)
        } else {
            project_dir.join("libs").join(&name)
        }
    } else {
        project_dir.join("src").join("lib")
    };
    
    create_directory(&lib_dir)?;
    
    // Create src directory if workspace
    if is_workspace {
        let src_dir = lib_dir.join("src");
        create_directory(&src_dir)?;
        
        // Create lib.rs
        fs::write(
            src_dir.join("lib.rs"),
            "//! Library crate\n\npub fn hello() -> &'static str {\n    \"Hello from library!\"\n}\n"
        )?;
        
        // Create Cargo.toml
        let cargo_content = format!(r#"[package]
name = "{}-{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_dir.file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project directory name"))?
        .replace('-', "_"), name);
        
        fs::write(lib_dir.join("Cargo.toml"), cargo_content)?;
    } else {
        // Create lib.rs in src/lib
        fs::write(
            lib_dir.join("lib.rs"),
            "//! Library module\n\npub fn hello() -> &'static str {\n    \"Hello from library module!\"\n}\n"
        )?;
    }
    
    println!("{} {} {}", 
        "Added".green(),
        name.green(),
        "library component".green());
    
    Ok(())
}

#[allow(dead_code)]
fn _select_database_components() -> Result<Vec<String>> {
    let primary_options = vec![
        "PostgreSQL".to_string(),
        "MySQL".to_string(),
        "SQLite".to_string(),
        "MongoDB".to_string(),
        "TypeDB".to_string(),
        "CockroachDB".to_string(),
        "TimescaleDB".to_string(),
        "ScyllaDB".to_string(),
    ];

    let cache_options = vec![
        "Redis".to_string(),
        "Memcached".to_string(),
        "Hazelcast".to_string(),
        "Aerospike".to_string(),
        "Ignite".to_string(),
    ];

    let vector_options = vec![
        "Pinecone".to_string(),
        "Qdrant".to_string(),
        "Milvus".to_string(),
        "Chroma".to_string(),
        "Weaviate".to_string(),
        "Vespa".to_string(),
        "Faiss".to_string(),
        "OpenSearch".to_string(),
    ];

    let graph_options = vec![
        "Neo4j".to_string(),
        "TypeDB".to_string(),
        "ArangoDB".to_string(),
        "JanusGraph".to_string(),
        "DGraph".to_string(),
        "TigerGraph".to_string(),
        "Amazon Neptune".to_string(),
    ];

    // Combine all options
    let mut all_options = Vec::new();
    all_options.extend(primary_options.iter().map(|db| format!("Primary: {}", db)));
    all_options.extend(cache_options.iter().map(|db| format!("Cache: {}", db)));
    all_options.extend(vector_options.iter().map(|db| format!("Vector: {}", db)));
    all_options.extend(graph_options.iter().map(|db| format!("Graph: {}", db)));
    all_options.push("Custom...".to_string());

    let selections = MultiSelect::new()
        .with_prompt("Select database components to include (you can select multiple types)")
        .items(&all_options)
        .interact()?;
    
    if selections.is_empty() {
        return Ok(vec!["Custom...".to_string()]);
    }

    Ok(selections.into_iter().map(|i| all_options[i].clone()).collect())
}

#[allow(dead_code)]
fn _select_client_components() -> Result<Vec<String>> {
    let options = vec![
        "Dioxus (Desktop)".to_string(),
        "Tauri".to_string(),
        "Browser (WASM)".to_string(),
        "Custom...".to_string(),
    ];

    let mut selections = MultiSelect::new()
        .with_prompt("Select client components to include")
        .items(&options)
        .interact()?;

    if selections.is_empty() {
        selections.push(2); // browser-wasm
    }

    Ok(selections.into_iter().map(|i| options[i].clone()).collect())
}
