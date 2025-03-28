use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

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

pub fn read_cargo_toml(project_dir: &Path) -> Result<String> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    
    if !cargo_toml_path.exists() {
        return Ok("".to_string());
    }
    
    let cargo_toml = fs::read_to_string(&cargo_toml_path)
        .context(format!("Failed to read Cargo.toml at {:?}", cargo_toml_path))?;
    
    Ok(cargo_toml)
}

pub fn update_workspace_members(project_dir: &Path) -> Result<()> {
    println!("{}", "Updating workspace members...".blue());
    
    // Read existing Cargo.toml
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        println!("{} {}", 
            "Warning:".yellow().bold(),
            "No Cargo.toml found at project root".yellow());
        return Ok(());
    }
    
    let cargo_toml = fs::read_to_string(&cargo_toml_path)
        .context("Failed to read Cargo.toml")?;
    
    // Check if it's a workspace
    if !cargo_toml.contains("[workspace]") {
        println!("{} {}", 
            "Warning:".yellow().bold(),
            "Not a workspace Cargo.toml".yellow());
        return Ok(());
    }
    
    // Find all potential workspace members
    let mut members = Vec::new();
    
    // Check for the main src directory
    if project_dir.join("src").exists() {
        members.push("src".to_string());
    }
    
    // Check for client apps
    if project_dir.join("client").exists() {
        for entry in fs::read_dir(project_dir.join("client"))? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.path().file_name() {
                    if let Some(name_str) = name.to_str() {
                        members.push(format!("client/{}", name_str));
                    }
                }
            }
        }
    }
    
    // Check for server services
    if project_dir.join("server").exists() {
        for entry in fs::read_dir(project_dir.join("server"))? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.path().file_name() {
                    if let Some(name_str) = name.to_str() {
                        members.push(format!("server/{}", name_str));
                    }
                }
            }
        }
    }
    
    // Check for libs
    if project_dir.join("libs").exists() {
        for entry in fs::read_dir(project_dir.join("libs"))? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.path().file_name() {
                    if let Some(name_str) = name.to_str() {
                        members.push(format!("libs/{}", name_str));
                    }
                }
            }
        }
    }
    
    // Check for binaries
    if project_dir.join("binaries").exists() {
        for entry in fs::read_dir(project_dir.join("binaries"))? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.path().file_name() {
                    if let Some(name_str) = name.to_str() {
                        members.push(format!("binaries/{}", name_str));
                    }
                }
            }
        }
    }
    
    // Check for AI models
    if project_dir.join("ai").exists() {
        for entry in fs::read_dir(project_dir.join("ai"))? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.path().file_name() {
                    if let Some(name_str) = name.to_str() {
                        members.push(format!("ai/{}", name_str));
                    }
                }
            }
        }
    }
    
    // Check for edge apps
    if project_dir.join("edge").exists() {
        for entry in fs::read_dir(project_dir.join("edge"))? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.path().file_name() {
                    if let Some(name_str) = name.to_str() {
                        members.push(format!("edge/{}", name_str));
                    }
                }
            }
        }
    }
    
    // Check for embedded devices
    if project_dir.join("embedded").exists() {
        for entry in fs::read_dir(project_dir.join("embedded"))? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(name) = entry.path().file_name() {
                    if let Some(name_str) = name.to_str() {
                        members.push(format!("embedded/{}", name_str));
                    }
                }
            }
        }
    }
    
    // Generate updated Cargo.toml with members
    let workspace_section = format!(
        r#"[workspace]
members = [
{}
]

[workspace.dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
thiserror = "1.0"
anyhow = "1.0"
"#,
        members.iter()
            .map(|m| format!("    \"{}\"", m))
            .collect::<Vec<_>>()
            .join(",\n")
    );
    
    // Update the members section in the original Cargo.toml
    let updated_cargo_toml = if let Some(start) = cargo_toml.find("[workspace]") {
        if let Some(end) = cargo_toml[start..].find("[workspace.dependencies]") {
            let before = &cargo_toml[..start];
            let after = &cargo_toml[start + end..];
            format!("{}{}{}", before, workspace_section, after)
        } else {
            workspace_section
        }
    } else {
        workspace_section
    };
    
    // Write updated Cargo.toml
    fs::write(cargo_toml_path, updated_cargo_toml)
        .context("Failed to write updated Cargo.toml")?;
    
    println!("{} {}", 
        "Successfully updated".green(),
        format!("{} workspace members", members.len()).green());
    
    Ok(())
}

pub fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    create_directory(dst.to_str().unwrap())?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(src)?;
        let target = dst.join(relative);
        
        if path.is_dir() {
            create_directory(target.to_str().unwrap())?;
        } else {
            fs::copy(path, target)?;
        }
    }
    
    Ok(())
}
