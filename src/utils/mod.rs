use std::fs;
use std::path::Path;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use walkdir::WalkDir;
use toml;
use toml_edit::Item;

pub fn create_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("Created directory: {}", path.display());
    }
    Ok(())
}

#[allow(dead_code)]
pub fn write_cargo_toml(project_dir: &Path) -> Result<()> {
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        // Using a default project name 
        "rust_project"
    );
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)
        .context("Failed to write Cargo.toml")?;
    
    Ok(())
}

#[allow(dead_code)]
pub fn write_env_file(project_dir: &Path) -> Result<()> {
    let env_sample = r#"# Database connection
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_workspace

# Server config
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Client config
CLIENT_API_URL=http://localhost:3000/api

# JWT Secret
JWT_SECRET=your-secret-key-here

# Vector database connection (if using)
VECTOR_DB_URL=http://localhost:8080

# AI model settings (if using)
MODEL_PATH=./ai/models/model.onnx
"#;
    
    fs::write(project_dir.join(".env.sample"), env_sample)
        .context("Failed to write .env.sample")?;
    
    Ok(())
}

#[allow(dead_code)]
pub fn read_cargo_toml(project_dir: &Path) -> Result<String> {
    let cargo_path = project_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        return Err(anyhow::anyhow!("Cargo.toml not found"));
    }
    
    fs::read_to_string(&cargo_path)
        .context(format!("Failed to read {}", cargo_path.display()))
}

#[allow(dead_code)]
pub fn write_cargo_toml_content(project_dir: &Path, content: &str) -> Result<()> {
    let cargo_path = project_dir.join("Cargo.toml");
    
    fs::write(&cargo_path, content)
        .context(format!("Failed to write {}", cargo_path.display()))?;
    
    println!("{} {}", "Updated".green(), cargo_path.display());
    
    Ok(())
}

#[allow(dead_code)]
pub fn update_workspace_members(project_dir: &Path) -> Result<bool> {
    let cargo_content = read_cargo_toml(project_dir)?;
    
    // Parse the TOML content
    let cargo_toml: toml::Value = toml::from_str(&cargo_content)
        .context("Failed to parse Cargo.toml as valid TOML")?;
    
    // Check if it's a workspace
    if cargo_toml.get("workspace").is_none() {
        return Err(anyhow::anyhow!("Not a Cargo workspace (no [workspace] section in Cargo.toml)"));
    }
    
    // Extract existing workspace members
    let mut updated = false;
    let mut existing_members = Vec::new();
    
    if let Some(workspace) = cargo_toml.get("workspace").and_then(|w| w.as_table()) {
        if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
            for member in members {
                if let Some(member_str) = member.as_str() {
                    existing_members.push(member_str.to_string());
                }
            }
        }
    }
    
    // Discover crates in the project directory
    let mut crates_to_add = Vec::new();
    
    // Check common workspace directories
    for dir in &["client", "server", "shared", "libs", "crates"] {
        let dir_path = project_dir.join(dir);
        if dir_path.exists() && dir_path.is_dir() {
            // Check if we have the wildcard pattern already
            let wildcard = format!("{}/*", dir);
            if !existing_members.contains(&wildcard) && !existing_members.iter().any(|m| m.starts_with(&format!("{}/", dir))) {
                // Look for individual crates
                for entry in fs::read_dir(&dir_path).context(format!("Failed to read directory {}", dir_path.display()))? {
                    let entry = entry.context("Failed to read directory entry")?;
                    let path = entry.path();
                    
                    if path.is_dir() && path.join("Cargo.toml").exists() {
                        let relative_path = format!("{}/{}", dir, path.file_name().unwrap().to_string_lossy());
                        if !existing_members.contains(&relative_path) {
                            crates_to_add.push(relative_path);
                        }
                    }
                }
            }
        }
    }
    
    // Add root level crates
    for entry in fs::read_dir(project_dir).context("Failed to read project directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        if path.is_dir() && path.join("Cargo.toml").exists() {
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            
            // Skip common directories that might contain multiple crates and system directories
            if ![
                "src", "target", ".git", ".github", ".ferrisup"
            ].contains(&dir_name.as_str()) && !existing_members.contains(&dir_name) {
                crates_to_add.push(dir_name);
            }
        }
    }
    
    // If we found new crates, update the Cargo.toml
    if !crates_to_add.is_empty() {
        updated = true;
        
        // Create a new TOML structure with updated members
        let mut new_cargo = cargo_toml.clone();
        
        // Get or create the workspace table
        let workspace = new_cargo.get_mut("workspace")
            .and_then(|w| w.as_table_mut())
            .expect("Workspace section should exist");
        
        // Get or create the members array
        let members = if let Some(members) = workspace.get_mut("members").and_then(|m| m.as_array_mut()) {
            members
        } else {
            workspace.insert("members".to_string(), toml::Value::Array(Vec::new()));
            workspace.get_mut("members").and_then(|m| m.as_array_mut()).unwrap()
        };
        
        // Add new crates
        for crate_path in crates_to_add {
            println!("Adding workspace member: {}", crate_path.green());
            members.push(toml::Value::String(crate_path.to_string()));
        }
        
        // Write the updated TOML back to the file
        let updated_content = toml::to_string(&new_cargo)
            .context("Failed to serialize updated Cargo.toml")?;
        
        write_cargo_toml_content(project_dir, &updated_content)?;
    }
    
    Ok(updated)
}

#[allow(dead_code)]
pub fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    // Create the destination directory if it doesn't exist
    create_directory(dst)?;
    
    // Use a robust directory traversal to handle potential symlinks
    let walker = WalkDir::new(src).follow_links(true).into_iter();
    
    // Filter out errors in directory traversal
    let walker = walker.filter_map(|e| e.ok());
    
    for entry in walker {
        let path = entry.path();
        // Skip the root directory itself
        if path == src {
            continue;
        }
        
        // Get the path relative to the source directory
        let relative = path.strip_prefix(src)?;
        let target = dst.join(relative);
        
        if path.is_file() {
            // Create parent directories if needed
            if let Some(parent) = target.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            
            // Copy the file
            std::fs::copy(path, &target)
                .with_context(|| format!("Failed to copy {} to {}", path.display(), target.display()))?;
            
            println!("Copied: {} -> {}", path.display(), target.display());
        } else if path.is_dir() && !target.exists() {
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(&target)
                .with_context(|| format!("Failed to create directory {}", target.display()))?;
        }
    }
    
    println!("{} {} {} {}", 
        "Successfully copied".green(),
        src.display(),
        "to".green(),
        dst.display());
    
    Ok(())
}

// Helper function to extract dependencies from a TOML table
#[allow(dead_code)]
pub fn extract_dependencies(deps_table: &Item) -> Result<Vec<(String, String, Option<Vec<String>>)>> {
    let mut dependencies = Vec::new();

    if let Some(deps_table) = deps_table.as_table() {
        for (name, value) in deps_table.iter() {
            if let Some(version) = value.as_str() {
                // Simple version string without features
                dependencies.push((name.to_string(), version.to_string(), None));
            } else if let Some(table) = value.as_table() {
                if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                    // Check for features
                    let mut features = Vec::new();
                    if let Some(features_value) = table.get("features").and_then(|f| f.as_array()) {
                        for feature in features_value {
                            if let Some(feature_str) = feature.as_str() {
                                features.push(feature_str.to_string());
                            }
                        }
                    }
                    
                    let features_option = if features.is_empty() { None } else { Some(features) };
                    dependencies.push((name.to_string(), version.to_string(), features_option));
                }
            }
        }
    }

    Ok(dependencies)
}

// Helper function to update Cargo.toml with dependencies using cargo add
#[allow(dead_code)]
pub fn update_cargo_with_dependencies(cargo_path: &Path, dependencies: Vec<(String, String, Option<Vec<String>>)>) -> Result<()> {
    // Get the project directory (parent of the Cargo.toml file)
    let project_dir = cargo_path.parent().ok_or_else(|| anyhow!("Could not determine project directory"))?;
    
    // Save current directory to return to it after
    let current_dir = std::env::current_dir()?;
    
    // Change to project directory to run cargo add
    std::env::set_current_dir(project_dir)?;
    
    for (name, version, features) in dependencies {
        // Build cargo add command
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("add").arg(&name);
        
        // Add version if it's not "*"
        if version != "*" {
            cmd.arg("--vers").arg(&version);
        }
        
        // Add features if provided
        if let Some(feat_list) = features {
            if !feat_list.is_empty() {
                let features_str = feat_list.join(",");
                cmd.arg("--features").arg(features_str);
            }
        }
        
        // Run the command
        let output = cmd.output()
            .context(format!("Failed to add dependency: {}", name))?;
        
        if !output.status.success() {
            println!("{} {}", 
                "Warning:".yellow().bold(), 
                format!("Failed to add dependency: {}", name).yellow());
            
            // Print error message if available
            if let Ok(err) = String::from_utf8(output.stderr) {
                if !err.is_empty() {
                    println!("{}", err);
                }
            }
        }
    }
    
    // Change back to original directory
    std::env::set_current_dir(current_dir)?;
    
    Ok(())
}

// Helper function to copy directory contents
#[allow(dead_code)]
pub fn copy_dir_contents(from: &Path, to: &Path) -> Result<()> {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if file_type.is_dir() {
            create_directory(&to_path)?;
            copy_dir_contents(&from_path, &to_path)?;
        } else {
            fs::copy(&from_path, &to_path)?;
        }
    }

    Ok(())
}