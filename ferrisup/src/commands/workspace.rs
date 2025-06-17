use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use std::path::{Path, PathBuf};
use std::fs;
use ferrisup_common::{fs::create_directory, cargo::*};


/// Execute the workspace command to manage Cargo workspaces
pub fn execute(action: Option<&str>, path: Option<&str>) -> Result<()> {
    println!("{}", "FerrisUp Workspace Manager".bold().green());
    
    // Get project path
    let project_dir = if let Some(p) = path {
        PathBuf::from(p)
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
    
    // Get action
    let action_str = if let Some(act) = action {
        act.to_string()
    } else {
        let options = vec!["init", "add", "remove", "list", "optimize"];
        let selection = Select::new()
            .with_prompt("Select workspace action")
            .items(&options)
            .default(0)
            .interact()?;
        
        options[selection].to_string()
    };
    
    // Execute the selected action
    match action_str.as_str() {
        "init" => init_workspace(&project_dir)?,
        "add" => add_crate_to_workspace(&project_dir)?,
        "remove" => remove_crate_from_workspace(&project_dir)?,
        "list" => list_workspace_members(&project_dir)?,
        "optimize" => optimize_workspace(&project_dir)?,
        _ => return Err(anyhow::anyhow!("Invalid action. Use 'init', 'add', 'remove', 'list', or 'optimize'")),
    }
    
    Ok(())
}

/// Initialize a new Cargo workspace
fn init_workspace(project_dir: &Path) -> Result<()> {
    // Check if Cargo.toml exists
    let cargo_toml_path = project_dir.join("Cargo.toml");
    
    let workspace_exists = if cargo_toml_path.exists() {
        let content = read_cargo_toml(project_dir)?;
        content.contains("[workspace]")
    } else {
        false
    };
    
    if workspace_exists {
        println!("{}", "Workspace already initialized!".yellow());
        return Ok(());
    }
    
    // Ask for workspace members
    let default_dirs = vec![
        "client_old/*".to_string(),
        "server/*".to_string(),
        "shared/*".to_string(),
    ];
    
    let mut dirs = if !cargo_toml_path.exists() {
        // New workspace from scratch
        default_dirs
    } else {
        // Convert existing project to workspace
        println!("\n{}", "Converting existing project to workspace".green());
        
        let options = vec![
            "Use default workspace structure (client_old/*, server/*, shared/*)",
            "Discover existing crates",
            "Manually specify members",
        ];
        
        let selection = Select::new()
            .with_prompt("How would you like to initialize the workspace?")
            .items(&options)
            .default(0)
            .interact()?;
        
        match selection {
            0 => default_dirs,
            1 => discover_crates(project_dir)?,
            2 => {
                let input = Input::<String>::new()
                    .with_prompt("Enter comma-separated workspace members (e.g. 'crate1, crate2/*, shared/*')")
                    .interact()?;
                
                input.split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            },
            _ => default_dirs,
        }
    };
    
    // Create the workspace Cargo.toml
    let cargo_content = if !cargo_toml_path.exists() {
        // Create new Cargo.toml
        format!(
            r#"[workspace]
members = [
{}
]

[workspace.dependencies]
# Common dependencies for workspace members
anyhow = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
log = "0.4"
"#,
            dirs.iter()
                .map(|dir| format!("    \"{}\",", dir))
                .collect::<Vec<String>>()
                .join("\n")
        )
    } else {
        // Modify existing Cargo.toml
        let content = read_cargo_toml(project_dir)?;
        
        // Preserve existing content and add workspace section
        if content.contains("[package]") {
            // Convert an application to a workspace root
            // First, move package section to its own crate
            let package_name = extract_package_name(&content).unwrap_or("app".to_string());
            
            // Create app directory for the existing package
            let app_dir = project_dir.join(&package_name);
            if !app_dir.exists() {
                create_directory(&app_dir)?;
                
                // Move existing src directory to app directory
                let src_dir = project_dir.join("src");
                if src_dir.exists() {
                    let target_dir = app_dir.join("src");
                    fs::rename(&src_dir, &target_dir)?;
                }
                
                // Create app Cargo.toml with the package section
                let app_cargo = app_dir.join("Cargo.toml");
                fs::write(&app_cargo, extract_package_section(&content))?;
                
                println!("{} {}", "Moved existing package to:".green(), app_dir.display());
                
                // Add the new crate to workspace members
                dirs.push(package_name);
            }
            
            // Create new root Cargo.toml
            format!(
                r#"[workspace]
members = [
{}
]

[workspace.dependencies]
# Common dependencies for workspace members
anyhow = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
log = "0.4"
"#,
                dirs.iter()
                    .map(|dir| format!("    \"{}\",", dir))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        } else {
            // Just add workspace section to existing Cargo.toml
            format!(
                r#"{}

[workspace]
members = [
{}
]

[workspace.dependencies]
# Common dependencies for workspace members
anyhow = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
log = "0.4"
"#,
                content,
                dirs.iter()
                    .map(|dir| format!("    \"{}\",", dir))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        }
    };
    
    // Write the Cargo.toml file
    write_cargo_toml_content(project_dir, &cargo_content)?;
    
    println!("{} {}", "Initialized workspace in:".green(), project_dir.display());
    println!("{} {}", "Workspace members:".green(), dirs.join(", "));
    
    // Create default directories if they don't exist
    for dir in &["client_old", "server", "shared"] {
        let path = project_dir.join(dir);
        if !path.exists() {
            create_directory(&path)?;
            println!("{} {}", "Created directory:".green(), path.display());
        }
    }
    
    Ok(())
}

/// Add a new crate to an existing workspace
fn add_crate_to_workspace(project_dir: &Path) -> Result<()> {
    // Verify it's a workspace
    let cargo_content = read_cargo_toml(project_dir)?;
    if !cargo_content.contains("[workspace]") {
        return Err(anyhow::anyhow!("Not a Cargo workspace (no [workspace] section in Cargo.toml)"));
    }
    
    // Get crate type
    let crate_types = vec!["client_old", "server", "shared", "custom"];
    let selection = Select::new()
        .with_prompt("Select crate type")
        .items(&crate_types)
        .default(0)
        .interact()?;
    
    let crate_type = crate_types[selection];
    
    // Get crate name
    let crate_name = Input::<String>::new()
        .with_prompt("Enter crate name")
        .interact()?;
    
    // Determine crate path based on type
    let crate_path = match crate_type {
        "client_old" => project_dir.join("../../../client_old").join(&crate_name),
        "server" => project_dir.join("server").join(&crate_name),
        "shared" => project_dir.join("shared").join(&crate_name),
        _ => project_dir.join(&crate_name),
    };
    
    // Create crate directory
    create_directory(&crate_path)?;
    
    // Get crate template
    let is_bin = if crate_type == "client_old" || crate_type == "server" {
        true
    } else {
        Confirm::new()
            .with_prompt("Is this a binary crate? (No for library)")
            .default(false)
            .interact()?
    };
    
    // Create src directory and main.rs/lib.rs
    let src_dir = crate_path.join("src");
    create_directory(&src_dir)?;
    
    if is_bin {
        fs::write(
            src_dir.join("main.rs"),
            "fn main() {\n    println!(\"Hello from {}!\");\n}\n".replace("{}", &crate_name)
        )?;
    } else {
        fs::write(
            src_dir.join("lib.rs"),
            "//! {} library\n\n/// Example function\npub fn hello() -> &'static str {\n    \"Hello from {}!\"\n}\n"
                .replace("{}", &crate_name)
        )?;
    }
    
    // Create Cargo.toml for the crate
    let crate_cargo_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        if crate_type == "custom" {
            crate_name.clone()
        } else {
            let project_name = project_dir.file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.replace('-', "_"))
                .unwrap_or_else(|| "project".to_string());
            format!("{}-{}", project_name, crate_name)
        }
    );
    
    fs::write(crate_path.join("Cargo.toml"), crate_cargo_content)?;
    
    println!("{} {}", "Created crate:".green(), crate_path.display());
    
    // Update workspace members
    update_workspace_members(project_dir)?;
    
    Ok(())
}

/// Remove a crate from an existing workspace
fn remove_crate_from_workspace(project_dir: &Path) -> Result<()> {
    // Verify it's a workspace
    let cargo_content = read_cargo_toml(project_dir)?;
    if !cargo_content.contains("[workspace]") {
        return Err(anyhow::anyhow!("Not a Cargo workspace (no [workspace] section in Cargo.toml)"));
    }
    
    // List workspace members
    let members = list_workspace_crates(project_dir)?;
    
    if members.is_empty() {
        println!("{}", "No workspace members found".yellow());
        return Ok(());
    }
    
    // Let user select a crate to remove
    let selection = Select::new()
        .with_prompt("Select crate to remove from workspace")
        .items(&members)
        .default(0)
        .interact()?;
    
    let crate_path = members[selection].clone();
    
    // Confirm deletion
    let delete_files = Confirm::new()
        .with_prompt(format!("Also delete {} files?", crate_path))
        .default(false)
        .interact()?;
    
    // Remove files if requested
    if delete_files {
        let full_path = project_dir.join(&crate_path);
        fs::remove_dir_all(&full_path)
            .context(format!("Failed to remove {}", full_path.display()))?;
        
        println!("{} {}", "Deleted crate files:".green(), crate_path);
    }
    
    // Update workspace members
    update_workspace_members(project_dir)?;
    
    println!("{} {}", "Removed crate from workspace:".green(), crate_path);
    
    Ok(())
}

/// List members of an existing workspace
fn list_workspace_members(project_dir: &Path) -> Result<()> {
    // Verify it's a workspace
    let cargo_content = read_cargo_toml(project_dir)?;
    if !cargo_content.contains("[workspace]") {
        return Err(anyhow::anyhow!("Not a Cargo workspace (no [workspace] section in Cargo.toml)"));
    }
    
    // Extract workspace members
    let members = extract_workspace_members(&cargo_content);
    
    println!("\n{}", "Workspace Members:".bold());
    
    if members.is_empty() {
        println!("  No members found");
    } else {
        for (i, member) in members.iter().enumerate() {
            println!("  {}. {}", i + 1, member);
        }
    }
    
    // List actual crates found (resolved)
    let crates = list_workspace_crates(project_dir)?;
    
    println!("\n{}", "Found Crates:".bold());
    
    if crates.is_empty() {
        println!("  No crates found");
    } else {
        for (i, crate_path) in crates.iter().enumerate() {
            println!("  {}. {}", i + 1, crate_path);
        }
    }
    
    Ok(())
}

/// Optimize a workspace by identifying and fixing common issues
fn optimize_workspace(project_dir: &Path) -> Result<()> {
    println!("{}", "Optimizing workspace...".green());
    
    // Verify it's a workspace
    let cargo_content = read_cargo_toml(project_dir)?;
    if !cargo_content.contains("[workspace]") {
        return Err(anyhow::anyhow!("Not a Cargo workspace (no [workspace] section in Cargo.toml)"));
    }
    
    // Check for duplicated dependencies
    let mut improvements = Vec::new();
    improvements.push("Checking for dependency issues...".to_string());
    
    // Update workspace members - ensure all crates are included
    let updated = update_workspace_members(project_dir)?;
    if updated {
        improvements.push("✓ Updated workspace members list".to_string());
    }
    
    // Check if workspace.dependencies exists and add if not
    if !cargo_content.contains("[workspace.dependencies]") {
        // Add workspace.dependencies section header only
        let updated_content = format!(
            r#"{}\n
[workspace.dependencies]
# Common dependencies for workspace members
"#,
            cargo_content
        );
        
        // Write updated Cargo.toml with just the section header
        write_cargo_toml_content(project_dir, &updated_content)?;
        
        // Now add common dependencies using our utility function
        let common_deps = vec![
            ("anyhow".to_string(), "1.0".to_string(), None),
            ("serde".to_string(), "1.0".to_string(), Some(vec!["derive".to_string()])),
            ("log".to_string(), "0.4".to_string(), None)
        ];
        
        let cargo_path = project_dir.join("Cargo.toml");
        update_cargo_with_dependencies(&cargo_path, common_deps, false)?;
        improvements.push("✓ Added [workspace.dependencies] section".to_string());
    }
    
    // Report improvements
    println!("\n{}", "Workspace Optimization Results:".bold());
    for improvement in improvements {
        println!("  {}", improvement);
    }
    
    println!("\n{}", "Workspace optimized successfully!".green());
    
    Ok(())
}

/// Helper function to discover crates in a project directory
fn discover_crates(project_dir: &Path) -> Result<Vec<String>> {
    let mut crates = Vec::new();
    
    // Look for directories with Cargo.toml files
    let walkdir = walkdir::WalkDir::new(project_dir)
        .follow_links(true)
        .max_depth(3) // Don't go too deep
        .into_iter()
        .filter_map(|e| e.ok());
    
    for entry in walkdir {
        let path = entry.path();
        
        // Skip the root directory itself
        if path == project_dir {
            continue;
        }
        
        // Check if it has a Cargo.toml file
        if path.is_dir() && path.join("Cargo.toml").exists() {
            if let Ok(rel_path) = path.strip_prefix(project_dir) {
                let rel_path_str = rel_path.to_string_lossy().to_string();
                crates.push(rel_path_str);
            }
        }
    }
    
    // If nothing found in subdirectories, check for common patterns
    if crates.is_empty() {
        // Check for client_old/server/shared directories
        for dir in &["client_old", "server", "shared"] {
            let dir_path = project_dir.join(dir);
            if dir_path.exists() && dir_path.is_dir() {
                crates.push(format!("{}/*", dir));
            }
        }
    }
    
    Ok(crates)
}

/// Helper function to list actual crates in a workspace
fn list_workspace_crates(project_dir: &Path) -> Result<Vec<String>> {
    let mut crates = Vec::new();
    
    // Extract workspace members
    let cargo_content = read_cargo_toml(project_dir)?;
    let members = extract_workspace_members(&cargo_content);
    
    // Resolve glob patterns and check if each member exists
    for member in members {
        if member.contains('*') {
            // Handle glob pattern
            let parts: Vec<&str> = member.split('*').collect();
            let prefix = parts[0];
            
            let prefix_path = project_dir.join(prefix);
            if prefix_path.exists() && prefix_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&prefix_path) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        if entry.path().is_dir() && entry.path().join("Cargo.toml").exists() {
                            if let Ok(rel_path) = entry.path().strip_prefix(project_dir) {
                                crates.push(rel_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        } else {
            // Handle direct path
            let member_path = project_dir.join(&member);
            if member_path.exists() && member_path.is_dir() && member_path.join("Cargo.toml").exists() {
                crates.push(member);
            }
        }
    }
    
    Ok(crates)
}

/// Helper function to extract workspace members from Cargo.toml content
fn extract_workspace_members(cargo_content: &str) -> Vec<String> {
    let mut members = Vec::new();
    
    // Basic parsing of members array
    if let Some(workspace_section) = cargo_content.split("[workspace]").nth(1) {
        if let Some(members_section) = workspace_section.split("members").nth(1) {
            if let Some(members_list) = members_section.split('[').nth(1) {
                if let Some(members_list) = members_list.split(']').next() {
                    for line in members_list.lines() {
                        let line = line.trim();
                        if line.starts_with('"') && line.contains('"') {
                            let member = line
                                .trim_start_matches('"')
                                .split('"')
                                .next()
                                .unwrap_or("")
                                .trim()
                                .trim_end_matches(',');
                            
                            if !member.is_empty() {
                                members.push(member.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    members
}

/// Helper function to extract package name from Cargo.toml content
fn extract_package_name(cargo_content: &str) -> Option<String> {
    if let Some(package_section) = cargo_content.split("[package]").nth(1) {
        if let Some(name_line) = package_section
            .lines()
            .find(|line| line.trim().starts_with("name"))
        {
            if let Some(name) = name_line
                .split('=')
                .nth(1)
                .map(|s| s.trim())
                .map(|s| s.trim_matches('"'))
                .map(|s| s.trim_matches('\''))
            {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Helper function to extract the package section from Cargo.toml content
fn extract_package_section(cargo_content: &str) -> String {
    if let Some(package_section) = cargo_content.split("[package]").nth(1) {
        if let Some(end_index) = package_section.find('[') {
            let section = &package_section[..end_index];
            return format!("[package]{}", section);
        } else {
            return format!("[package]{}", package_section);
        }
    }
    String::new()
}
