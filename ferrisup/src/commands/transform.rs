use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::ffi::OsStr;
// Command import removed (no longer needed)
use crate::commands::import_fixer::fix_component_imports;
use crate::utils::{
    copy_dir_contents, create_directory, extract_dependencies, update_cargo_with_dependencies,
    update_workspace_members,
};
use dialoguer::{Confirm, Input, MultiSelect, Select};
use toml_edit::{value, Document, Item, Table, Value};

pub fn execute(project_path: Option<&str>, template_name: Option<&str>) -> Result<()> {
    println!(
        "{}",
        "FerrisUp Interactive Project Transformer".bold().green()
    );
    println!(
        "{}",
        "Transform your existing Rust project with new features".blue()
    );

    // Check if we're in test mode
    let is_test_mode = std::env::var("FERRISUP_TEST_MODE").is_ok();

    // Interactive mode if project path is not provided
    let path_str = match project_path {
        Some(p) => p.to_string(),
        None => {
            // Default to current directory
            let current_dir = std::env::current_dir()?;
            let use_current_dir = if is_test_mode {
                true
            } else {
                Confirm::new()
                    .with_prompt("Use current directory for transformation?")
                    .default(true)
                    .interact()?
            };

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
        println!(
            "{} {} {}",
            "Error:".red().bold(),
            "Directory".red(),
            format!("'{}' does not exist", path_str).red()
        );

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
        println!(
            "{} {}",
            "Error:".red().bold(),
            format!(
                "'{}' is not a valid Rust project (Cargo.toml not found)",
                path_str
            )
            .red()
        );

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

    // Analyze project structure
    println!("{}", "Analyzing project structure...".blue());
    let structure = analyze_project_structure(project_dir)?;

    // Print detected project type
    let project_type = if structure.is_workspace {
        "Workspace"
    } else if structure.is_binary {
        "Binary"
    } else {
        "Library"
    };

    println!(
        "{} {}",
        "Detected project type:".blue(),
        project_type.cyan()
    );

    // Main transformation loop
    let mut is_workspace = structure.is_workspace;
    loop {
        // Show different options based on whether it's a workspace or not
        let options = if is_workspace {
            vec!["Add a component", "Exit"]
        } else {
            vec![
                "Convert project to workspace",
                "Use current structure",
                "Exit",
            ]
        };

        let option_idx = if is_test_mode {
            0
        } else {
            Select::new()
                .with_prompt("What would you like to do?")
                .items(&options)
                .default(0)
                .interact()?
        };

        if is_workspace {
            match option_idx {
                0 => {
                    // Add a component
                    add_component(project_dir)?;
                }
                1 => {
                    // Exit
                    println!("{}", "Exiting transformation.".blue());
                    print_final_next_steps(project_dir)?;
                    break;
                }
                _ => unreachable!(),
            }
        } else {
            match option_idx {
                0 => {
                    // Convert to workspace
                    convert_to_workspace(project_dir, is_test_mode)?;
                    is_workspace = true;
                    // Continue to the next iteration with workspace options
                    continue;
                }
                1 => {
                    // Use current structure
                    println!("{}", "Using current structure.".blue());
                    add_component_without_workspace(project_dir)?;
                }
                2 => {
                    // Exit
                    println!("{}", "Exiting transformation.".blue());
                    break;
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

// Structure to hold project analysis
#[derive(Debug)]
struct ProjectStructure {
    is_workspace: bool,
    is_binary: bool,
    project_name: String,
}

// Function to analyze project structure
fn analyze_project_structure(project_dir: &Path) -> Result<ProjectStructure> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;

    // Parse Cargo.toml
    let cargo_doc = cargo_toml_content
        .parse::<Document>()
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

// Function to update path references in files kept at the root
// Function to update path references in files kept at the root
// Function to update path references in files kept at the root
fn update_root_file_references(project_dir: &Path, component_name: &str, files_to_keep_at_root: &[String]) -> Result<()> {
    println!("{}", "Updating references in files kept at root...".blue());
    
    for file_name in files_to_keep_at_root {
        let file_path = project_dir.join(file_name);
        if !file_path.exists() {
            continue;
        }
        
        // Special handling for .gitignore files
        if file_name.to_lowercase() == ".gitignore" {
            update_gitignore_references(&file_path, component_name)?;
            continue;
        }
        
        // Skip binary files and directories
        if file_path.is_dir() {
            continue;
        }
        
        // Try to read the file as text
        if let Ok(content) = fs::read_to_string(&file_path) {
            // Look for paths that might need updating
            let mut updated_content = content.clone();
            
            // Replace direct references to files that are now in the component directory
            // Using a simpler regex approach to avoid escaping issues
            let src_regex = format!(r"(src/[\w\-\.\/]+)");
            let re = regex::Regex::new(&src_regex).unwrap();
            
            updated_content = re.replace_all(&updated_content, |caps: &regex::Captures| {
                format!("{}/{}", component_name, &caps[1])
            }).to_string();
            
            // If content was modified, write it back
            if content != updated_content {
                println!("Updated references in root file: {}", file_name);
                fs::write(&file_path, updated_content)?;
            }
        }
    }
    
    Ok(())
}

// Special handling for .gitignore files using wildcards
// Special handling for .gitignore files using wildcards
fn update_gitignore_references(gitignore_path: &Path, component_name: &str) -> Result<()> {
    let content = fs::read_to_string(gitignore_path)?;
    let lines = content.lines().collect::<Vec<_>>();
    let mut modified = false;
    let mut all_lines = Vec::new();
    
    // First, collect all existing lines
    for line in &lines {
        all_lines.push(line.to_string());
    }
    
    // Check for src/ references that need to be updated
    for line in &lines {
        // Skip comments and empty lines
        if line.trim().starts_with('#') || line.trim().is_empty() {
            continue;
        }
        
        // If line references src/ directly, add a wildcard version for the component
        if line.trim() == "src/" || line.trim().starts_with("src/") {
            // Create a new line with the component path using wildcards
            let new_line = format!("{}/{}/*", component_name, line.trim());
            if !all_lines.contains(&new_line) {
                all_lines.push(new_line);
                modified = true;
            }
        }
    }
    
    if modified {
        println!("Updated .gitignore with wildcards for component paths");
        fs::write(gitignore_path, all_lines.join("\n"))?;
    }
    
    Ok(())
}

fn convert_to_workspace(project_dir: &Path, is_test_mode: bool) -> Result<()> {
    println!("{}", "Converting project to workspace...".blue());
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let project_name = &structure.project_name;

    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;
    // We use the original project name as the default component name instead of detecting component type    // Use the original project name as the default component name
    let default_name = project_name;
    // Prompt for component name with default based on component type
    let component_name = Input::<String>::new()
        .with_prompt(format!(
            "What would you like to name the first component? [{}]",
            default_name
        ))
        .default(default_name.to_string())
        .interact_text()?;

    // Create component directory and src subdirectory
    let component_dir = project_dir.join(&component_name);
    create_directory(&component_dir)?;
    create_directory(&component_dir.join("src"))?;

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
        component_name.clone(),    // The new component directory being created
    ];
    
    // These files must be moved to the component directory to maintain functionality
    // and should not be selectable to keep at the root
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
    
    // Track if .gitignore is in the root to determine if we need to create one
    // Using underscore prefix to indicate intentionally unused variable
    let _has_gitignore = all_root_entries.contains(&".gitignore".to_string());

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
                println!("  - {} (required for component functionality)", file.cyan());
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

        let selections = MultiSelect::new()
            .items(&selectable_entries_for_prompt)
            .with_prompt("Select files/directories to KEEP at the project root (they will NOT be moved to the new component). Use Space to select/deselect, Enter to confirm.")
            .defaults(&default_selections)
            .interact_opt()? 
            .unwrap_or_else(Vec::new);

        selections
            .into_iter()
            .map(|index| selectable_entries_for_prompt[index].clone())
            .collect()
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

    // Move project files to component directory
    println!("{}", "\nProcessing files for component directory...".blue().bold());
    println!("{}", "The following actions will be taken:".blue());
    
    // First, identify and categorize all files for better user understanding
    let mut critical_files_to_move = Vec::new();
    let mut other_files_to_move = Vec::new();
    let mut files_kept_at_root = Vec::new();
    let mut workspace_files = Vec::new();
    
    let entries = fs::read_dir(project_dir)?; // Read entries for categorization
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        
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
    if !is_test_mode {
        let proceed = Confirm::new()
            .with_prompt("\nProceed with these file movements?")
            .default(true)
            .interact()?;
            
        if !proceed {
            println!("{}", "Workspace transformation cancelled.".red());
            return Ok(());
        }
    }
    
    // Now actually move the files
    println!("{}", "\nMoving files to component directory...".blue());
    let entries = fs::read_dir(project_dir)?; // Re-read entries to ensure freshness
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        // Skip:
        // 1. Files/dirs that are essential for the workspace or the new component dir itself (always_skip_filenames)
        // 2. Files/dirs explicitly selected by the user to keep at root
        if always_skip_filenames.contains(&file_name) {
            // These are essential workspace/tooling files, or the component dir itself, always skipped.
            continue;
        } else if files_to_keep_at_root.contains(&file_name) {
            continue;
        }
        
        // Move file or directory to component
        let target_path = component_dir.join(&file_name);

        if path.is_dir() {
            // Copy directory recursively
            copy_dir_all(&path, &target_path)?;
            // Remove original after successful copy
            fs::remove_dir_all(&path)?;
            println!("Moved directory to component '{}': {}", component_name.cyan(), file_name.green());
        } else {
            // Copy file
            fs::copy(&path, &target_path)?;
            // Remove original after successful copy
            fs::remove_file(&path)?;
            println!("Moved file to component '{}': {}", component_name.cyan(), file_name.green());
        }
    }

    // Just copy the original Cargo.toml to the component directory
    let original_cargo_path = project_dir.join("Cargo.toml");
    let component_cargo_path = component_dir.join("Cargo.toml");

    // Copy the original Cargo.toml to the component directory
    fs::copy(&original_cargo_path, &component_cargo_path)?;

    // Update the component Cargo.toml package name
    let component_cargo_content = fs::read_to_string(&component_cargo_path)?;
    let mut component_cargo_doc = component_cargo_content
        .parse::<Document>()
        .context("Failed to parse component Cargo.toml")?;

    // Update the package name using the component_name chosen by the user
    if let Some(package) = component_cargo_doc.get_mut("package") {
        if let Some(table) = package.as_table_mut() {
            table.insert(
                "name",
                value(format!(
                    "{}",
                    component_name.to_lowercase()
                )),
            );
        }
    }

    // Write updated component Cargo.toml
    fs::write(component_cargo_path, component_cargo_doc.to_string())?;

    // Update imports in source files to use the new package name
    update_source_imports(
        &component_dir,
        &project_name.to_lowercase(),
        &component_name.to_lowercase(),
    )?;

    // Create new Cargo.toml for workspace
    let workspace_cargo_toml = format!(
        r#"[workspace]
members = [
    "{}"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
"#,
        component_name
    );

    fs::write(project_dir.join("Cargo.toml"), workspace_cargo_toml)?;

    // Update component's Cargo.toml to use project-prefixed package name
    let component_cargo_path = component_dir.join("Cargo.toml");
    if component_cargo_path.exists() {
        let component_cargo_content = fs::read_to_string(&component_cargo_path)?;
        let mut component_doc = component_cargo_content
            .parse::<Document>()
            .context("Failed to parse component Cargo.toml")?;

        // Update package name to use project_component format with underscores
        if let Some(package) = component_doc.get_mut("package") {
            if let Some(_name) = package.get_mut("name") {
                // Keep original package name - no change needed
            }
        }

        // Write updated Cargo.toml
        fs::write(component_cargo_path, component_doc.to_string())?;
    } else {
        // Create new Cargo.toml for component if it doesn't exist
        let component_cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
            component_name
        );

        fs::write(component_cargo_path, component_cargo_toml)?;
    }

    // Detect framework from the original project (for metadata only)
    let mut detected_framework = None;
    let src_main_path = component_dir.join("src/main.rs");
    let src_lib_path = component_dir.join("src/lib.rs");

    // Check both main.rs and lib.rs for framework detection
    for src_path in &[&src_main_path, &src_lib_path] {
        if src_path.exists() {
            let content = fs::read_to_string(src_path)?;

            // Try to detect the framework from imports (for metadata only)
            if content.contains("use poem") {
                detected_framework = Some("poem");
                break;
            } else if content.contains("use axum") {
                detected_framework = Some("axum");
                break;
            } else if content.contains("use actix_web") {
                detected_framework = Some("actix");
                break;
            } else if content.contains("use leptos") {
                detected_framework = Some("leptos");
                break;
            } else if content.contains("use dioxus") {
                detected_framework = Some("dioxus");
                break;
            }
        }
    }

    // We're not adding dependencies manually anymore since we've preserved the original ones
    if let Some(framework) = detected_framework {
        println!("{} {}", "Detected framework:".blue(), framework.cyan());
    }

    // Determine the component type based on the component name and detected framework
    let template = match component_name.as_str() {
        "client" => "client",
        "server" => "server",
        "shared" => "shared",
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        _ => "server", // Default to server if unknown
    };

    // Store transformation metadata
    store_transformation_metadata(project_dir, &component_name, template, detected_framework)?;

    // Store component type in component's Cargo.toml metadata
    store_component_type_in_cargo(&component_dir, template)?;

    // Update references in files kept at the root
    update_root_file_references(project_dir, &component_name, &files_to_keep_at_root)?;

    // Fix workspace resolver
    let workspace_cargo_path = project_dir.join("Cargo.toml");
    let workspace_cargo_content = fs::read_to_string(&workspace_cargo_path)?;
    let mut workspace_doc = workspace_cargo_content
        .parse::<Document>()
        .context("Failed to parse workspace Cargo.toml")?;

    // Add resolver = "2" to workspace
    if let Some(workspace) = workspace_doc.get_mut("workspace") {
        if let Some(table) = workspace.as_table_mut() {
            if table.get("resolver").is_none() {
                table.insert("resolver", value("2"));
            }
        }
    }

    // Write updated workspace Cargo.toml
    fs::write(workspace_cargo_path, workspace_doc.to_string())?;

    // Always create a root-level README.md with project structure description
    // This is important regardless of whether the user selected to keep README.md at the root
    println!("{}", "\nCreating root-level README.md with workspace structure description...".blue());
    create_root_readme(project_dir, &component_name)?;
    
    // Always create a root-level .gitignore with standard Rust workspace patterns
    // This is important regardless of whether the user selected to keep .gitignore at the root
    println!("{}", "Creating root-level .gitignore with standard Rust workspace patterns...".blue());
    create_root_gitignore(project_dir)?;

    // Print success message
    println!("{}", "Project successfully converted to workspace!".green());
    println!("{}", "Created root README.md with project structure description".green());
    println!("{}", "Created root .gitignore with standard Rust workspace patterns".green());

    // Print framework-specific instructions only for reference
    if let Some(framework) = detected_framework {
        println!("{} {}", "Detected framework:".blue(), framework.cyan());
    }

    Ok(())
}

// Function to create a root-level README.md with project structure description
fn create_root_readme(project_dir: &Path, component_name: &str) -> Result<()> {
    let readme_path = project_dir.join("README.md");
    
    // If README.md already exists, back it up
    if readme_path.exists() {
        let backup_path = project_dir.join("README.md.bak");
        println!("{} {}", "Backing up existing README.md to".yellow(), backup_path.display().to_string().yellow());
        fs::copy(&readme_path, &backup_path)?;
    }
    
    // Create a new README.md with workspace structure information
    let readme_content = format!(r#"# Workspace Project

This is a Rust workspace project created with FerrisUp.

## Project Structure

This workspace contains the following components:

- `{}`: The main component of the project

## Development

To build all components in the workspace:

```bash
cargo build
```

To run tests for all components:

```bash
cargo test
```

To add a new component to the workspace:

```bash
ferrisup transform
```

## License

This project is licensed under the terms specified in the LICENSE file, if present.
"#, component_name);
    
    fs::write(readme_path, readme_content)?;
    Ok(())
}
fn create_root_gitignore(project_dir: &Path) -> Result<()> {
    let gitignore_path = project_dir.join(".gitignore");
    
    if gitignore_path.exists() {
        // Back up existing .gitignore
        let backup_path = project_dir.join(".gitignore.bak");
        println!("{} {}", "Backing up existing .gitignore to".yellow(), backup_path.display().to_string().yellow());
        fs::copy(&gitignore_path, &backup_path)?;
        
        // Read existing content
        let existing_content = fs::read_to_string(&gitignore_path)?;
        
        // Create new content with workspace patterns
        let gitignore_content = format!(r#"# Modified by FerrisUp Workspace Transformation

# Rust Workspace Standard Patterns
/target/
**/*.rs.bk
*.pdb

# IDEs and editors
/.idea/
/.vscode/
*.swp
*.swo
*.iml

# OS specific
.DS_Store
Thumbs.db

# Build artifacts
*.o
*.so
*.dylib
*.dll
*.exe

# Logs
*.log

# Original content below
{}
"#, existing_content);
        
        fs::write(gitignore_path, gitignore_content)?;
    } else {
        // Create a new .gitignore with standard patterns
        let gitignore_content = r#"# Generated by FerrisUp

# Rust
/target/
**/*.rs.bk
*.pdb
Cargo.lock

# IDEs and editors
/.idea/
/.vscode/
*.swp
*.swo
*.iml

# OS specific
.DS_Store
Thumbs.db

# Build artifacts
*.o
*.so
*.dylib
*.dll
*.exe

# Logs
*.log
"#;

        fs::write(gitignore_path, gitignore_content)?;
    }
    
    Ok(())
}

// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

// Function to add a component to a workspace
pub fn add_component(project_dir: &Path) -> Result<()> {
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let _project_name = &structure.project_name;

    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

    // Select component type
    let component_types = vec![
        "client - Frontend web application (Leptos, Yew, or Dioxus)",
        "server - Web server with API endpoints (Axum, Actix, or Poem)",
        "shared - Shared code between client and server",
        "edge - Edge computing applications (Cloudflare, Vercel, Fastly)",
        "data-science - Data science and machine learning projects",
        "embedded - Embedded systems firmware",
    ];

    let component_idx = Select::new()
        .with_prompt("Select component type:")
        .items(&component_types)
        .default(0)
        .interact()?;

    // Map index to component type
    let component_type = match component_idx {
        0 => "client",
        1 => "server",
        2 => "shared",
        3 => "edge",
        4 => "data-science",
        5 => "embedded",
        _ => "client", // Default to client
    };

    // Prompt for component name with default based on component type
    let component_name = Input::<String>::new()
        .with_prompt(format!("Component name [{}]", component_type))
        .default(component_type.to_string())
        .interact_text()?;

    // Define component directory path (but don't create it yet)
    let component_dir = project_dir.join(&component_name);

    // Check if directory already exists
    if component_dir.exists() {
        println!(
            "{} {}",
            "Error:".red().bold(),
            format!("Component directory '{}' already exists", component_name).red()
        );
        return Ok(());
    }

    // Select framework if applicable
    let framework = match component_type {
        "client" => {
            let frameworks = vec![
                "leptos - Reactive web framework with fine-grained reactivity",
                "dioxus - Elegant React-like framework for desktop, web, and mobile",
                "tauri - Build smaller, faster, and more secure desktop applications",
            ];

            let framework_idx = Select::new()
                .with_prompt("Select framework:")
                .items(&frameworks)
                .default(0)
                .interact()?;

            match framework_idx {
                0 => Some("leptos"),
                1 => Some("dioxus"),
                2 => Some("tauri"),
                _ => None,
            }
        }
        "server" => {
            let frameworks = vec![
                "axum - Ergonomic and modular web framework by Tokio",
                "actix - Powerful, pragmatic, and extremely fast web framework",
                "poem - Full-featured and easy-to-use web framework",
            ];

            let framework_idx = Select::new()
                .with_prompt("Select framework:")
                .items(&frameworks)
                .default(0)
                .interact()?;

            match framework_idx {
                0 => Some("axum"),
                1 => Some("actix"),
                2 => Some("poem"),
                _ => None,
            }
        }
        "edge" => {
            let providers = vec![
                "cloudflare - Cloudflare Workers",
                "vercel - Vercel Edge Functions",
                "fastly - Fastly Compute@Edge",
                "aws - AWS Lambda@Edge",
            ];

            let provider_idx = Select::new()
                .with_prompt("Select provider:")
                .items(&providers)
                .default(0)
                .interact()?;

            match provider_idx {
                0 => Some("cloudflare"),
                1 => Some("vercel"),
                2 => Some("fastly"),
                3 => Some("aws"),
                _ => None,
            }
        }
        "data-science" => {
            let frameworks = vec![
                "polars - Fast DataFrame library",
                "linfa - Machine learning framework",
            ];

            let framework_idx = Select::new()
                .with_prompt("Select framework:")
                .items(&frameworks)
                .default(0)
                .interact()?;

            match framework_idx {
                0 => Some("polars"),
                1 => Some("linfa"),
                _ => None,
            }
        }
        _ => None,
    };

    // Create the component by directly calling the new command
    println!(
        "{}",
        format!(
            "Creating {} component with name: {}",
            component_type, component_name
        )
        .blue()
    );

    // Map component type to template
    // For shared components, we need to explicitly use "library" as the template
    let template = if component_type == "shared" {
        "library"
    } else {
        map_component_to_template(component_type)
    };

    // Print the template being used for debugging
    println!("{}", format!("Using template: {}", template).blue());

    // Save current directory to return to it after component creation
    let current_dir = std::env::current_dir()?;

    // Change to project directory to create component at the right location
    std::env::set_current_dir(project_dir)?;

    // Call the new command to create the component
    let result = crate::commands::new::execute(
        Some(&component_name),
        Some(template),
        framework.as_deref(),
        None,
        None,
        false,
        false,
        false,
        None,
    );

    // Change back to original directory
    std::env::set_current_dir(current_dir)?;

    if let Err(e) = result {
        println!("{} {}", "Error creating component:".red().bold(), e);
        return Err(anyhow!("Failed to create component"));
    }

    // Display detected framework
    if let Some(framework_name) = &framework {
        println!("{} {}", "Using framework:".blue(), framework_name.cyan());
    }

    // Store transformation metadata
    store_transformation_metadata(project_dir, &component_name, template, framework.as_deref())?;

    // Store component type in component's Cargo.toml metadata
    store_component_type_in_cargo(&component_dir, template)?;

    // If this is a shared component, make it accessible to all workspace members
    if component_type == "shared" {
        make_shared_component_accessible(project_dir, &component_name)?;
    }

    // Create an empty list for files to keep at root since we're not moving files in this case
    let files_to_keep_at_root: Vec<String> = Vec::new();
    
    // Update references in files kept at the root
    update_root_file_references(project_dir, &component_name, &files_to_keep_at_root)?;

    // Define component cargo path
    let component_cargo_path = component_dir.join("Cargo.toml");
    
    // Get project name from directory name
    let project_name = project_dir.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    
    if component_cargo_path.exists() {
        let component_cargo_content = fs::read_to_string(&component_cargo_path)?;
        let mut component_doc = component_cargo_content
            .parse::<Document>()
            .context("Failed to parse component Cargo.toml")?;

        // Update package name to use project_component format with underscores
        if let Some(package) = component_doc.get_mut("package") {
            if let Some(_name) = package.get_mut("name") {
                // Keep original package name - no change needed
            }
        }

        // Write updated Cargo.toml
        fs::write(component_cargo_path, component_doc.to_string())?;

        // Fix imports in source files to use the new package name
        // Make sure we're using the actual project name, not 'unknown'
        // First try to get the name from workspace.package.name
        // If that doesn't exist, try to get it from the directory name
        // If all else fails, use the project_dir name
        let workspace_cargo_path = project_dir.join("Cargo.toml");
        let actual_project_name = if workspace_cargo_path.exists() {
            if let Ok(workspace_content) = fs::read_to_string(&workspace_cargo_path) {
                if let Ok(workspace_doc) = workspace_content.parse::<Document>() {
                    // Try workspace.package.name first
                    if let Some(workspace) = workspace_doc.get("workspace") {
                        if let Some(pkg) = workspace.get("package") {
                            if let Some(name) = pkg.get("name") {
                                if let Some(name_str) = name.as_str() {
                                    name_str.to_string()
                                } else {
                                    // Fall back to project directory name
                                    project_dir
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(project_name)
                                        .to_string()
                                }
                            } else {
                                // Fall back to project directory name
                                project_dir
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(project_name)
                                    .to_string()
                            }
                        } else {
                            // Try package.name directly (for non-workspace projects)
                            if let Some(pkg) = workspace_doc.get("package") {
                                if let Some(name) = pkg.get("name") {
                                    if let Some(name_str) = name.as_str() {
                                        name_str.to_string()
                                    } else {
                                        // Fall back to project directory name
                                        project_dir
                                            .file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or(project_name)
                                            .to_string()
                                    }
                                } else {
                                    // Fall back to project directory name
                                    project_dir
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(project_name)
                                        .to_string()
                                }
                            } else {
                                // Fall back to project directory name
                                project_dir
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(project_name)
                                    .to_string()
                            }
                        }
                    } else {
                        // Try package.name directly (for non-workspace projects)
                        if let Some(pkg) = workspace_doc.get("package") {
                            if let Some(name) = pkg.get("name") {
                                if let Some(name_str) = name.as_str() {
                                    name_str.to_string()
                                } else {
                                    // Fall back to project directory name
                                    project_dir
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(project_name)
                                        .to_string()
                                }
                            } else {
                                // Fall back to project directory name
                                project_dir
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(project_name)
                                    .to_string()
                            }
                        } else {
                            // Fall back to project directory name
                            project_dir
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or(project_name)
                                .to_string()
                        }
                    }
                } else {
                    // Fall back to project directory name
                    project_dir
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(project_name)
                        .to_string()
                }
            } else {
                // Fall back to project directory name
                project_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(project_name)
                    .to_string()
            }
        } else {
            // Fall back to project directory name
            project_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(project_name)
                .to_string()
        };

        println!(
            "{}",
            format!("Using project name '{}' for imports", actual_project_name).blue()
        );
        fix_component_imports(&component_dir, &component_name, &actual_project_name)?;
    }

    // Update workspace Cargo.toml to include the new component
    // Note: The update_workspace_members function will automatically detect and add the component
    if let Err(e) = update_workspace_members(project_dir) {
        println!(
            "{} {}",
            "Warning: Failed to update workspace members:"
                .yellow()
                .bold(),
            e
        );
    }

    println!(
        "{}",
        format!(
            "Component '{}' successfully added to workspace!",
            component_name
        )
        .green()
    );

    Ok(())
}

// Function to add a component without converting to workspace
pub fn add_component_without_workspace(project_dir: &Path) -> Result<()> {
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let _project_name = &structure.project_name;

    // Select component type - ONLY show module-compatible components
    let component_types = vec![
        "shared - Shared code between client and server",
        "minimal - Simple Rust module with minimal dependencies",
        // Could add other simple module types here if needed
    ];

    let component_idx = Select::new()
        .with_prompt("Select module type:")
        .items(&component_types)
        .default(0)
        .interact()?;

    // Map index to component type
    let component_type = match component_idx {
        0 => "shared",
        1 => "minimal",
        _ => "shared", // Default to shared
    };

    // Prompt for component name with default based on component type
    let module_name = Input::<String>::new()
        .with_prompt(format!("Module name [{}]", component_type))
        .default(component_type.to_string())
        .interact_text()?;

    // Create module directory inside src instead of at project root
    let src_dir = project_dir.join("src");
    let module_dir = src_dir.join(&module_name);

    // Check if directory already exists
    if module_dir.exists() {
        println!(
            "{} {}",
            "Error:".red().bold(),
            format!("Module directory '{}' already exists", module_name).red()
        );
        return Ok(());
    }

    // Create module directory
    create_directory(&module_dir)?;

    // First, create a temporary directory to generate the component template
    let temp_dir = tempfile::tempdir()?;
    let temp_component_dir = temp_dir.path().join(&module_name);

    // Save current directory
    let current_dir = std::env::current_dir()?;

    // Change to temp directory to create component template
    std::env::set_current_dir(temp_dir.path())?;

    // Map component type to template
    let template = map_component_to_template(component_type);

    println!(
        "{}",
        format!(
            "Creating {} module with name: {}",
            component_type, module_name
        )
        .blue()
    );

    // Call the new command to create the component template in temp directory
    let result = crate::commands::new::execute(
        Some(&module_name),
        Some(template),
        None, // No framework needed for modules
        None,
        None,
        false,
        false,
        true, // Use non-interactive mode
        None,
    );

    // Change back to original directory
    std::env::set_current_dir(current_dir)?;

    if let Err(e) = result {
        println!("{} {}", "Error creating component:".red().bold(), e);
        return Err(anyhow!("Failed to create component"));
    }

    // Extract dependencies from template's Cargo.toml
    let template_cargo_path = temp_component_dir.join("Cargo.toml");
    let mut dependencies_to_add = Vec::new();

    if template_cargo_path.exists() {
        let template_cargo_content = fs::read_to_string(&template_cargo_path)?;
        let template_doc = template_cargo_content
            .parse::<Document>()
            .context("Failed to parse template Cargo.toml")?;

        if let Some(deps) = template_doc.get("dependencies") {
            dependencies_to_add = extract_dependencies(deps)?;
        }
    }

    // Copy src files from template to module directory
    let template_src_dir = temp_component_dir.join("src");
    if template_src_dir.exists() {
        copy_dir_contents(&template_src_dir, &module_dir)?;

        // Rename lib.rs to mod.rs if it exists
        let lib_rs_path = module_dir.join("lib.rs");
        let mod_rs_path = module_dir.join("mod.rs");

        if lib_rs_path.exists() && !mod_rs_path.exists() {
            fs::rename(lib_rs_path, mod_rs_path)?;
        }
    }

    // Update project's Cargo.toml with dependencies
    let project_cargo_path = project_dir.join("Cargo.toml");
    if project_cargo_path.exists() && !dependencies_to_add.is_empty() {
        update_cargo_with_dependencies(&project_cargo_path, dependencies_to_add, false)?;
    }

    // Update main.rs or lib.rs to include the new module
    update_project_source_to_include_module(project_dir, &module_name)?;

    println!(
        "{}",
        format!(
            "Module '{}' successfully created within the project!",
            module_name
        )
        .green()
    );

    Ok(())
}

// Helper function to update project source to include the new module
fn update_project_source_to_include_module(project_dir: &Path, module_name: &str) -> Result<()> {
    // Determine if this is a binary or library project
    let main_rs_path = project_dir.join("src/main.rs");
    let lib_rs_path = project_dir.join("src/lib.rs");

    if main_rs_path.exists() {
        // Binary project
        let mut content = fs::read_to_string(&main_rs_path)?;

        // Add module declaration if not already present
        if !content.contains(&format!("mod {};", module_name)) {
            content.push_str(&format!("\nmod {};\n", module_name));
            fs::write(main_rs_path, content)?;
        }
    } else if lib_rs_path.exists() {
        // Library project
        let mut content = fs::read_to_string(&lib_rs_path)?;

        // Add module declaration and pub use if not already present
        if !content.contains(&format!("mod {};", module_name)) {
            content.push_str(&format!("\nmod {};\n", module_name));
            content.push_str(&format!("pub use {}::*;\n", module_name));
            fs::write(lib_rs_path, content)?;
        }
    }

    Ok(())
}

// Helper function to map component type to template
fn map_component_to_template(component_type: &str) -> &str {
    match component_type {
        "client" => "client",
        "server" => "server",
        "shared" => "library", // Use library template for shared components
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        "minimal" => "minimal",
        _ => "server", // Default to server
    }
}

// Helper function to update imports in source files
fn update_source_imports(
    component_dir: &Path,
    project_name: &str,
    component_name: &str,
) -> Result<()> {
    // Create the new package name
    // Use the original package name instead of creating a new prefixed one
    let new_package_name = component_name.to_string();

    // Walk through all Rust source files in the component directory
    let src_dir = component_dir.join("src");
    if !src_dir.exists() {
        return Ok(());
    }

    // Process main.rs
    let main_rs_path = src_dir.join("main.rs");
    if main_rs_path.exists() {
        update_imports_in_file(&main_rs_path, project_name, &new_package_name)?;
    }

    // Process lib.rs
    let lib_rs_path = src_dir.join("lib.rs");
    if lib_rs_path.exists() {
        update_imports_in_file(&lib_rs_path, project_name, &new_package_name)?;
    }

    // Process other Rust files in the src directory
    process_directory_imports(&src_dir, project_name, &new_package_name)?;

    Ok(())
}

// Helper function to update imports in a single file
fn update_imports_in_file(
    file_path: &Path,
    project_name: &str,
    new_package_name: &str,
) -> Result<()> {
    // Read the file content
    let content = fs::read_to_string(file_path)?;

    // Only replace exact package name to avoid multiple replacements
    // For example, replace "use app::*;" with "use app_client::*;"
    // but not "use app_client::*;" with "use app_client_client::*;"

    // Use regex to ensure we're only replacing the exact package name
    let re_import = regex::Regex::new(&format!(r"\buse\s+{}\b", regex::escape(project_name)))
        .context("Failed to create regex for import")?;
    let updated_content = re_import.replace_all(&content, format!("use {}", new_package_name));

    // Write the updated content back to the file
    fs::write(file_path, updated_content.to_string())?;

    Ok(())
}

// Helper function to recursively process all Rust files in a directory
fn process_directory_imports(
    dir_path: &Path,
    project_name: &str,
    new_package_name: &str,
) -> Result<()> {
    if !dir_path.exists() || !dir_path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            process_directory_imports(&path, project_name, new_package_name)?;
        } else if path.is_file() {
            // Process Rust files
            if let Some(extension) = path.extension() {
                if extension == "rs" {
                    update_imports_in_file(&path, project_name, new_package_name)?;
                }
            }
        }
    }

    Ok(())
}
#[allow(dead_code)]
// Function to detect component type based on project files
fn detect_component_type(project_dir: &Path) -> Result<&'static str> {
    // First, check if there's an explicit component type in the Cargo.toml metadata
    let cargo_toml = project_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml)?;
        let cargo_doc = cargo_content.parse::<Document>().ok();

        if let Some(doc) = cargo_doc {
            // Check for ferrisup metadata in Cargo.toml
            if let Some(package) = doc.get("package") {
                if let Some(metadata) = package.get("metadata") {
                    if let Some(ferrisup) = metadata.get("ferrisup") {
                        if let Some(component_type) = ferrisup.get("component_type") {
                            if let Some(component_str) = component_type.as_str() {
                                match component_str {
                                    "client" => return Ok("client"),
                                    "server" => return Ok("server"),
                                    "shared" => return Ok("shared"),
                                    "edge" => return Ok("edge"),
                                    "serverless" => return Ok("serverless"),
                                    "data-science" => return Ok("data-science"),
                                    "embedded" => return Ok("embedded"),
                                    "binary" => return Ok("binary"),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Check for .ferrisup/metadata.toml - this would have the original component type
    let metadata_path = project_dir.join(".ferrisup/metadata.toml");
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let metadata_doc = metadata_content.parse::<Document>().ok();

        if let Some(doc) = metadata_doc {
            // Try to get component_type directly
            if let Some(component_type) = doc.get("component_type") {
                if let Some(component_str) = component_type.as_str() {
                    match component_str {
                        "client" => return Ok("client"),
                        "server" => return Ok("server"),
                        "shared" => return Ok("shared"),
                        "edge" => return Ok("edge"),
                        "serverless" => return Ok("serverless"),
                        "data-science" => return Ok("data-science"),
                        "embedded" => return Ok("embedded"),
                        "binary" => return Ok("binary"),
                        _ => {}
                    }
                }
            }

            // Try to infer from template
            if let Some(template) = doc.get("template") {
                if let Some(template_str) = template.as_str() {
                    if template_str.contains("serverless") {
                        return Ok("serverless");
                    } else if template_str.contains("edge") {
                        return Ok("edge");
                    } else if template_str.contains("client") {
                        return Ok("client");
                    } else if template_str.contains("server") {
                        return Ok("server");
                    } else if template_str.contains("shared") {
                        return Ok("shared");
                    } else if template_str.contains("data-science") {
                        return Ok("data-science");
                    } else if template_str.contains("embedded") {
                        return Ok("embedded");
                    }
                }
            }
        }

        // Fallback to simple string matching if parsing fails
        if metadata_content.contains("component_type = \"client\"") {
            return Ok("client");
        } else if metadata_content.contains("component_type = \"server\"") {
            return Ok("server");
        } else if metadata_content.contains("component_type = \"shared\"") {
            return Ok("shared");
        } else if metadata_content.contains("component_type = \"edge\"") {
            return Ok("edge");
        } else if metadata_content.contains("component_type = \"serverless\"") {
            return Ok("serverless");
        } else if metadata_content.contains("component_type = \"data-science\"") {
            return Ok("data-science");
        } else if metadata_content.contains("component_type = \"embedded\"") {
            return Ok("embedded");
        } else if metadata_content.contains("component_type = \"binary\"") {
            return Ok("binary");
        } else if metadata_content.contains("template = \"client\"") {
            return Ok("client");
        } else if metadata_content.contains("template = \"server\"") {
            return Ok("server");
        } else if metadata_content.contains("template = \"shared\"") {
            return Ok("shared");
        } else if metadata_content.contains("template = \"edge\"") {
            return Ok("edge");
        } else if metadata_content.contains("template = \"serverless\"") {
            return Ok("serverless");
        } else if metadata_content.contains("template = \"data-science\"") {
            return Ok("data-science");
        } else if metadata_content.contains("template = \"embedded\"") {
            return Ok("embedded");
        } else if metadata_content.contains("template = \"binary\"") {
            return Ok("binary");
        }
    }

    // Check for edge-specific files and directories
    if project_dir.join("workers-site").exists() || project_dir.join("wrangler.toml").exists() {
        return Ok("edge");
    }

    // Check for Vercel files - could be edge or serverless
    if project_dir.join("vercel.json").exists() || project_dir.join(".vercel").exists() {
        // Check if this is a serverless function by looking at the Cargo.toml
        let cargo_toml = project_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let cargo_content = fs::read_to_string(&cargo_toml)?;

            // If it contains serverless keywords, it's a serverless project
            if cargo_content.contains("lambda")
                || cargo_content.contains("aws-lambda")
                || cargo_content.contains("aws_lambda")
                || cargo_content.contains("serverless")
            {
                return Ok("serverless");
            }

            // If it contains edge keywords, it's an edge project
            if cargo_content.contains("wasm")
                || cargo_content.contains("static-site")
                || cargo_content.contains("edge")
            {
                return Ok("edge");
            }
        }

        // Check for serverless directory structure
        if project_dir.join("api").exists() {
            return Ok("serverless");
        }

        // Check for edge-specific files
        if project_dir.join("index.html").exists() && project_dir.join("pkg").exists() {
            return Ok("edge");
        }

        // Check metadata files for clues
        let metadata_path = project_dir.join(".ferrisup/metadata.toml");
        if metadata_path.exists() {
            let metadata_content = fs::read_to_string(&metadata_path)?;
            if metadata_content.contains("static-site") || metadata_content.contains("edge/static")
            {
                return Ok("edge");
            }
        }

        // Default to edge for Vercel projects with no serverless indicators
        return Ok("edge");
    }

    // Check for serverless-specific files
    if project_dir.join("serverless.yml").exists()
        || project_dir.join(".aws").exists()
        || project_dir.join("template.yaml").exists()
        || project_dir.join("template.yml").exists()
        || project_dir.join("sam-template.yaml").exists()
        || project_dir.join("sam-template.yml").exists()
    {
        return Ok("serverless");
    }

    // Check for client-specific files and imports
    let src_dir = project_dir.join("src");
    let main_rs = src_dir.join("main.rs");
    let lib_rs = src_dir.join("lib.rs");
    let index_html = project_dir.join("index.html");
    let cargo_toml = project_dir.join("Cargo.toml");

    // Check for frameworks in Cargo.toml
    if cargo_toml.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml)?;

        // Check for serverless frameworks first (higher priority)
        if cargo_content.contains("lambda")
            || cargo_content.contains("aws-lambda")
            || cargo_content.contains("aws_lambda")
            || cargo_content.contains("serverless")
        {
            return Ok("serverless");
        }

        // Check for edge frameworks
        if cargo_content.contains("worker")
            || cargo_content.contains("cloudflare")
            || cargo_content.contains("fastly")
        {
            return Ok("edge");
        }

        // Check for Vercel - could be edge or serverless
        if cargo_content.contains("vercel") {
            // Look for clues that this is a serverless function
            if project_dir.join("api").exists() {
                return Ok("serverless");
            } else {
                return Ok("edge");
            }
        }

        // Check for client frameworks
        if cargo_content.contains("leptos")
            || cargo_content.contains("dioxus")
            || cargo_content.contains("yew")
            || cargo_content.contains("trunk")
            || cargo_content.contains("wasm")
        {
            return Ok("client");
        }
    }

    // Check for index.html (typical in client projects)
    if index_html.exists() {
        return Ok("client");
    }

    // Check for imports in source files
    for rs_file in &[main_rs, lib_rs] {
        if rs_file.exists() {
            let content = fs::read_to_string(rs_file)?;

            // Check for edge frameworks
            if content.contains("use worker")
                || content.contains("use cloudflare")
                || content.contains("use vercel")
                || content.contains("use fastly")
            {
                return Ok("edge");
            }

            // Check for serverless frameworks
            if content.contains("use lambda")
                || content.contains("use aws_lambda")
                || content.contains("use lambda_runtime")
                || content.contains("lambda::handler")
                || content.contains("lambda::function")
            {
                return Ok("serverless");
            }

            // Check for client frameworks
            if content.contains("use leptos")
                || content.contains("use dioxus")
                || content.contains("use yew")
                || content.contains("wasm_bindgen")
            {
                return Ok("client");
            }

            // Check for server frameworks
            if content.contains("use poem")
                || content.contains("use axum")
                || content.contains("use actix")
                || content.contains("use rocket")
                || content.contains("use warp")
            {
                return Ok("server");
            }
        }
    }

    // Look for Trunk.toml (client project)
    if project_dir.join("Trunk.toml").exists() {
        return Ok("client");
    }

    // Check for data-science specific dependencies
    if cargo_toml.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml)?;
        if cargo_content.contains("polars")
            || cargo_content.contains("linfa")
            || cargo_content.contains("ndarray")
        {
            return Ok("data-science");
        }

        // Check for minimal template
        if let Some(metadata) = cargo_content.find("[package.metadata.ferrisup]") {
            let after_metadata = &cargo_content[metadata..];
            if after_metadata.contains("component_type = \"minimal\"") {
                return Ok("minimal");
            }
        }
    }

    // Check for embedded specific dependencies
    if cargo_toml.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml)?;
        if cargo_content.contains("embedded-hal")
            || cargo_content.contains("cortex-m")
            || cargo_content.contains("stm32")
        {
            return Ok("embedded");
        }
    }

    // Check for serverless-related file patterns
    let src_dir = project_dir.join("src");
    if src_dir.exists() {
        let entries = fs::read_dir(src_dir)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            // Look for handler.rs, lambda.rs, or function.rs which are common in serverless projects
            if file_name == "handler.rs" || file_name == "lambda.rs" || file_name == "function.rs" {
                return Ok("serverless");
            }
        }
    }

    // Check for minimal project structure
    let src_dir = project_dir.join("src");
    let main_rs = src_dir.join("main.rs");

    // If it's just a simple binary with a main.rs and no other special files, it's likely minimal
    if main_rs.exists() && src_dir.exists() {
        // Check if src directory has only main.rs and no other files
        let entries = fs::read_dir(&src_dir)?;
        let file_count = entries.count();

        if file_count == 1 && cargo_toml.exists() {
            // Read Cargo.toml to check for minimal dependencies
            let cargo_content = fs::read_to_string(&cargo_toml)?;

            // If Cargo.toml is simple (few dependencies), it's likely minimal
            if !cargo_content.contains("axum")
                && !cargo_content.contains("actix")
                && !cargo_content.contains("poem")
                && !cargo_content.contains("rocket")
                && !cargo_content.contains("leptos")
                && !cargo_content.contains("dioxus")
                && !cargo_content.contains("yew")
            {
                // Check main.rs for simplicity
                let main_content = fs::read_to_string(&main_rs)?;
                if main_content.lines().count() < 30 {
                    return Ok("minimal");
                }
            }
        }
    }

    // Default to binary for CLI applications, server for other binary projects, shared for libraries
    let structure = analyze_project_structure(project_dir)?;
    if structure.is_binary {
        // Check if it's a CLI application by looking for CLI-specific dependencies
        if cargo_toml.exists() {
            let cargo_content = fs::read_to_string(&cargo_toml)?;
            if cargo_content.contains("clap")
                || cargo_content.contains("structopt")
                || cargo_content.contains("argh")
                || cargo_content.contains("pico-args")
                || cargo_content.contains("gumdrop")
                || cargo_content.contains("command-line")
                || cargo_content.contains("command_line")
                || cargo_content.contains("cli")
            {
                return Ok("binary");
            }
        }
        
        // For binary projects, prefer serverless over server as default if we detect any AWS-related files
        if project_dir.join(".aws").exists()
            || cargo_toml.exists() && fs::read_to_string(&cargo_toml)?.contains("aws")
        {
            Ok("serverless")
        } else {
            Ok("server")
        }
    } else {
        Ok("shared")
    }
}

// Function to store component type in Cargo.toml metadata
fn store_component_type_in_cargo(component_dir: &Path, component_type: &str) -> Result<()> {
    let cargo_path = component_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        return Ok(());
    }

    let cargo_content = fs::read_to_string(&cargo_path)?;
    let mut doc = cargo_content
        .parse::<Document>()
        .context("Failed to parse Cargo.toml")?;

    // Ensure package section exists
    if doc.get("package").is_none() {
        doc.insert("package", Item::Table(Table::new()));
    }

    // Get or create metadata section
    let package = doc["package"].as_table_mut().unwrap();
    if package.get("metadata").is_none() {
        package.insert("metadata", Item::Table(Table::new()));
    }

    // Get or create ferrisup section in metadata
    let metadata = package["metadata"].as_table_mut().unwrap();
    if metadata.get("ferrisup").is_none() {
        metadata.insert("ferrisup", Item::Table(Table::new()));
    }

    // Set component_type in ferrisup metadata
    let ferrisup = metadata["ferrisup"].as_table_mut().unwrap();
    ferrisup.insert("component_type", value(component_type));

    // Write updated Cargo.toml
    fs::write(cargo_path, doc.to_string())?;

    Ok(())
}

// Function to make shared components accessible to all workspace members
fn make_shared_component_accessible(project_dir: &Path, component_name: &str) -> Result<()> {
    println!("{}", format!("Making shared component '{}' accessible to all workspace members...", component_name).blue());
    
    // Use the component name directly as the module name
    let module_name = component_name.to_string();
    
    // Path to the shared component directory
    let shared_dir = project_dir.join(component_name);
    
    // We don't need to modify the shared component's lib.rs file structure
    // as it will be a standard library crate
    
    // Update the shared component's Cargo.toml to use the component name directly
    let cargo_path = shared_dir.join("Cargo.toml");
    if cargo_path.exists() {
        let cargo_content = fs::read_to_string(&cargo_path)?;
        let mut doc = cargo_content.parse::<Document>()
            .context("Failed to parse shared component's Cargo.toml")?;
        
        // Use the component name directly as the package name
        if let Some(package) = doc.get_mut("package") {
            if let Some(name) = package.get_mut("name") {
                *name = Item::Value(Value::from(component_name));
            }
        }
        
        fs::write(&cargo_path, doc.to_string())?;
    }
    
    // Update the root workspace Cargo.toml to include the shared component
    let root_cargo_path = project_dir.join("Cargo.toml");
    if root_cargo_path.exists() {
        let root_cargo_content = fs::read_to_string(&root_cargo_path)?;
        let mut root_doc = root_cargo_content.parse::<Document>()
            .context("Failed to parse root Cargo.toml")?;
        
        // Make sure the workspace section exists
        if root_doc.get("workspace").is_none() {
            root_doc.insert("workspace", Item::Table(Table::new()));
        }
        
        // Make sure the workspace.members section exists
        if let Some(workspace) = root_doc.get_mut("workspace") {
            if workspace.get("members").is_none() {
                let mut array = toml_edit::Array::new();
                array.set_trailing_comma(true);
                workspace.as_table_mut().unwrap().insert("members", Item::Value(Value::Array(array)));
            }
            
            // Add the component to workspace members if not already there
            if let Some(members) = workspace.get_mut("members") {
                if let Some(array) = members.as_array_mut() {
                    // Check if the component is already in the members list
                    let component_str = format!("{}", component_name);
                    let mut found = false;
                    for item in array.iter() {
                        if let Some(member) = item.as_str() {
                            if member == component_name {
                                found = true;
                                break;
                            }
                        }
                    }
                    
                    if !found {
                        array.push(component_str);
                    }
                }
            }
        }
        
        fs::write(&root_cargo_path, root_doc.to_string())?;
    }
    
    // Find all other components in the workspace and add the shared component as a dependency
    for entry in fs::read_dir(project_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip non-directories and the shared component itself
        if !path.is_dir() || path.file_name() == Some(OsStr::new(component_name)) {
            continue;
        }
        
        // Skip directories that don't look like components (e.g., target, .git)
        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if dir_name.starts_with(".") || dir_name == "target" {
            continue;
        }
        
        // Check if this is a component by looking for Cargo.toml
        let component_cargo_path = path.join("Cargo.toml");
        if component_cargo_path.exists() {
            // Add the shared component as a dependency
            let cargo_content = fs::read_to_string(&component_cargo_path)?;
            let mut doc = cargo_content.parse::<Document>()
                .context(format!("Failed to parse {}'s Cargo.toml", dir_name))?;
            
            // Add dependency section if it doesn't exist
            if doc.get("dependencies").is_none() {
                doc.insert("dependencies", Item::Table(Table::new()));
            }
            
            // Add the shared component as a dependency
            if let Some(dependencies) = doc.get_mut("dependencies") {
                if let Some(deps_table) = dependencies.as_table_mut() {
                    let mut shared_dep = Table::new();
                    shared_dep.insert("path", Item::Value(Value::from(format!("../{}", component_name))));
                    deps_table.insert(&module_name, Item::Table(shared_dep));
                }
            }
            
            fs::write(&component_cargo_path, doc.to_string())?;
            println!("{}", format!("Added shared component as dependency to {}", dir_name).green());
            
            // Add the module declaration to the component's lib.rs or main.rs
            let lib_rs_path = path.join("src").join("lib.rs");
            let main_rs_path = path.join("src").join("main.rs");
                        // Function to add the module declaration to a file
            let add_module_declaration = |file_path: &Path| -> Result<()> {
                if file_path.exists() {
                    let mut content = fs::read_to_string(file_path)?;
                    
                    // Create the module declaration using modern Rust 2018 style imports
                    let module_declaration = format!(
                        "\n// Import the shared component\nuse {}::*;\n", 
                        module_name
                    );
                    
                    // Only add if it doesn't already exist
                    if !content.contains(&format!("use {}::", module_name)) {
                        // Find a good place to insert the import
                        if let Some(pos) = content.find("fn ") {
                            // Insert before the first function
                            let (before, after) = content.split_at(pos);
                            content = format!("{}{}{}", before, module_declaration, after);
                        } else {
                            // Just append to the top if no function is found
                            content = format!("{}{}", module_declaration, content);
                        }
                        
                        fs::write(file_path, content)?;
                        println!("{}", format!("Added shared module import to {}", file_path.display()).green());
                    }
                }
                Ok(())
            };
            
            // Try to add the module declaration to lib.rs first, then main.rs if lib.rs doesn't exist
            if lib_rs_path.exists() {
                add_module_declaration(&lib_rs_path)?;
            } else if main_rs_path.exists() {
                add_module_declaration(&main_rs_path)?;
            }
        }
    }
    
    println!("{}", format!("Shared component '{}' is now accessible to all workspace members", component_name).green());
    
    Ok(())
}

// Function to store transformation metadata
fn store_transformation_metadata(
    project_dir: &Path,
    component_name: &str,
    template: &str,
    framework: Option<&str>,
) -> Result<()> {
    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

    let metadata_path = ferrisup_dir.join("metadata.toml");

    // Create or load existing metadata
    let metadata_content = if metadata_path.exists() {
        fs::read_to_string(&metadata_path)?
    } else {
        String::new()
    };

    let mut doc = if metadata_content.is_empty() {
        Document::new()
    } else {
        metadata_content
            .parse::<Document>()
            .context("Failed to parse metadata.toml")?
    };

    // Ensure components table exists
    if doc.get("components").is_none() {
        doc.insert("components", Item::Table(Table::new()));
    }

    // Add component metadata
    let components = doc["components"].as_table_mut().unwrap();

    let mut component_table = Table::new();
    component_table.insert("template", value(template));

    // Explicitly store the component_type based on the template or component name
    let component_type = match template {
        "client" => "client",
        "server" => "server",
        "shared" => "shared",
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        _ => match component_name {
            "client" => "client",
            "server" => "server",
            "shared" => "shared",
            "edge" => "edge",
            "serverless" => "serverless",
            "data-science" => "data-science",
            "embedded" => "embedded",
            _ => template, // Default to template name if no match
        },
    };
    component_table.insert("component_type", value(component_type));

    if let Some(fw) = framework {
        component_table.insert("framework", value(fw));
    }
    component_table.insert("created_at", value(chrono::Local::now().to_rfc3339()));

    components.insert(component_name, Item::Table(component_table));

    // Write metadata back to file
    fs::write(metadata_path, doc.to_string()).context("Failed to write metadata.toml")?;

    Ok(())
}

// Function to print final next steps
fn print_final_next_steps(project_dir: &Path) -> Result<()> {
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;

    if !structure.is_workspace {
        return Ok(());
    }

    // Get workspace members from Cargo.toml
    let workspace_cargo_path = project_dir.join("Cargo.toml");
    let workspace_cargo_content = fs::read_to_string(&workspace_cargo_path)?;
    let workspace_doc = workspace_cargo_content
        .parse::<Document>()
        .context("Failed to parse workspace Cargo.toml")?;

    // Extract component names from workspace members
    let mut component_names = Vec::new();
    if let Some(workspace) = workspace_doc.get("workspace") {
        if let Some(members) = workspace.get("members") {
            if let Some(members_array) = members.as_array() {
                for member in members_array {
                    if let Some(member_str) = member.as_str() {
                        component_names.push(member_str.to_string());
                    }
                }
            }
        }
    }

    // We don't need the project_name anymore

    println!("{}", "\nFinal Steps:\n".green().bold());

    // 1. Navigate to project directory
    println!("{}", "1. Navigate to project directory".blue());
    println!("cd {}", project_dir.display());
    println!();

    // Print comprehensive build instructions
    println!("{}", "Working with your components:".yellow().bold());

    // 2. Build all components at once
    println!("{}", "2. To build all components at once:".blue());
    print!("cargo build");

    // Add individual component build commands
    for component in &component_names {
        print!(" && cargo build -p {}", component);
    }
    println!("\n");

    // 3. Build specific components
    println!("{}", "3. To build specific components:".blue());
    for component in &component_names {
        println!("   cargo build -p {}", component);
    }
    println!();

    // 4. Adding dependencies
    println!("{}", "4. To add dependencies to components:".blue());
    println!("   cd [component_name] && cargo add [dependency_name]");
    println!("   OR");
    println!("   cargo add [dependency_name] --package [component_name]");
    println!();

    // 5. Adding more components
    println!("{}", "5. To add more components in the future:".blue());
    println!("   ferrisup transform");
    println!();

    Ok(())
}

// End of file

// Function to fix imports in a component after updating the package name
