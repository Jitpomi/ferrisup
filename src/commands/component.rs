use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use std::path::{Path, PathBuf};
use std::fs;

use crate::utils::{read_cargo_toml, update_workspace_members};

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
        let options = ["add", "remove", "list"];
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
    // Get workspace structure
    let cargo_content = read_cargo_toml(project_dir)?;
    let is_workspace = cargo_content.contains("[workspace]");
    
    // The menu shown should be based on whether the project is a workspace
    // If component_type is provided, we'll pass it to the appropriate transform function
    // Otherwise, the transform function will handle the component type selection
    
    // Save current directory
    let current_dir = std::env::current_dir()?;
    
    // Change to project directory
    std::env::set_current_dir(project_dir)?;
    
    // Store the component type for success message
    let component_name = component_type.unwrap_or("new").to_string();
    
    // Call the appropriate transform command function based on workspace status
    // This ensures we get the correct menu based on whether it's a workspace
    let result = if is_workspace {
        // For workspace projects - shows all component types
        crate::commands::transform::add_component(project_dir)
    } else {
        // For non-workspace projects - shows only module-compatible components
        crate::commands::transform::add_component_without_workspace(project_dir)
    };
    
    // Change back to original directory
    std::env::set_current_dir(current_dir)?;
    
    // Handle any errors
    if let Err(e) = result {
        println!("{} {}", "Error adding component:".red().bold(), e);
        return Err(anyhow::anyhow!("Failed to add component"));
    }
    
    // Update workspace if needed
    if is_workspace {
        update_workspace_members(project_dir)?;
    }
    
    println!("{} {} {}", 
        "Successfully added".green(),
        component_name.green(),
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
    
    println!("{}", "Components:".green());
    for component in components {
        println!("  - {}", component);
    }
    
    Ok(())
}

/// Discover components in a project
fn discover_components(project_dir: &Path) -> Result<Vec<String>> {
    let entries = fs::read_dir(project_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| {
            // Check if it has a Cargo.toml file
            entry.path().join("Cargo.toml").exists() && 
            // Exclude common non-component directories
            !entry.file_name().to_string_lossy().starts_with('.')
        })
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    
    Ok(entries)
}

// All component implementation functions have been removed as we now delegate to transform command's functions
// This includes database, AI, edge, embedded, and library component implementations
// The component command now uses the same component types and creation logic as the transform command
