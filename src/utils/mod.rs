use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;

pub fn create_directory(path: &str) -> Result<()> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)
            .context(format!("Failed to create directory: {}", path))?;
    }
    Ok(())
}

pub fn write_cargo_toml(project_dir: &Path, config: &crate::config::Config) -> Result<()> {
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        config.project_name
    );
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)
        .context("Failed to write Cargo.toml")?;
    
    Ok(())
}

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

/// Read the contents of a Cargo.toml file
pub fn read_cargo_toml(project_dir: &Path) -> Result<String> {
    let cargo_path = project_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        return Err(anyhow::anyhow!("Cargo.toml not found"));
    }
    
    fs::read_to_string(&cargo_path)
        .context(format!("Failed to read {}", cargo_path.display()))
}

/// Write content to a Cargo.toml file
pub fn write_cargo_toml_content(project_dir: &Path, content: &str) -> Result<()> {
    let cargo_path = project_dir.join("Cargo.toml");
    
    fs::write(&cargo_path, content)
        .context(format!("Failed to write {}", cargo_path.display()))?;
    
    println!("{} {}", "Updated".green(), cargo_path.display());
    
    Ok(())
}

/// Update workspace members by scanning for crates and updating Cargo.toml
pub fn update_workspace_members(project_dir: &Path) -> Result<bool> {
    let cargo_content = read_cargo_toml(project_dir)?;
    
    if !cargo_content.contains("[workspace]") {
        return Err(anyhow::anyhow!("Not a Cargo workspace (no [workspace] section in Cargo.toml)"));
    }
    
    // Extract existing workspace members
    let mut updated = false;
    let mut existing_members = Vec::new();
    
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
                                existing_members.push(member.to_string());
                            }
                        }
                    }
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
                crates_to_add.push(wildcard);
                updated = true;
            }
        }
    }
    
    // Also check for immediate subdirectories with Cargo.toml
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if path.join("Cargo.toml").exists() {
                    // It's a crate
                    if let Ok(rel_path) = path.strip_prefix(project_dir) {
                        let rel_path_str = rel_path.to_string_lossy().to_string();
                        if !existing_members.contains(&rel_path_str) && 
                           !existing_members.iter().any(|m| rel_path_str.starts_with(m.trim_end_matches('*').trim_end_matches('/'))) {
                            crates_to_add.push(rel_path_str);
                            updated = true;
                        }
                    }
                }
            }
        }
    }
    
    if updated {
        // Add new crates to the workspace
        let mut new_members = existing_members.clone();
        new_members.extend(crates_to_add);
        
        // Update Cargo.toml
        let members_str = new_members.iter()
            .map(|m| format!("    \"{}\",", m))
            .collect::<Vec<String>>()
            .join("\n");
        
        // Replace members section
        let re = Regex::new(r"members\s*=\s*\[\s*[\s\S]*?\]").unwrap();
        let new_members_section = format!("members = [\n{}\n]", members_str);
        
        let new_content = if let Some(caps) = re.captures(&cargo_content) {
            cargo_content.replace(caps.get(0).unwrap().as_str(), &new_members_section)
        } else {
            // If regex fails, just append it after [workspace]
            let parts: Vec<&str> = cargo_content.splitn(2, "[workspace]").collect();
            if parts.len() == 2 {
                format!("{}[workspace]\n{}\n{}", parts[0], new_members_section, parts[1])
            } else {
                cargo_content
            }
        };
        
        write_cargo_toml_content(project_dir, &new_content)?;
        
        println!("{} {}", "Updated workspace members:".green(), new_members.join(", "));
    }
    
    Ok(updated)
}

/// Recursively copy a directory and all its contents
#[allow(dead_code)]
pub fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    // Create the destination directory if it doesn't exist
    create_directory(dst.to_str().unwrap())?;
    
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
