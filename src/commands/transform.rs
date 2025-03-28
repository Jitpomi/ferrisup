use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Input, MultiSelect, Select};

use crate::templates::{get_template, list_templates};
use crate::utils::{create_directory, read_cargo_toml, update_workspace_members};

pub fn execute(project_path: Option<&str>, template_name: Option<&str>) -> Result<()> {
    println!("{}", "FerrisUp Interactive Project Transformer".bold().green());
    println!("{}", "Transform your existing Rust project with new features".blue());
    
    // Interactive mode if project path is not provided
    let path_str = match project_path {
        Some(p) => p.to_string(),
        None => {
            // Default to current directory
            let current_dir = std::env::current_dir()?;
            let use_current_dir = Confirm::new()
                .with_prompt("Use current directory for transformation?")
                .default(true)
                .interact()?;
            
            if use_current_dir {
                current_dir.to_string_lossy().to_string()
            } else {
                // Prompt for project path
                Input::new()
                    .with_prompt("Enter the path to your project")
                    .interact_text()?
            }
        }
    };
    
    let project_dir = Path::new(&path_str);
    
    // Check if directory exists
    if !project_dir.exists() {
        println!("{} {} {}", 
            "Error:".red().bold(), 
            "Directory".red(), 
            format!("'{}' does not exist", path_str).red());
        
        // Ask if user wants to specify a different path
        if Confirm::new()
            .with_prompt("Would you like to specify a different path?")
            .default(true)
            .interact()?
        {
            return execute(None, template_name);
        } else {
            return Ok(());
        }
    }
    
    // Check if it's a valid Rust project (has Cargo.toml)
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        println!("{} {}", 
            "Error:".red().bold(), 
            format!("'{}' is not a valid Rust project (Cargo.toml not found)", path_str).red());
        
        // Ask if user wants to specify a different path
        if Confirm::new()
            .with_prompt("Would you like to specify a different path?")
            .default(true)
            .interact()?
        {
            return execute(None, template_name);
        } else {
            return Ok(());
        }
    }
    
    // If template name is not provided, offer interactive selection
    let selected_template = match template_name {
        Some(t) => t.to_string(),
        None => {
            // Ask about transformation approach
            println!("\n{}", "Transformation Approach".bold().cyan());
            let approach_options = vec![
                "Use a predefined template",
                "Customize transformation manually",
            ];
            
            let approach_idx = Select::new()
                .with_prompt("How would you like to transform your project?")
                .items(&approach_options)
                .default(0)
                .interact()?;
            
            if approach_idx == 0 {
                // Get available templates
                let templates = list_templates()?;
                let template_names: Vec<String> = templates.iter().map(|(name, desc)| {
                    format!("{} - {}", name, desc)
                }).collect();
                
                // Present template options
                let template_idx = Select::new()
                    .with_prompt("Select a template")
                    .items(&template_names)
                    .default(0)
                    .interact()?;
                
                templates[template_idx].0.clone()
            } else {
                // Custom transformation
                // First analyze the current project structure
                let structure = analyze_project_structure(project_dir)?;
                
                println!("\n{}", "Custom Transformation Options".bold().cyan());
                
                let component_options = vec![
                    "Add client applications",
                    "Add server services",
                    "Add shared libraries",
                    "Add AI components",
                    "Add edge computing",
                    "Add embedded systems support",
                    "Convert to workspace structure",
                ];
                
                let selections = MultiSelect::new()
                    .with_prompt("Select components to add to your project")
                    .items(&component_options)
                    .interact()?;
                
                // Flag for custom mode
                if !selections.is_empty() {
                    // Apply custom transformations
                    if selections.contains(&6) && !structure.is_workspace {
                        init_workspace(project_dir, &structure)?;
                    }
                    
                    if selections.contains(&0) {
                        add_client(project_dir)?;
                    }
                    
                    if selections.contains(&1) {
                        add_server(project_dir)?;
                    }
                    
                    if selections.contains(&2) {
                        add_libs(project_dir)?;
                    }
                    
                    if selections.contains(&3) {
                        add_ai(project_dir)?;
                    }
                    
                    if selections.contains(&4) {
                        add_edge(project_dir)?;
                    }
                    
                    if selections.contains(&5) {
                        add_embedded(project_dir)?;
                    }
                    
                    // Print success message
                    println!("\n{}", "Project transformation complete!".bold().green());
                    println!("Your project has been customized with the selected components.");
                    
                    return Ok(());
                }
                
                // If no custom selections, default to minimal
                "minimal".to_string()
            }
        }
    };
    
    // Get template configuration
    let _template = get_template(&selected_template)
        .context(format!("Failed to find template '{}'", selected_template))?;
    
    println!("{} {} {} {}", 
        "Transforming".yellow().bold(), 
        path_str.yellow(), 
        "to".yellow().bold(), 
        selected_template.yellow().bold());
    
    // Analyze the current project structure
    let structure = analyze_project_structure(project_dir)?;
    
    // Confirm the transformation
    let confirm_msg = format!(
        "This will transform your project to a '{}' template. This may modify your project structure and files. Continue?",
        selected_template
    );
    
    let proceed = Confirm::new()
        .with_prompt(&confirm_msg)
        .default(true)
        .interact()?;
    
    if !proceed {
        println!("{}", "Transformation cancelled.".yellow());
        return Ok(());
    }
    
    // Transform the project to the selected template
    match selected_template.as_str() {
        "full-stack" => transform_to_full_stack(project_dir, &structure)?,
        "library" => transform_to_library(project_dir, &structure)?,
        "gen-ai" => transform_to_gen_ai(project_dir, &structure)?,
        "edge-app" => transform_to_edge_app(project_dir, &structure)?,
        "embedded" => transform_to_embedded(project_dir, &structure)?,
        _ => transform_to_template(project_dir, &selected_template, &structure)?,
    }
    
    println!("\n{}", "Project transformation complete!".bold().green());
    println!("Your project has been transformed to: {}", selected_template.cyan());
    
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct ProjectStructure {
    root_path: PathBuf,
    project_name: String,
    _has_database: bool,
    has_libs: bool,
    _has_binaries: bool,
    has_server: bool,
    has_client: bool,
    has_ai: bool,
    has_edge: bool,
    has_embedded: bool,
    is_workspace: bool,
    is_binary: bool,
    is_library: bool,
}

fn analyze_project_structure(project_dir: &Path) -> Result<ProjectStructure> {
    // Check for workspace
    let cargo_toml = read_cargo_toml(project_dir)?;
    let is_workspace = cargo_toml.contains("[workspace]");
    
    // Check for directories
    let has_client = project_dir.join("client").exists();
    let has_server = project_dir.join("server").exists();
    let has_database = project_dir.join("database").exists();
    let has_libs = project_dir.join("libs").exists();
    let has_binaries = project_dir.join("binaries").exists();
    let has_ai = project_dir.join("ai").exists();
    let has_edge = project_dir.join("edge").exists();
    let has_embedded = project_dir.join("embedded").exists();
    
    // Check if it's a binary or library
    let src_dir = project_dir.join("src");
    let is_binary = src_dir.join("main.rs").exists();
    let is_library = src_dir.join("lib.rs").exists();
    
    Ok(ProjectStructure {
        root_path: project_dir.to_path_buf(),
        project_name: project_dir.file_name().unwrap().to_str().unwrap().to_string(),
        _has_database: has_database,
        has_libs,
        _has_binaries: has_binaries,
        has_server,
        has_client,
        has_ai,
        has_edge,
        has_embedded,
        is_workspace,
        is_binary,
        is_library,
    })
}

fn transform_to_template(project_dir: &Path, template_name: &str, structure: &ProjectStructure) -> Result<()> {
    match template_name {
        "minimal" => {
            // No transformation needed for minimal template
            println!("{}", "No transformation needed for minimal template".yellow());
        },
        "library" => {
            transform_to_library(project_dir, structure)?;
        },
        "full-stack" => {
            transform_to_full_stack(project_dir, structure)?;
        },
        "gen-ai" => {
            transform_to_gen_ai(project_dir, structure)?;
        },
        "edge-app" => {
            transform_to_edge_app(project_dir, structure)?;
        },
        "embedded" => {
            transform_to_embedded(project_dir, structure)?;
        },
        _ => {
            println!("{} {}", 
                "Warning:".yellow().bold(), 
                format!("Unknown template '{}', applying minimal transformation", template_name).yellow());
        }
    }
    
    Ok(())
}

fn transform_to_library(project_dir: &Path, structure: &ProjectStructure) -> Result<()> {
    if structure.is_library {
        println!("{}", "Project is already a library".yellow());
        return Ok(());
    }
    
    if structure.is_binary {
        // Convert binary to library
        println!("{}", "Converting binary to library...".blue());
        
        // Read main.rs content
        let main_path = project_dir.join("src").join("main.rs");
        let main_content = fs::read_to_string(&main_path)
            .context("Failed to read main.rs")?;
        
        // Create lib.rs with adapted content
        let lib_content = if main_content.contains("fn main()") {
            // Extract functions and types from main.rs, excluding main()
            let mut lib_lines = Vec::new();
            let mut in_main_fn = false;
            
            for line in main_content.lines() {
                if line.contains("fn main()") {
                    in_main_fn = true;
                    continue;
                }
                
                if in_main_fn {
                    if line.trim() == "}" {
                        in_main_fn = false;
                    }
                    continue;
                }
                
                lib_lines.push(line.to_string());
            }
            
            // Add a public function that wraps the main functionality
            lib_lines.push("".to_string());
            lib_lines.push("/// Main library function".to_string());
            lib_lines.push("pub fn run() {".to_string());
            lib_lines.push("    println!(\"Hello from library!\");".to_string());
            lib_lines.push("}".to_string());
            
            lib_lines.join("\n")
        } else {
            // If main.rs doesn't have a main function, just use it as lib.rs
            main_content
        };
        
        // Write lib.rs
        fs::write(project_dir.join("src").join("lib.rs"), lib_content)
            .context("Failed to write lib.rs")?;
        
        // Update main.rs to use the library
        let new_main_content = format!(
            r#"//! Binary crate that uses the library
use {}::run;

fn main() {{
    run();
}}
"#,
            project_dir.file_name().unwrap().to_str().unwrap()
        );
        
        fs::write(main_path, new_main_content)
            .context("Failed to update main.rs")?;
        
        // Update Cargo.toml
        let cargo_toml = read_cargo_toml(project_dir)?;
        let updated_cargo_toml = if cargo_toml.contains("[lib]") {
            cargo_toml
        } else {
            let mut lines = cargo_toml.lines().collect::<Vec<_>>();
            
            // Find position to insert [lib] section
            let mut insert_pos = 0;
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with("[dependencies]") {
                    insert_pos = i;
                    break;
                }
            }
            
            // Insert [lib] section
            lines.insert(insert_pos, "");
            lines.insert(insert_pos, "[lib]");
            lines.insert(insert_pos, "");
            
            lines.join("\n")
        };
        
        fs::write(project_dir.join("Cargo.toml"), updated_cargo_toml)
            .context("Failed to update Cargo.toml")?;
    } else {
        // Not a binary, create a basic library
        create_directory(project_dir.join("src").to_str().unwrap())?;
        
        fs::write(
            project_dir.join("src").join("lib.rs"),
            "pub fn hello() {\n    println!(\"Hello from library!\");\n}\n"
        )?;
    }
    
    Ok(())
}

fn transform_to_full_stack(project_dir: &Path, structure: &ProjectStructure) -> Result<()> {
    // Initialize workspace if needed
    if !structure.is_workspace {
        init_workspace(project_dir, structure)?;
    }
    
    // Add client if missing
    if !structure.has_client {
        add_client(project_dir)?;
    }
    
    // Add server if missing
    if !structure.has_server {
        add_server(project_dir)?;
    }
    
    // Add libs if missing
    if !structure.has_libs {
        add_libs(project_dir)?;
    }
    
    // Update workspace members
    update_workspace_members(project_dir)?;
    
    Ok(())
}

fn transform_to_gen_ai(project_dir: &Path, structure: &ProjectStructure) -> Result<()> {
    // Initialize workspace if needed
    if !structure.is_workspace {
        init_workspace(project_dir, structure)?;
    }
    
    // Add AI components if missing
    if !structure.has_ai {
        add_ai(project_dir)?;
    }
    
    // Add libs if missing (for shared code)
    if !structure.has_libs {
        add_libs(project_dir)?;
    }
    
    // Update workspace members
    update_workspace_members(project_dir)?;
    
    Ok(())
}

fn transform_to_edge_app(project_dir: &Path, structure: &ProjectStructure) -> Result<()> {
    // Initialize workspace if needed
    if !structure.is_workspace {
        init_workspace(project_dir, structure)?;
    }
    
    // Add edge components if missing
    if !structure.has_edge {
        add_edge(project_dir)?;
    }
    
    // Update workspace members
    update_workspace_members(project_dir)?;
    
    Ok(())
}

fn transform_to_embedded(project_dir: &Path, structure: &ProjectStructure) -> Result<()> {
    // Initialize workspace if needed
    if !structure.is_workspace {
        init_workspace(project_dir, structure)?;
    }
    
    // Add embedded components if missing
    if !structure.has_embedded {
        add_embedded(project_dir)?;
    }
    
    // Update workspace members
    update_workspace_members(project_dir)?;
    
    Ok(())
}

fn init_workspace(project_dir: &Path, structure: &ProjectStructure) -> Result<()> {
    println!("{}", "Initializing workspace...".blue());
    
    // Read existing Cargo.toml
    let cargo_toml = read_cargo_toml(project_dir)?;
    
    // Extract the package name
    let package_name = if let Some(name_line) = cargo_toml.lines()
        .find(|line| line.trim().starts_with("name ="))
    {
        name_line.split('=').nth(1)
            .map(|s| s.trim().trim_matches('"').to_string())
            .unwrap_or_else(|| project_dir.file_name().unwrap().to_str().unwrap().to_string())
    } else {
        project_dir.file_name().unwrap().to_str().unwrap().to_string()
    };
    
    // Create workspace Cargo.toml
    let workspace_toml = format!(
        r#"[workspace]
members = [
    "src"
]

[workspace.dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
thiserror = "1.0"
anyhow = "1.0"
"#
    );
    
    // Create src directory if it doesn't exist
    let src_dir = project_dir.join("src");
    if !src_dir.exists() {
        create_directory(src_dir.to_str().unwrap())?;
    }
    
    // Move current code to src directory if needed
    if structure.is_binary || structure.is_library {
        // Create package Cargo.toml in src
        let src_cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
            package_name
        );
        
        fs::write(src_dir.join("Cargo.toml"), src_cargo_toml)?;
        
        // Update main workspace Cargo.toml
        fs::write(project_dir.join("Cargo.toml"), workspace_toml)?;
    } else {
        // Just update the existing Cargo.toml to be a workspace
        fs::write(project_dir.join("Cargo.toml"), workspace_toml)?;
    }
    
    Ok(())
}

pub fn add_client(project_dir: &Path) -> Result<()> {
    println!("{}", "Adding client components...".blue());
    
    let client_dir = project_dir.join("client");
    create_directory(client_dir.to_str().unwrap())?;
    
    // Create a default Dioxus app
    let app_name = "app";
    let app_dir = client_dir.join(app_name);
    create_directory(app_dir.to_str().unwrap())?;
    create_directory(app_dir.join("src").to_str().unwrap())?;
    
    // Create Cargo.toml for the app
    let app_cargo_toml = r#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = "0.4"
dioxus-web = "0.4"
"#;
    
    fs::write(app_dir.join("Cargo.toml"), app_cargo_toml)?;
    
    // Create main.rs with a basic Dioxus app
    let main_rs = r#"use dioxus::prelude::*;

fn main() {
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            style: "text-align: center; padding: 20px;",
            h1 { "Welcome to Dioxus!" }
            p { "This is a default app created by FerrisUp" }
        }
    })
}
"#;
    
    fs::write(app_dir.join("src").join("main.rs"), main_rs)?;
    
    Ok(())
}

pub fn add_server(project_dir: &Path) -> Result<()> {
    println!("{}", "Adding server components...".blue());
    
    let server_dir = project_dir.join("server");
    create_directory(server_dir.to_str().unwrap())?;
    
    // Create a default API service
    let service_name = "api";
    let service_dir = server_dir.join(service_name);
    create_directory(service_dir.to_str().unwrap())?;
    create_directory(service_dir.join("src").to_str().unwrap())?;
    
    // Create Cargo.toml for the service
    let service_cargo_toml = r#"[package]
name = "api"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.6"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
"#;
    
    fs::write(service_dir.join("Cargo.toml"), service_cargo_toml)?;
    
    // Create main.rs with a basic Axum server
    let main_rs = r#"use axum::{
    routing::get,
    Router, Json
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/hello", get(hello));

    // Run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello from the API!".to_string(),
    })
}
"#;
    
    fs::write(service_dir.join("src").join("main.rs"), main_rs)?;
    
    Ok(())
}

pub fn add_libs(project_dir: &Path) -> Result<()> {
    println!("{}", "Adding library components...".blue());
    
    let libs_dir = project_dir.join("libs");
    create_directory(libs_dir.to_str().unwrap())?;
    
    // Create core, models, and utils libraries
    for lib_name in &["core", "models", "utils"] {
        let lib_dir = libs_dir.join(lib_name);
        create_directory(lib_dir.to_str().unwrap())?;
        create_directory(lib_dir.join("src").to_str().unwrap())?;
        
        // Create Cargo.toml for the library
        let lib_cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
thiserror = "1.0"
"#,
            lib_name
        );
        
        fs::write(lib_dir.join("Cargo.toml"), lib_cargo_toml)?;
        
        // Create lib.rs with basic code
        let lib_rs = format!(
            r#"//! {} library

/// Says hello from the library
pub fn hello() {{
    println!("Hello from {} library!");
}}
"#,
            lib_name, lib_name
        );
        
        fs::write(lib_dir.join("src").join("lib.rs"), lib_rs)?;
    }
    
    Ok(())
}

pub fn add_ai(project_dir: &Path) -> Result<()> {
    println!("{}", "Adding AI components...".blue());
    
    let ai_dir = project_dir.join("ai");
    create_directory(ai_dir.to_str().unwrap())?;
    
    // Create a default AI model
    let model_name = "inference";
    let model_dir = ai_dir.join(model_name);
    create_directory(model_dir.to_str().unwrap())?;
    create_directory(model_dir.join("src").to_str().unwrap())?;
    
    // Create Cargo.toml for the model
    let model_cargo_toml = r#"[package]
name = "inference"
version = "0.1.0"
edition = "2021"

[dependencies]
tch = "0.10"
tract-onnx = "0.19"
tokenizers = "0.13"
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
"#;
    
    fs::write(model_dir.join("Cargo.toml"), model_cargo_toml)?;
    
    // Create lib.rs with a basic AI inference model
    let lib_rs = r#"//! AI Inference Module

use anyhow::Result;

/// AI model for inference
pub struct Model {
    name: String,
}

impl Model {
    /// Create a new AI model
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
    
    /// Run inference on input
    pub fn infer(&self, input: &str) -> Result<String> {
        // This is a placeholder for actual AI inference
        println!("Running inference with model: {}", self.name);
        
        Ok(format!("AI Response to: {}", input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_inference() {
        let model = Model::new("test-model");
        let result = model.infer("Hello AI").unwrap();
        assert!(result.contains("Hello AI"));
    }
}
"#;
    
    fs::write(model_dir.join("src").join("lib.rs"), lib_rs)?;
    
    Ok(())
}

pub fn add_edge(project_dir: &Path) -> Result<()> {
    println!("{}", "Adding edge computing components...".blue());
    
    let edge_dir = project_dir.join("edge");
    create_directory(edge_dir.to_str().unwrap())?;
    
    // Create a default edge worker
    let worker_name = "worker";
    let worker_dir = edge_dir.join(worker_name);
    create_directory(worker_dir.to_str().unwrap())?;
    create_directory(worker_dir.join("src").to_str().unwrap())?;
    
    // Create Cargo.toml for the worker
    let worker_cargo_toml = r#"[package]
name = "worker"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "console", "Document", "Element", "HtmlElement", "Window"
] }
wasm-bindgen-futures = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.4"
"#;
    
    fs::write(worker_dir.join("Cargo.toml"), worker_cargo_toml)?;
    
    // Create lib.rs with a basic Cloudflare Worker
    let lib_rs = r#"use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn greeting(name: &str) -> String {
    console::log_1(&"Called from WASM".into());
    format!("Hello, {}! Welcome to Ferris Edge", name)
}

#[wasm_bindgen(start)]
pub fn start() {
    console::log_1(&"Edge worker initialized".into());
}
"#;
    
    fs::write(worker_dir.join("src").join("lib.rs"), lib_rs)?;
    
    Ok(())
}

pub fn add_embedded(project_dir: &Path) -> Result<()> {
    println!("{}", "Adding embedded systems components...".blue());
    
    let embedded_dir = project_dir.join("embedded");
    create_directory(embedded_dir.to_str().unwrap())?;
    
    // Create a default embedded device
    let device_name = "device";
    let device_dir = embedded_dir.join(device_name);
    create_directory(device_dir.to_str().unwrap())?;
    create_directory(device_dir.join("src").to_str().unwrap())?;
    
    // Create Cargo.toml for the device
    let device_cargo_toml = r#"[package]
name = "device"
version = "0.1.0"
edition = "2021"

[dependencies]
embedded-hal = "0.2"
panic-halt = "0.2"
cortex-m = "0.7"
cortex-m-rt = "0.7"

[[bin]]
name = "device"
test = false
bench = false
"#;
    
    fs::write(device_dir.join("Cargo.toml"), device_cargo_toml)?;
    
    // Create main.rs with a basic embedded program
    let main_rs = r#"//! Basic embedded device program
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Embedded device initialization would go here
    
    loop {
        // Main device loop
    }
}
"#;
    
    fs::write(device_dir.join("src").join("main.rs"), main_rs)?;
    
    Ok(())
}
