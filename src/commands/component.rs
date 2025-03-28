use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::path::{Path, PathBuf};
use std::fs;

use crate::config::{Config, read_config};
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
    
    create_directory(client_dir.to_str().unwrap())?;
    
    // Create src directory
    let src_dir = client_dir.join("src");
    create_directory(src_dir.to_str().unwrap())?;
    
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
"#, project_dir.file_name().unwrap().to_str().unwrap().replace('-', "_"), name);
        
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
    
    create_directory(server_dir.to_str().unwrap())?;
    
    // Create src directory
    let src_dir = server_dir.join("src");
    create_directory(src_dir.to_str().unwrap())?;
    
    // Create Cargo.toml if workspace
    if is_workspace {
        let cargo_content = format!(r#"[package]
name = "{}-{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_dir.file_name().unwrap().to_str().unwrap().replace('-', "_"), name);
        
        fs::write(server_dir.join("Cargo.toml"), cargo_content)?;
    }
    
    println!("{} {} {}", 
        "Added".green(),
        framework.green(),
        "server component".green());
    
    Ok(())
}

fn add_database_component(project_dir: &Path, is_workspace: bool) -> Result<()> {
    // Get database type
    let db_types = vec!["postgresql", "mysql", "sqlite", "mongodb", "redis"];
    let selection = Select::new()
        .with_prompt("Select database type")
        .items(&db_types)
        .default(0)
        .interact()?;
    
    let db_type = db_types[selection];
    
    // Get ORM
    let orms = vec!["diesel", "sqlx", "sea-orm", "mongodb"];
    let selection = Select::new()
        .with_prompt("Select ORM/driver")
        .items(&orms)
        .default(0)
        .interact()?;
    
    let orm = orms[selection];
    
    // Create database directory structure
    let db_dir = if is_workspace {
        project_dir.join("database")
    } else {
        project_dir.join("src").join("database")
    };
    
    create_directory(db_dir.to_str().unwrap())?;
    
    // Create migrations directory if using Diesel
    if orm == "diesel" {
        create_directory(db_dir.join("migrations").to_str().unwrap())?;
    }
    
    // Create schema file
    fs::write(
        db_dir.join("schema.rs"), 
        "// Database schema will be generated here\n"
    )?;
    
    // Create Cargo.toml if workspace
    if is_workspace {
        let cargo_content = format!(r#"[package]
name = "{}-database"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, project_dir.file_name().unwrap().to_str().unwrap().replace('-', "_"));
        
        fs::write(db_dir.join("Cargo.toml"), cargo_content)?;
    }
    
    println!("{} {} {} {}", 
        "Added".green(),
        db_type.green(),
        "database with".green(),
        orm.green());
    
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
    
    create_directory(ai_dir.to_str().unwrap())?;
    
    // Create model and inference subdirectories
    create_directory(ai_dir.join("models").to_str().unwrap())?;
    create_directory(ai_dir.join("inference").to_str().unwrap())?;
    
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
"#, project_dir.file_name().unwrap().to_str().unwrap().replace('-', "_"));
        
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
    
    create_directory(edge_dir.to_str().unwrap())?;
    
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
"#, project_dir.file_name().unwrap().to_str().unwrap().replace('-', "_"));
        
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
    
    create_directory(embedded_dir.to_str().unwrap())?;
    
    // Create src directory
    let src_dir = embedded_dir.join("src");
    create_directory(src_dir.to_str().unwrap())?;
    
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
    create_directory(embedded_dir.join(".cargo").to_str().unwrap())?;
    
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
"#, project_dir.file_name().unwrap().to_str().unwrap().replace('-', "_"));
        
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
    
    create_directory(lib_dir.to_str().unwrap())?;
    
    // Create src directory if workspace
    if is_workspace {
        let src_dir = lib_dir.join("src");
        create_directory(src_dir.to_str().unwrap())?;
        
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
"#, project_dir.file_name().unwrap().to_str().unwrap().replace('-', "_"), name);
        
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

/// Prompt for database components
fn select_database_components() -> Result<Vec<String>> {
    let options = vec![
        "PostgreSQL".to_string(),
        "MySQL".to_string(),
        "SQLite".to_string(),
        "MongoDB".to_string(),
        "Custom...".to_string(),
    ];

    let mut selections = MultiSelect::new()
        .with_prompt("Select database components to include")
        .items(&options)
        .interact()?;
    
    if selections.is_empty() {
        selections.push(4); // custom
    }

    Ok(selections.into_iter().map(|i| options[i].clone()).collect())
}

/// Prompt for client components
fn select_client_components() -> Result<Vec<String>> {
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
