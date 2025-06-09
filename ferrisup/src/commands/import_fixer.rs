use anyhow::{Result, Context};
use colored::Colorize;
use std::path::Path;
use std::fs;
use regex::Regex;
use walkdir::WalkDir;
use toml_edit::{Document, value};

/// Fixes imports in a component after the package name has been updated
/// 
/// This function recursively searches through all Rust files in the component
/// and updates import statements to use the new package name.
/// 
/// For example, if a component was created with name "client" but the package
/// was renamed to "app_client" in Cargo.toml, this function will update all
/// imports from "use client::*" to "use app_client::*".
pub fn fix_component_imports(component_dir: &Path, component_name: &str, project_name: &str) -> Result<()> {
    println!("{}", format!("Fixing imports in component: {}", component_name).blue());
    
    // First, update the package name in Cargo.toml
    let cargo_toml_path = component_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml_path)
            .context("Failed to read component Cargo.toml")?;
        
        let mut cargo_doc = cargo_content.parse::<Document>()
            .context("Failed to parse component Cargo.toml")?;
        
        // Use the component name directly instead of {project_name}_{component_name}
        let new_package_name = component_name.to_string();
        
        if let Some(package) = cargo_doc.get_mut("package") {
            if let Some(name) = package.get_mut("name") {
                if let Some(current_name) = name.as_str() {
                    // Only update if the current name is different from what we want
                    if current_name != new_package_name {
                        *name = value(new_package_name.clone());
                        
                        // Write updated Cargo.toml
                        fs::write(&cargo_toml_path, cargo_doc.to_string())
                            .context("Failed to write updated Cargo.toml")?;
                        
                        println!("{}", format!("  Updated package name in Cargo.toml to: {}", new_package_name).blue());
                    }
                }
            }
        }
    }
    
    // Get all Rust files in the component directory
    let src_dir = component_dir.join("src");
    
    if !src_dir.exists() {
        return Ok(());
    }
    
    // Process all Rust files in the src directory recursively
    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()) {
            
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "rs" {
                // Read file content
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                
                // Replace imports like "use client::*;" with "use app_client::*;"
                // Also handle "use unknown_client::*;" pattern
                let re_component = match Regex::new(&format!(r"use\s+{}(::|\s+)", regex::escape(component_name))) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                
                let re_unknown = match Regex::new(&format!(r"use\s+unknown_{}(::|\s+)", regex::escape(component_name))) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                
                let new_package_name = format!("{}_{}", project_name, component_name);
                
                // Apply both replacements
                let updated_content1 = re_component.replace_all(&content, format!("use {}{}", new_package_name, "$1"));
                let updated_content2 = re_unknown.replace_all(&updated_content1, format!("use {}{}", new_package_name, "$1"));
                
                // Write updated content back to file if changes were made
                if content != updated_content2 {
                    if let Err(_) = fs::write(path, updated_content2.as_bytes()) {
                        continue;
                    }
                    println!("  Fixed imports in: {}", path.display());
                }
            }
        }
    }
    
    Ok(())
}
