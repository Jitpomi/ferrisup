use anyhow::{Result, Context};
use colored::Colorize;
use std::path::Path;
use std::fs;
use regex::Regex;

// Function to fix imports in a newly created component
pub fn fix_component_imports(component_dir: &Path, component_name: &str, project_name: &str) -> Result<()> {
    println!("{}", format!("Fixing imports in component: {}", component_name).blue());
    
    // Get all Rust files in the component directory
    let src_dir = component_dir.join("src");
    
    if !src_dir.exists() {
        return Ok(());
    }
    
    // Process all Rust files in the src directory
    process_directory(&src_dir, component_name, project_name)?;
    
    Ok(())
}

// Helper function to recursively process all Rust files in a directory
fn process_directory(dir_path: &Path, component_name: &str, project_name: &str) -> Result<()> {
    if !dir_path.exists() || !dir_path.is_dir() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively process subdirectories
            process_directory(&path, component_name, project_name)?;
        } else if path.is_file() {
            // Process Rust files
            if let Some(extension) = path.extension() {
                if extension == "rs" {
                    fix_imports_in_file(&path, component_name, project_name)?;
                }
            }
        }
    }
    
    Ok(())
}

// Helper function to fix imports in a single file
fn fix_imports_in_file(file_path: &Path, component_name: &str, project_name: &str) -> Result<()> {
    // Read the file content
    let content = fs::read_to_string(file_path)?;
    
    // Create the new package name
    let new_package_name = format!("{}_{}", project_name, component_name);
    
    // Replace imports like "use client::*;" with "use app_client::*;"
    let re = Regex::new(&format!(r"use\s+{}(::|\s+)", regex::escape(component_name)))
        .context("Failed to create regex")?;
        
    let updated_content = re.replace_all(&content, format!("use {}{}", new_package_name, "$1"));
    
    // Write updated content back to file
    fs::write(file_path, updated_content.as_bytes())?;
    println!("  Fixed imports in: {}", file_path.display());
    
    Ok(())
}
