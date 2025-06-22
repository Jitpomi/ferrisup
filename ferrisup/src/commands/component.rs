use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use toml_edit::DocumentMut;

use ferrisup_common::cargo::{read_cargo_toml, update_workspace_members};

type UsageSummary = HashMap<String, HashMap<String, Vec<(usize, String)>>>;

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
    
    // Check if this is a workspace
    let cargo_content = read_cargo_toml(project_dir)?;
    let is_workspace = cargo_content.contains("[workspace]");
    
    // Before proceeding, check if the component is being used in other components
    if is_workspace {
        let usage_summary = check_component_usage(project_dir, &component_path, &components)?;
        
        if !usage_summary.is_empty() {
            println!("{}", "⚠️  Component is being used in other components:".yellow());
            println!();
            
            // Print usage summary
            for (component, files) in &usage_summary {
                println!("In {} :", component.cyan());
                
                // Group by file and then by type (import, definite usage, potential usage)
                for (file, lines) in files {
                    println!("  📄 {}", file);
                    
                    // First show imports
                    let imports: Vec<_> = lines.iter()
                        .filter(|(_, content)| content.starts_with("use "))
                        .collect();
                    
                    if !imports.is_empty() {
                        println!("    Imports:");
                        for (line_num, content) in &imports {
                            println!("      Line {}: {}", line_num, content);
                        }
                    }
                    
                    // Then show direct references
                    let direct_refs: Vec<_> = lines.iter()
                        .filter(|(_, content)| content.starts_with("[DIRECT REFERENCE]"))
                        .collect();
                    
                    if !direct_refs.is_empty() {
                        println!("    Direct References:");
                        for (line_num, content) in &direct_refs {
                            let clean_content = content.replace("[DIRECT REFERENCE] ", "");
                            println!("      Line {}: {}", line_num, clean_content);
                        }
                    }
                    
                    // Then show potential usage
                    let potential: Vec<_> = lines.iter()
                        .filter(|(_, content)| content.starts_with("[POTENTIAL USAGE]"))
                        .collect();
                    
                    if !potential.is_empty() {
                        println!("    Potential Usage:");
                        for (line_num, content) in &potential {
                            let clean_content = content.replace("[POTENTIAL USAGE] ", "");
                            println!("      Line {}: {}", line_num, clean_content);
                        }
                    }
                    
                    // Then show definite usage
                    let definite: Vec<_> = lines.iter()
                        .filter(|(_, content)| !content.starts_with("use ") && 
                                              !content.starts_with("[POTENTIAL USAGE]") &&
                                              !content.starts_with("[DIRECT REFERENCE]"))
                        .collect();
                    
                    if !definite.is_empty() {
                        println!("    Definite Usage:");
                        for (line_num, content) in &definite {
                            println!("      Line {}: {}", line_num, content);
                        }
                    }
                }
                println!();
            }
            
            println!("{}", "You must manually remove these references before removing the component.".yellow());
            println!("{}", "Component removal has been cancelled.".red());
            return Ok(());
        }
    }
    
    // Confirm removal
    let confirm = Confirm::new()
        .with_prompt(format!("Remove component {}?", component_path))
        .default(false)
        .interact()?;
    
    if !confirm {
        println!("Operation cancelled");
        return Ok(());
    }
    
    if is_workspace {
        // Remove component from other components' dependencies
        remove_component_dependencies(project_dir, &component_path)?;
        
        // Update metadata to remove the component
        remove_component_from_metadata(project_dir, &component_path)?;
    }
    
    // Remove the component directory
    let full_path = project_dir.join(&component_path);
    fs::remove_dir_all(&full_path)
        .context(format!("Failed to remove {}", full_path.display()))?;
    
    println!("{} {}", "Successfully removed component:".green(), component_path);
    
    // Update workspace members list if needed
    if is_workspace {
        // First update the workspace members list directly to ensure the removed component is gone
        update_workspace_members_after_removal(project_dir, &component_path)?;
        
        // Then run the standard update to discover any new members
        update_workspace_members(project_dir)?;
    }
    
    Ok(())
}

/// Remove a component from the .ferrisup/metadata.toml file
fn remove_component_from_metadata(project_dir: &Path, component_name: &str) -> Result<()> {
    let ferrisup_dir = project_dir.join(".ferrisup");
    if !ferrisup_dir.exists() {
        // No metadata directory, nothing to update
        return Ok(());
    }
    
    let metadata_path = ferrisup_dir.join("metadata.toml");
    if !metadata_path.exists() {
        // No metadata file, nothing to update
        return Ok(());
    }
    
    // Read and parse the metadata file
    let metadata_content = fs::read_to_string(&metadata_path)?;
    let mut metadata_doc = metadata_content
        .parse::<DocumentMut>()
        .context("Failed to parse metadata.toml")?;
    
    // Remove the component entry from metadata
    let component_key = format!("component.{}", component_name);
    if metadata_doc.contains_key(&component_key) {
        metadata_doc.remove(&component_key);
        println!("{} {}", "Updated".green(), "metadata.toml to remove component".cyan());
        
        // Write the updated metadata back to the file
        fs::write(metadata_path, metadata_doc.to_string())?;
    }
    
    Ok(())
}

/// Remove component dependencies from other components in the workspace
fn remove_component_dependencies(project_dir: &Path, component_name: &str) -> Result<()> {
    // Read the workspace Cargo.toml
    let workspace_cargo_path = project_dir.join("Cargo.toml");
    if !workspace_cargo_path.exists() {
        return Ok(());
    }
    
    let workspace_cargo_content = fs::read_to_string(&workspace_cargo_path)?;
    let mut workspace_doc = workspace_cargo_content
        .parse::<DocumentMut>()
        .context("Failed to parse workspace Cargo.toml")?;
    
    // Remove from workspace.dependencies if it exists
    if let Some(workspace) = workspace_doc.get_mut("workspace") {
        if let Some(workspace_table) = workspace.as_table_mut() {
            if let Some(deps) = workspace_table.get_mut("dependencies") {
                if let Some(deps_table) = deps.as_table_mut() {
                    // Check if the component is in workspace dependencies
                    if deps_table.contains_key(component_name) {
                        deps_table.remove(component_name);
                        println!("{} {}", "Removed".green(), 
                            format!("'{}' from workspace.dependencies", component_name).cyan());
                    }
                }
            }
        }
    }
    
    // Write updated workspace Cargo.toml
    fs::write(&workspace_cargo_path, workspace_doc.to_string())?;
    
    // Now find all other component directories and remove the dependency
    // from each component's Cargo.toml
    let components = discover_components(project_dir)?;
    
    println!("{} {}", "Checking".blue(), 
        format!("for dependencies and imports in other components").cyan());
    
    for comp in components {
        // Skip the component being removed
        if comp == component_name {
            continue;
        }
        
        let component_cargo_path = project_dir.join(&comp).join("Cargo.toml");
        if component_cargo_path.exists() {
            // Remove the dependency from this component
            remove_dependency_from_component(&component_cargo_path, component_name)?;
            
            // Also remove any import statements from source files
            remove_imports_from_component(&project_dir.join(&comp), component_name)?;
        }
    }
    
    Ok(())
}

/// Remove a dependency from a component's Cargo.toml
fn remove_dependency_from_component(cargo_path: &Path, dependency_name: &str) -> Result<()> {
    if !cargo_path.exists() {
        return Ok(());
    }
    
    let cargo_content = fs::read_to_string(cargo_path)?;
    let mut cargo_doc = cargo_content
        .parse::<DocumentMut>()
        .context(format!("Failed to parse {}", cargo_path.display()))?;
    
    let mut updated = false;
    
    // Check standard dependencies
    if let Some(deps) = cargo_doc.get_mut("dependencies") {
        if let Some(deps_table) = deps.as_table_mut() {
            if deps_table.contains_key(dependency_name) {
                deps_table.remove(dependency_name);
                updated = true;
            }
        }
    }
    
    // Check dev-dependencies
    if let Some(deps) = cargo_doc.get_mut("dev-dependencies") {
        if let Some(deps_table) = deps.as_table_mut() {
            if deps_table.contains_key(dependency_name) {
                deps_table.remove(dependency_name);
                updated = true;
            }
        }
    }
    
    // Check build-dependencies
    if let Some(deps) = cargo_doc.get_mut("build-dependencies") {
        if let Some(deps_table) = deps.as_table_mut() {
            if deps_table.contains_key(dependency_name) {
                deps_table.remove(dependency_name);
                updated = true;
            }
        }
    }
    
    if updated {
        // Write updated Cargo.toml
        fs::write(cargo_path, cargo_doc.to_string())?;
        println!("{} {}", "Removed".green(), 
            format!("'{}' dependency from {}", dependency_name, cargo_path.file_name().unwrap().to_string_lossy()).cyan());
    }
    
    Ok(())
}

/// Remove import statements referencing the removed component
fn remove_imports_from_component(component_dir: &Path, removed_component: &str) -> Result<()> {
    // Process both src/main.rs and src/lib.rs if they exist
    let src_dir = component_dir.join("src");
    if !src_dir.exists() {
        return Ok(());
    }
    
    // Check main.rs
    let main_rs = src_dir.join("main.rs");
    if main_rs.exists() {
        remove_imports_from_file(&main_rs, removed_component)?;
    }
    
    // Check lib.rs
    let lib_rs = src_dir.join("lib.rs");
    if lib_rs.exists() {
        remove_imports_from_file(&lib_rs, removed_component)?;
    }
    
    // Recursively check all .rs files in the src directory
    visit_rust_files(&src_dir, removed_component)?;
    
    Ok(())
}

/// Check if a component is being used in other components
/// Returns a map of component name -> (file path -> (line number, line content))
fn check_component_usage(project_dir: &Path, component_to_check: &str, all_components: &[String]) -> Result<UsageSummary> {
    let mut usage_summary: UsageSummary = HashMap::new();
    
    println!("{} {}", "Checking for usage of".blue(), component_to_check.cyan());
    
    // For each component (except the one being removed)
    for component in all_components {
        if component == component_to_check {
            continue;
        }
        
        println!("{} {}", "Scanning component".blue(), component.cyan());
        
        let component_dir = project_dir.join(component);
        let src_dir = component_dir.join("src");
        
        if !src_dir.exists() || !src_dir.is_dir() {
            println!("{} {}", "No src directory found in".yellow(), component.cyan());
            continue;
        }
        
        // We'll use visit_rust_files_for_usage to check all .rs files including main.rs and lib.rs
        visit_rust_files_for_usage(&src_dir, component_to_check, component, project_dir, &mut usage_summary)?;
    }
    
    Ok(usage_summary)
}

/// Check a file for actual usage of a component (beyond imports)
fn check_file_for_usage(
    file_path: &Path, 
    component_to_check: &str, 
    component_name: &str,
    project_dir: &Path,
    usage_summary: &mut UsageSummary
) -> Result<()> {
    if !file_path.exists() {
        return Ok(());
    }
    
    println!("  Checking file: {}", file_path.display());
    
    let content = fs::read_to_string(file_path)?;
    
    // Check if the file imports the component
    let snake_case = component_to_check.replace('-', "_");
    let import_patterns = [
        format!("use {}::*", component_to_check),
        format!("use {}::", component_to_check),
        // Also check for snake_case variant
        format!("use {}_::", component_to_check),
        format!("use {}_::*", component_to_check),
        // Check for snake case variant
        format!("use {}::*", snake_case),
        format!("use {}::", snake_case),
    ];
    
    let mut has_import = false;
    
    // First pass: find all import lines
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        
        // Skip empty lines or comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        
        // Check for import statements
        if trimmed.starts_with("use ") {
            for pattern in &import_patterns {
                if trimmed.contains(pattern) {
    
                    has_import = true;
                    
                    // Found import, add to summary - but check for duplicates first
                    let file_relative = file_path.strip_prefix(project_dir.join(component_name))
                        .unwrap_or(file_path)
                        .to_string_lossy()
                        .to_string();
                    
                    let entry = (line_num + 1, line.to_string());
                    
                    // Get or create the entry for this component and file
                    let file_entries = usage_summary
                        .entry(component_name.to_string())
                        .or_default()
                        .entry(file_relative.clone())
                        .or_default();
                    
                    // Only add if this exact entry doesn't already exist
                    if !file_entries.iter().any(|(num, content)| *num == entry.0 && content == &entry.1) {
                        file_entries.push(entry);
                    }
                }
            }
        }
    }
    
    // Always check for actual usage, regardless of imports
    // Second pass: check for actual usage
    let snake_case = component_to_check.replace('-', "_");
    
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        
        // Skip empty lines, comments, or import lines
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("use ") {
            continue;
        }
        
        // Check for actual usage of the component - both original and snake case
        if trimmed.contains(&component_to_check) || trimmed.contains(&snake_case) {
            println!("    Found usage on line {}: {}", line_num + 1, trimmed);
            
            // Found usage, add to summary
            let file_relative = file_path.strip_prefix(project_dir.join(component_name))
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();
            
            usage_summary
                .entry(component_name.to_string())
                .or_default()
                .entry(file_relative)
                .or_default()
                .push((line_num + 1, line.to_string()));
        }
        
        // Check for actual component name usage in code
        let component_name_snake = component_to_check.replace('-', "_");
        if trimmed.contains(&component_name_snake) || trimmed.contains(component_to_check) {
            // Found direct reference to component name
            let file_relative = file_path.strip_prefix(project_dir.join(component_name))
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();
            
            let entry = (line_num + 1, format!("[DIRECT REFERENCE] {}", line.trim()));
            
            // Get or create the entry for this component and file
            let file_entries = usage_summary
                .entry(component_name.to_string())
                .or_default()
                .entry(file_relative.clone())
                .or_default();
            
            // Only add if this exact entry doesn't already exist
            if !file_entries.iter().any(|(num, content)| *num == entry.0 && content == &entry.1) {
                file_entries.push(entry);
            }
            continue;
        }
        
        // Also check for function calls that might be from the component
        // but only if we have an import AND the line contains a function call
        if has_import {
            // Get exported functions from the component by checking its lib.rs or main.rs
            let component_dir = project_dir.join(component_to_check);
            let lib_rs = component_dir.join("src/lib.rs");
            let main_rs = component_dir.join("src/main.rs");
            
            // Try to find exported functions
            let mut exported_functions = Vec::new();
            
            // Check lib.rs for exported functions
            if lib_rs.exists() {
                if let Ok(content) = fs::read_to_string(&lib_rs) {
                    // Very basic function detection - could be improved
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with("pub fn ") {
                            if let Some(name_end) = line["pub fn ".len()..].find('(') {
                                let fn_name = &line["pub fn ".len()..][..name_end].trim();
                                exported_functions.push(fn_name.to_string());
                            }
                        }
                    }
                }
            }
            
            // Also check main.rs for exported functions
            if main_rs.exists() && exported_functions.is_empty() {
                if let Ok(content) = fs::read_to_string(&main_rs) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with("pub fn ") {
                            if let Some(name_end) = line["pub fn ".len()..].find('(') {
                                let fn_name = &line["pub fn ".len()..][..name_end].trim();
                                exported_functions.push(fn_name.to_string());
                            }
                        }
                    }
                }
            }
            
            // Check if line contains any of the exported functions
            let words: Vec<&str> = trimmed.split(|c: char| !c.is_alphanumeric() && c != '_').collect();
            for word in words {
                if !word.is_empty() && !word.chars().next().unwrap().is_uppercase() {
                    // Check if this word is one of the exported functions
                    if exported_functions.contains(&word.to_string()) ||
                       // Or if it looks like a function call and we couldn't determine exports
                       (exported_functions.is_empty() && 
                        (trimmed.contains(&format!("{}{{", word)) || 
                         trimmed.contains(&format!("{} {{", word)) ||
                         trimmed.contains(&format!("{}(", word)) ||
                         trimmed.contains(&format!("{}!", word)))) {
                        
                        // Add to summary as potential usage
                        let file_relative = file_path.strip_prefix(project_dir.join(component_name))
                            .unwrap_or(file_path)
                            .to_string_lossy()
                            .to_string();
                        
                        let entry = (line_num + 1, format!("[POTENTIAL USAGE] {}", line.trim()));
                        
                        // Get or create the entry for this component and file
                        let file_entries = usage_summary
                            .entry(component_name.to_string())
                            .or_default()
                            .entry(file_relative.clone())
                            .or_default();
                        
                        // Only add if this exact entry doesn't already exist
                        if !file_entries.iter().any(|(num, content)| *num == entry.0 && content == &entry.1) {
                            file_entries.push(entry);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Visit all Rust files in a directory and check for component usage
fn visit_rust_files_for_usage(
    dir: &Path, 
    component_to_check: &str, 
    component_name: &str,
    project_dir: &Path,
    usage_summary: &mut UsageSummary
) -> Result<()> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively visit subdirectories
            visit_rust_files_for_usage(&path, component_to_check, component_name, project_dir, usage_summary)?;
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            // Check .rs files
            check_file_for_usage(&path, component_to_check, component_name, project_dir, usage_summary)?;
        }
    }
    
    Ok(())
}

/// Visit all Rust files in a directory and its subdirectories
fn visit_rust_files(dir: &Path, removed_component: &str) -> Result<()> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively visit subdirectories
            visit_rust_files(path.as_path(), removed_component)?;
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            // Process Rust files
            remove_imports_from_file(path.as_path(), removed_component)?;
        }
    }
    
    Ok(())
}

/// Update workspace members list after component removal
fn update_workspace_members_after_removal(project_dir: &Path, removed_component: &str) -> Result<()> {
    let workspace_cargo_path = project_dir.join("Cargo.toml");
    if !workspace_cargo_path.exists() {
        return Ok(());
    }
    
    let workspace_cargo_content = fs::read_to_string(&workspace_cargo_path)?;
    let mut workspace_doc = workspace_cargo_content
        .parse::<DocumentMut>()
        .context("Failed to parse workspace Cargo.toml")?;
    
    // Get the workspace members array
    if let Some(workspace) = workspace_doc.get_mut("workspace") {
        if let Some(workspace_table) = workspace.as_table_mut() {
            if let Some(members) = workspace_table.get_mut("members") {
                if let Some(members_array) = members.as_array_mut() {
                    // Check if the component is in the members list
                    let original_len = members_array.len();
                    
                    // Remove the component from the members list
                    members_array.retain(|member| {
                        if let Some(member_str) = member.as_str() {
                            member_str != removed_component
                        } else {
                            true
                        }
                    });
                    
                    if members_array.len() < original_len {
                        println!("{} {}", "Removed".green(), 
                            format!("'{}' from workspace members", removed_component).cyan());
                    }
                }
            }
        }
    }
    
    // Write updated workspace Cargo.toml
    fs::write(&workspace_cargo_path, workspace_doc.to_string())?;
    
    Ok(())
}

/// Remove import statements from a Rust file
fn remove_imports_from_file(file_path: &Path, removed_component: &str) -> Result<()> {
    if !file_path.exists() {
        return Ok(());
    }
    
    let content = fs::read_to_string(file_path)?;
    
    // Look for import statements that reference the removed component
    // Match patterns like:
    // use removed_component::*;
    // use removed_component::some_module;
    // use crate::removed_component::*;
    // use {removed_component, other_component};
    // use component_with_removed_component_as_prefix;
    
    let mut updated_content = String::new();
    let mut updated = false;
    
    // Create various patterns to match different import styles
    let patterns = [
        format!("use {}::*", removed_component),        // use removed_component::*
    ];
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines or comments
        if trimmed.is_empty() || trimmed.starts_with("//") {
            updated_content.push_str(line);
            updated_content.push('\n');
            continue;
        }
        
        // Check if this line is importing from the removed component
        let mut should_skip = false;
        
        if trimmed.starts_with("use ") || trimmed.starts_with("pub use ") {
            // Check against our patterns
            for pattern in &patterns {
                if trimmed.contains(pattern) || 
                   // Also check for exact component name in braces import
                   (trimmed.contains("{") && trimmed.contains("}") && 
                    trimmed.contains(&format!(", {}, ", removed_component)) || 
                    trimmed.contains(&format!("{{{}", removed_component)) || 
                    trimmed.contains(&format!("{}, ", removed_component)) || 
                    trimmed.contains(&format!(", {}}}", removed_component))) {
                    
                    should_skip = true;
                    updated = true;
                    println!("{} {}", "Removing import:".yellow(), trimmed.cyan());
                    break;
                }
            }
            
            // Also check for component name as part of a path segment
            if !should_skip && 
               (trimmed.contains(&format!("::{}", removed_component)) || 
                trimmed.contains(&format!("{}::", removed_component))) {
                should_skip = true;
                updated = true;
                println!("{} {}", "Removing path import:".yellow(), trimmed.cyan());
            }
        }
        
        if should_skip {
            // Skip this line (don't add it to updated_content)
            continue;
        }
        
        // Keep all other lines
        updated_content.push_str(line);
        updated_content.push('\n');
    }
    
    if updated {
        // Write the updated content back to the file
        fs::write(file_path, updated_content)?;
        println!("{} {}", "Updated".green(), 
            format!("imports in {}", file_path.display()).cyan());
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
