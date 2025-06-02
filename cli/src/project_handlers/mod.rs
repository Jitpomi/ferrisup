pub mod traits;
pub mod cli_handler;
pub mod template_handler;

use std::path::Path;
use serde_json::Value;
use std::io;
use std::fs;
use traits::ProjectHandler;
use cli_handler::CliProjectHandler;
use template_handler::TemplateProjectHandler;

// Get all registered project handlers
pub fn get_handlers() -> Vec<Box<dyn ProjectHandler>> {
    let mut handlers: Vec<Box<dyn ProjectHandler>> = Vec::new();
    
    // Add CLI Handlers
    
    // Embassy CLI Handler
    handlers.push(Box::new(CliProjectHandler::new(
        "Embassy",
        "Embedded systems with the Embassy framework", 
        vec!["embedded-embassy".to_string()],
        "cargo embassy",
        |project_name, _target_dir, variables| {
            let mut args = vec!["init".to_string()];
            
            // Map mcu_target to chip
            if let Some(target) = variables.get("mcu_target").and_then(|t| t.as_str()) {
                let chip = match target {
                    "esp32" => "esp32c3",
                    _ => target
                };
                args.push("--chip".to_string());
                args.push(chip.to_string());
            }
            
            args.push(project_name.to_string());
            args
        },
        |project_name, _variables| {
            vec![
                format!("🚀 Navigate to your project: cd {}", project_name),
                "📝 Review the generated code".to_string(),
                "🔧 Build the project: cargo build --release".to_string(),
                "▶️ Flash the project: cargo run --release".to_string(),
                "📚 Read the Embassy documentation: https://embassy.dev".to_string()
            ]
        },
        Some("cargo install cargo-embassy".to_string()),
        Some("cargo embassy --version".to_string()),
    )));
    
    // Dioxus CLI Handler
    handlers.push(Box::new(CliProjectHandler::new(
        "Dioxus",
        "Cross-platform UI toolkit for Rust",
        vec!["client-dioxus".to_string(), "dioxus".to_string()],
        "dioxus create",
        |project_name, _target_dir, variables| {
            let mut args = vec![project_name.to_string()];
            
            if let Some(platform) = variables.get("platform").and_then(|p| p.as_str()) {
                args.push("--platform".to_string());
                args.push(platform.to_string());
            } else {
                args.push("--platform".to_string());
                args.push("web".to_string());
            }
            
            args
        },
        |project_name, variables| {
            let platform = variables.get("platform").and_then(|p| p.as_str()).unwrap_or("web");
            
            match platform {
                "web" => vec![
                    format!("🚀 Navigate to your project: cd {}", project_name),
                    "📝 Review the generated code".to_string(),
                    "🔧 Build the project: dx serve".to_string(),
                    "🌐 View your app at http://localhost:8080".to_string(),
                ],
                "desktop" => vec![
                    format!("🚀 Navigate to your project: cd {}", project_name),
                    "📝 Review the generated code".to_string(),
                    "🔧 Build the project: dx build --release".to_string(),
                    "▶️ Run the project: dx serve".to_string(),
                ],
                _ => vec![
                    format!("🚀 Navigate to your project: cd {}", project_name),
                    "📝 Review the generated code".to_string(),
                    "🔧 Build the project: dx build".to_string(),
                    "▶️ Run the project: dx serve".to_string(),
                ]
            }
        },
        Some("cargo install dioxus-cli".to_string()),
        Some("dioxus --version".to_string()),
    )));
    
    // Tauri CLI Handler
    handlers.push(Box::new(CliProjectHandler::new(
        "Tauri",
        "Build desktop applications with web technologies",
        vec!["client-tauri".to_string(), "tauri".to_string()],
        "cargo tauri",
        |project_name, _target_dir, _variables| {
            vec!["init".to_string(), "--app".to_string(), project_name.to_string()]
        },
        |project_name, _variables| {
            vec![
                format!("🚀 Navigate to your project: cd {}", project_name),
                "📝 Review the generated code".to_string(),
                "🔧 Build the project: cargo tauri dev".to_string(),
                "📦 Package for distribution: cargo tauri build".to_string(),
            ]
        },
        Some("cargo install tauri-cli".to_string()),
        Some("cargo tauri --version".to_string()),
    )));
    
    // Add Template Handlers
    
    // Full Stack Templates
    handlers.push(Box::new(TemplateProjectHandler::new(
        "Full Stack",
        "Complete application with client, server, and shared libraries",
        vec!["full-stack".to_string()]
    )));
    
    // Server Templates
    handlers.push(Box::new(TemplateProjectHandler::new(
        "Server",
        "Web server with API endpoints",
        vec!["server".to_string(), "axum".to_string(), "actix".to_string(), "poem".to_string()]
    )));
    
    // Data Science Templates
    handlers.push(Box::new(TemplateProjectHandler::new(
        "Data Science",
        "Data science and machine learning projects",
        vec!["data-science".to_string(), "burn".to_string(), "linfa".to_string()]
    )));
    
    // Edge Templates
    handlers.push(Box::new(TemplateProjectHandler::new(
        "Edge",
        "WebAssembly-based applications for edge computing",
        vec!["edge".to_string(), "edge-app".to_string()]
    )));
    
    // Standard Embedded Templates
    handlers.push(Box::new(TemplateProjectHandler::new(
        "Embedded",
        "Embedded systems firmware for microcontrollers",
        vec!["embedded".to_string()]
    )));
    
    // Serverless Templates
    handlers.push(Box::new(TemplateProjectHandler::new(
        "Serverless",
        "Serverless functions for cloud deployment",
        vec!["serverless".to_string()]
    )));
    
    // Generic Template Handler (fallback)
    handlers.push(Box::new(TemplateProjectHandler::new(
        "Generic",
        "Generic template handler for all other templates",
        vec!["minimal".to_string(), "library".to_string(), "iot-device".to_string(), "ml-pipeline".to_string()]
    )));
    
    handlers
}

// Find a handler for the given template name
pub fn find_handler(template_name: &str, variables: &Value) -> Option<Box<dyn ProjectHandler>> {
    for handler in get_handlers() {
        if handler.can_handle(template_name, variables) {
            return Some(handler);
        }
    }
    
    None
}

// Helper function to recursively copy a directory
pub fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_dir_all(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    
    Ok(())
}
