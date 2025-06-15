use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::path::Path;
use crate::commands::test_mode::{is_test_mode, test_mode_or};

// Print the banner for the transform command
pub fn print_banner() {
    println!(
        "{}",
        "FerrisUp Interactive Project Transformer".bold().green()
    );
    println!(
        "{}",
        "Transform your existing Rust project with new features".blue()
    );
}

// Print an error message
pub fn print_error(message: String) {
    println!(
        "{} {}",
        "Error:".red().bold(),
        message.red()
    );
}

// Confirm an action with the user
pub fn confirm_action(prompt: &str, default: bool) -> Result<bool> {
    if is_test_mode() {
        return Ok(default);
    }
    
    Ok(Confirm::new()
        .with_prompt(prompt)
        .default(default)
        .interact()?)
}

// Select an option from a list
pub fn select_option(prompt: &str, options: &[&str], default: usize) -> Result<usize> {
    if is_test_mode() {
        return Ok(default);
    }
    
    Ok(Select::new()
        .with_prompt(prompt)
        .items(options)
        .default(default)
        .interact()?)
}

// Multi-select options from a list
pub fn multi_select_options(prompt: &str, options: &[String], defaults: &[bool]) -> Result<Vec<usize>> {
    if is_test_mode() {
        return Ok(defaults.iter().enumerate()
            .filter_map(|(i, &selected)| if selected { Some(i) } else { None })
            .collect());
    }
    
    Ok(MultiSelect::new()
        .with_prompt(prompt)
        .items(options)
        .defaults(defaults)
        .interact()?)
}

// Get input with a default value
pub fn get_input_with_default(prompt: &str, default: &str) -> Result<String> {
    test_mode_or(default.to_string(), || {
        Input::<String>::new()
            .with_prompt(prompt)
            .default(default.to_string())
            .interact_text()
            .map_err(anyhow::Error::from)
    })
}

// Print final next steps after transformation
pub fn print_final_next_steps(project_dir: &Path) -> Result<()> {
    println!("\n{}", "Next Steps:".bold().green());
    println!("1. {} {}", "Build your project:".yellow(), "cargo build".cyan());
    println!("2. {} {}", "Run tests:".yellow(), "cargo test".cyan());
    
    // Check if this is a workspace
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(cargo_toml_path) {
        if content.contains("[workspace]") {
            println!("3. {} {}", "Add more components:".yellow(), "ferrisup transform".cyan());
        }
    }
    
    println!("\n{}", "Happy coding with FerrisUp!".green().bold());
    Ok(())
}

// Function to create a root-level README.md with project structure description
pub fn create_root_readme(project_dir: &Path, component_name: &str) -> Result<()> {
    let readme_path = project_dir.join("README.md");
    
    // If README.md already exists, back it up
    if readme_path.exists() {
        let backup_path = project_dir.join("README.md.bak");
        println!("{} {}", "Backing up existing README.md to".yellow(), backup_path.display().to_string().yellow());
        std::fs::copy(&readme_path, &backup_path)?;
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
    
    std::fs::write(readme_path, readme_content)?;
    Ok(())
}

// Function to create a root-level .gitignore with standard Rust workspace patterns
pub fn create_root_gitignore(project_dir: &Path) -> Result<()> {
    let gitignore_path = project_dir.join(".gitignore");
    
    if gitignore_path.exists() {
        // Back up existing .gitignore
        let backup_path = project_dir.join(".gitignore.bak");
        println!("{} {}", "Backing up existing .gitignore to".yellow(), backup_path.display().to_string().yellow());
        std::fs::copy(&gitignore_path, &backup_path)?;
        
        // Read existing content
        let existing_content = std::fs::read_to_string(&gitignore_path)?;
        
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
        
        std::fs::write(gitignore_path, gitignore_content)?;
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

        std::fs::write(gitignore_path, gitignore_content)?;
    }
    
    Ok(())
}
