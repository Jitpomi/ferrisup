use anyhow::{Result};
use std::fs;
use std::path::Path;
use toml_edit::{value, DocumentMut};
use colored::Colorize;
use dialoguer::MultiSelect;
use crate::commands::test_mode::is_test_mode;
use super::utils::update_root_file_references;
use shared::fs::copy_directory;
use super::ui::{create_root_readme, create_root_gitignore};
// Removed unused import: use shared::fs::create_directory;

// Helper function to categorize files for workspace conversion
pub fn categorize_files(
    project_dir: &Path,
    component_name: &str,
    files_to_keep_at_root: &[String]
) -> Result<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)> {
    // These files will always be skipped and not offered to the user for selection.
    let always_skip_filenames = vec![
        "Cargo.toml".to_string(),    // Workspace Cargo.toml will be created anew
        "Cargo.lock".to_string(),  // Workspace Cargo.lock
        ".git".to_string(),        // Git directory
        ".ferrisup".to_string(),   // FerrisUp metadata directory
        component_name.to_string(),    // The new component directory being created
    ];
    
    // These files must be moved to the component directory to maintain functionality
    // and should not be selectable to keep at root
    let critical_component_files = vec![
        "src".to_string(),         // Source code must move with the component
        "build.rs".to_string(),    // Build script is specific to the component
        "benches".to_string(),     // Benchmarks are specific to the component
        "examples".to_string(),    // Examples are specific to the component
        "bin".to_string()          // Binary files are specific to the component
    ];
    
    // First, identify and categorize all files for better user understanding
    let mut critical_files_to_move = Vec::new();
    let mut other_files_to_move = Vec::new();
    let mut files_kept_at_root = Vec::new();
    let mut workspace_files = Vec::new();
    
    let entries = fs::read_dir(project_dir)?; // Read entries for categorization
    for entry in entries {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = src_path.file_name().unwrap().to_string_lossy().to_string();
        
        if always_skip_filenames.contains(&file_name) {
            workspace_files.push(file_name);
        } else if files_to_keep_at_root.contains(&file_name) {
            files_kept_at_root.push(file_name);
        } else if critical_component_files.contains(&file_name) {
            critical_files_to_move.push(file_name);
        } else {
            other_files_to_move.push(file_name);
        }
    }
    
    Ok((critical_files_to_move, other_files_to_move, files_kept_at_root, workspace_files))
}

// Function to select files to keep at root
pub fn select_files_to_keep_at_root(project_dir: &Path, component_name: &str) -> Result<Vec<String>> {
    // Get list of all files and directories in the project root
    let all_root_entries: Vec<String> = fs::read_dir(project_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();

    // These files will always be skipped and not offered to the user for selection.
    let always_skip_filenames = vec![
        "Cargo.toml".to_string(),    // Workspace Cargo.toml will be created anew
        "Cargo.lock".to_string(),  // Workspace Cargo.lock
        ".git".to_string(),        // Git directory
        ".ferrisup".to_string(),   // FerrisUp metadata directory
        component_name.to_string(),    // The new component directory being created
    ];
    
    // These files must be moved to the component directory to maintain functionality
    // and should not be selectable to keep at root
    let critical_component_files = vec![
        "src".to_string(),         // Source code must move with the component
        "build.rs".to_string(),    // Build script is specific to the component
        "benches".to_string(),     // Benchmarks are specific to the component
        "examples".to_string(),    // Examples are specific to the component
        "bin".to_string()          // Binary files are specific to the component
    ];
    
    // Build artifacts and temporary files that should stay at the root
    let build_artifacts_and_temp_files = vec![
        "target".to_string(),      // Build artifacts
        ".idea".to_string(),       // IntelliJ IDEA settings
        ".vscode".to_string(),     // VS Code settings
        ".DS_Store".to_string(),   // macOS folder settings
        "Cargo.lock".to_string(),  // Cargo lock file
        "*.log".to_string(),       // Log files
        "*.tmp".to_string(),       // Temporary files
        "*.swp".to_string(),       // Vim swap files
        "*.bak".to_string(),       // Backup files
    ];

    // Determine which files are eligible for the user to select to keep at root
    // Only build artifacts and temporary files should be selectable
    let selectable_entries_for_prompt: Vec<String> = all_root_entries
        .iter()
        .filter(|name| {
            // Files must not be in always_skip_filenames or critical_component_files
            !always_skip_filenames.contains(name) && 
            !critical_component_files.contains(name) && 
            // Additionally, only allow selection of build artifacts and temporary files
            // or files that match patterns in build_artifacts_and_temp_files
            build_artifacts_and_temp_files.iter().any(|pattern| {
                if pattern.contains('*') {
                    // Handle wildcard patterns
                    let pattern_parts: Vec<&str> = pattern.split('*').collect();
                    if pattern_parts.len() == 2 {
                        let prefix = pattern_parts[0];
                        let suffix = pattern_parts[1];
                        name.starts_with(prefix) && name.ends_with(suffix)
                    } else {
                        // Simple exact match for non-wildcard patterns
                        pattern == *name
                    }
                } else {
                    // Simple exact match for non-wildcard patterns
                    pattern == *name
                }
            })
        })
        .cloned()
        .collect();

    // Print information about critical files that will automatically move to the component
    if !critical_component_files.is_empty() {
        println!(
            "{}",
            "\nIMPORTANT: The following critical files/directories will automatically be moved to the component:".bold().yellow()
        );
        for file in &critical_component_files {
            if all_root_entries.contains(file) {
                println!("  - {}", file.cyan());
            }
        }
        println!();
    }

    let files_to_keep_at_root: Vec<String> = if !selectable_entries_for_prompt.is_empty() {
        println!(
            "{}",
            "The following files/directories are in your project root:".yellow()
        );

        // Automatically pre-select only build artifacts and temporary files to keep at root
        let default_selections: Vec<bool> = selectable_entries_for_prompt
            .iter()
            .map(|entry_name| {
                build_artifacts_and_temp_files.contains(entry_name)
            })
            .collect();

        println!(
            "{}",
            "\nSAFE DEFAULTS: Only build artifacts and temporary files are pre-selected to stay at root.".bold().green()
        );
        println!(
            "{}",
            "All source code, documentation, and project-specific files will be moved to the component.".green()
        );
        println!();

        if is_test_mode() {
            // In test mode, use default selections
            selectable_entries_for_prompt
                .iter()
                .zip(default_selections.iter())
                .filter_map(|(name, &selected)| if selected { Some(name.clone()) } else { None })
                .collect()
        } else {
            let selections = MultiSelect::new()
                .items(&selectable_entries_for_prompt)
                .with_prompt("Select files/directories to KEEP at the project root (they will NOT be moved to the new component). Use Space to select/deselect, Enter to confirm.")
                .defaults(&default_selections)
                .interact()?;

            selections
                .into_iter()
                .map(|index| selectable_entries_for_prompt[index].clone())
                .collect()
        }
    } else {
        println!("{}", "No movable files found in the project root to select for keeping.".yellow());
        Vec::new()
    };

    if !files_to_keep_at_root.is_empty() {
        println!(
            "{} {:?}",
            "Files selected to keep at root (will not be moved):".cyan(),
            files_to_keep_at_root
        );
    }

    Ok(files_to_keep_at_root)
}

// Function to move files from project root to component directory
pub fn move_files_to_component(
    project_dir: &Path, 
    component_dir: &Path, 
    files_to_keep_at_root: &[String],
    always_skip_filenames: &[String]
) -> Result<()> {
    println!("{}", "Moving files to component directory...".blue());
    
    let entries = fs::read_dir(project_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        // Skip:
        // 1. Files/dirs that are essential for the workspace or the new component dir itself
        // 2. Files/dirs explicitly selected by the user to keep at root
        if always_skip_filenames.contains(&file_name) || files_to_keep_at_root.contains(&file_name) {
            continue;
        }
        
        // Move file or directory to component
        let target_path = component_dir.join(&file_name);

        if path.is_dir() {
            // Copy directory recursively
            copy_directory(&path, &target_path)?;
            // Remove original after successful copy
            fs::remove_dir_all(&path)?;
        } else {
            // Copy file
            fs::copy(&path, &target_path)?;
            // Remove original after successful copy
            fs::remove_file(&path)?;
        }
    }
    
    Ok(())
}

// Function to update component's Cargo.toml
pub fn update_component_cargo_toml(component_dir: &Path, component_name: &str) -> Result<()> {
    let component_cargo_path = component_dir.join("Cargo.toml");
    if component_cargo_path.exists() {
        let component_cargo_content = fs::read_to_string(&component_cargo_path)?;
        let mut component_doc = component_cargo_content.parse::<DocumentMut>()?;

        // Update package name to use component_name chosen by the user
        if let Some(package) = component_doc.get_mut("package") {
            if let Some(table) = package.as_table_mut() {
                table.insert("name", value(component_name.to_lowercase()));
            }
        }

        // Write updated Cargo.toml
        fs::write(component_cargo_path, component_doc.to_string())?;
    }
    
    Ok(())
}

// Function to create workspace Cargo.toml
pub fn create_workspace_cargo_toml(project_dir: &Path, component_name: &str) -> Result<()> {
    let workspace_cargo_toml = format!(
        r#"[workspace]
members = [
    "{}"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
resolver = "2"
"#,
        component_name
    );

    fs::write(project_dir.join("Cargo.toml"), workspace_cargo_toml)?;
    
    Ok(())
}

// Function to finalize workspace setup
pub fn finalize_workspace_setup(
    project_dir: &Path, 
    _component_dir: &Path, 
    component_name: &str, 
    files_to_keep_at_root: &[String]
) -> Result<()> {
    // Update references in files kept at the root
    update_root_file_references(project_dir, component_name, files_to_keep_at_root)?;

    // Create root-level README.md with project structure description
    create_root_readme(project_dir, component_name)?;
    
    // Create root-level .gitignore with standard Rust workspace patterns
    create_root_gitignore(project_dir)?;

    // Print success message
    println!("{}", "Project successfully converted to workspace!".green());
    
    Ok(())
}
