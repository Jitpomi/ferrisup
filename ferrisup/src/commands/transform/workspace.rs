use anyhow::{Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use dialoguer::Confirm;
// Removed unused import: use toml_edit::DocumentMut;

use crate::commands::test_mode::{is_test_mode, test_mode_or};
use super::project_structure::{analyze_project_structure, detect_framework};
use super::utils::{store_transformation_metadata, store_component_type_in_cargo, update_source_imports};
use super::ui::{get_input_with_default};
use super::workspace_utils::{
    select_files_to_keep_at_root, 
    move_files_to_component, 
    update_component_cargo_toml,
    create_workspace_cargo_toml,
    finalize_workspace_setup,
    categorize_files
};
use ferrisup_common::fs::create_directory;

// Main function to convert a project to a workspace
pub fn convert_to_workspace(project_dir: &Path) -> Result<()> {
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let project_name = &structure.project_name;

    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;
    
    // Use the original project name as the default component name
    let default_name = project_name;
    
    // Prompt for component name with default based on component type
    let component_name = test_mode_or(default_name.to_string(), || {
        get_input_with_default(
            &format!("What would you like to name the first component? [{}]", default_name),
            default_name
        )
    })?;

    // Create component directory and src subdirectory
    let component_dir = project_dir.join(&component_name);
    create_directory(&component_dir)?;
    create_directory(&component_dir.join("src"))?;

    // Select files to keep at root
    let files_to_keep_at_root = select_files_to_keep_at_root(project_dir, &component_name)?;
    
    // These files will always be skipped during move
    let always_skip_filenames = vec![
        "Cargo.toml".to_string(),
        "Cargo.lock".to_string(),
        ".git".to_string(),
        ".ferrisup".to_string(),
        component_name.clone(),
    ];

    // Categorize files for display
    let (critical_files_to_move, other_files_to_move, files_kept_at_root, _workspace_files) = 
        categorize_files(project_dir, &component_name, &files_to_keep_at_root)?;
    
    // Display categorized files
    if !critical_files_to_move.is_empty() {
        println!("{}", "\nCritical files that MUST move to component:".yellow().bold());
        for file in &critical_files_to_move {
            println!("  → {} (required for component functionality)", file.cyan());
        }
    }
    
    if !other_files_to_move.is_empty() {
        println!("{}", "\nOther files that will move to component:".yellow());
        for file in &other_files_to_move {
            println!("  → {}", file.green());
        }
    }
    
    if !files_kept_at_root.is_empty() {
        println!("{}", "\nFiles that will stay at the root:".yellow());
        for file in &files_kept_at_root {
            println!("  → {}", file.blue());
        }
    }
    
    // Confirm with user before proceeding
    if !is_test_mode() {
        let proceed = Confirm::new()
            .with_prompt("\nProceed with these file movements?")
            .default(true)
            .interact()?;
            
        if !proceed {
            println!("{}", "Workspace transformation cancelled.".red());
            return Ok(());
        }
    }
    
    // Move files to component directory
    move_files_to_component(project_dir, &component_dir, &files_to_keep_at_root, &always_skip_filenames)?;

    // Copy the original Cargo.toml to the component directory
    let original_cargo_path = project_dir.join("Cargo.toml");
    let component_cargo_path = component_dir.join("Cargo.toml");
    fs::copy(&original_cargo_path, &component_cargo_path)?;

    // Update the component Cargo.toml package name
    update_component_cargo_toml(&component_dir, &component_name)?;

    // Update imports in source files to use the new package name
    update_source_imports(
        &component_dir,
        &project_name.to_lowercase(),
        &component_name.to_lowercase(),
    )?;

    // Create new Cargo.toml for workspace
    create_workspace_cargo_toml(project_dir, &component_name)?;

    // Detect framework from the original project (for metadata only)
    let src_main_path = component_dir.join("src/main.rs");
    let src_lib_path = component_dir.join("src/lib.rs");
    let detected_framework = detect_framework(&[&src_main_path, &src_lib_path]);

    // Determine the component type based on the component name and detected framework
    let template = match component_name.as_str() {
        "client_old" => "client_old",
        "server" => "server",
        "ferrisup_common" => "ferrisup_common",
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        _ => "server", // Default to server if unknown
    };

    // Store transformation metadata
    store_transformation_metadata(
        project_dir, 
        &component_name, 
        template, 
        detected_framework.as_deref()
    )?;

    // Store component type in component's Cargo.toml metadata
    store_component_type_in_cargo(&component_dir, template)?;

    // Finalize workspace setup
    finalize_workspace_setup(project_dir, &component_dir, &component_name, &files_to_keep_at_root)?;

    // Print framework-specific instructions only for reference
    if let Some(framework) = detected_framework {
        println!("{} {}", "Detected framework:".blue(), framework.cyan());
    }

    Ok(())
}
