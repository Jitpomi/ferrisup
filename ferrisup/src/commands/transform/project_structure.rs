use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use toml_edit::DocumentMut;

// Structure to hold project analysis
#[derive(Debug)]
pub struct ProjectStructure {
    pub is_workspace: bool,
    pub is_binary: bool,
    pub project_name: String,
}

// Function to analyze project structure
pub fn analyze_project_structure(project_dir: &Path) -> Result<ProjectStructure> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;

    // Parse Cargo.toml
    let cargo_doc = cargo_toml_content
        .parse::<DocumentMut>()
        .context("Failed to parse Cargo.toml as TOML")?;

    // Check if it's a workspace
    let is_workspace = cargo_doc.get("workspace").is_some();

    // Check if it's a binary or library
    let is_binary = if let Some(_lib) = cargo_doc.get("lib") {
        false
    } else {
        // Check for bin target or assume binary if no lib section
        cargo_doc.get("bin").is_some() || !cargo_doc.get("package").is_none()
    };

    // Get project name
    let project_name = if let Some(package) = cargo_doc.get("package") {
        if let Some(name) = package.get("name") {
            name.as_str().unwrap_or("unknown").to_string()
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    Ok(ProjectStructure {
        is_workspace,
        is_binary,
        project_name,
    })
}

// Function to detect framework from source files
pub fn detect_framework(src_paths: &[&Path]) -> Option<String> {
    for src_path in src_paths {
        if src_path.exists() {
            if let Ok(content) = fs::read_to_string(src_path) {
                // Try to detect the framework from imports
                if content.contains("use poem") {
                    return Some("poem".to_string());
                } else if content.contains("use axum") {
                    return Some("axum".to_string());
                } else if content.contains("use actix_web") {
                    return Some("actix".to_string());
                } else if content.contains("use leptos") {
                    return Some("leptos".to_string());
                } else if content.contains("use dioxus") {
                    return Some("dioxus".to_string());
                }
            }
        }
    }
    None
}

// Map component type to template
pub fn map_component_to_template(component_type: &str) -> &str {
    match component_type {
        "client_old" => "client_old",
        "server" => "server",
        "ferrisup_common" => "library", // For ferrisup_common components, we use "library" as the template
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        _ => "server", // Default to server if unknown
    }
}
