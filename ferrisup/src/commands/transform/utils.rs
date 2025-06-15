use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use toml_edit::{value, DocumentMut};
use shared::fs::{create_directory, visit_dirs};
use regex::Regex;
use colored::Colorize;


// Function to update source imports to use the new package name
pub fn update_source_imports(component_dir: &Path, old_name: &str, new_name: &str) -> Result<()> {
    let src_dir = component_dir.join("src");
    if !src_dir.exists() {
        return Ok(());
    }

    visit_dirs(&src_dir, &|file| {
        if let Some(extension) = file.extension() {
            if extension == "rs" {
                if let Ok(content) = fs::read_to_string(file) {
                    // Replace imports like `use old_name::` with `use new_name::`
                    let old_import = format!("use {}::", old_name);
                    let new_import = format!("use {}::", new_name);
                    let updated_content = content.replace(&old_import, &new_import);

                    // Replace imports like `crate::` with `new_name::`
                    // This is more complex and might need regex for better accuracy
                    let crate_regex = Regex::new(r"crate::").unwrap();
                    let updated_content_str = crate_regex.replace_all(&updated_content, format!("{}", new_name).as_str());

                    if content != updated_content_str {
                        // Convert Cow<'_, str> to String explicitly
                        let updated_string = updated_content_str.into_owned();
                        fs::write(file, updated_string)?;
                    }
                }
            }
        }
        Ok(())
    })?;

    Ok(())
}



// Store transformation metadata in .ferrisup directory
pub fn store_transformation_metadata(
    project_dir: &Path, 
    component_name: &str, 
    template: &str, 
    framework: Option<&str>
) -> Result<()> {
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

    let metadata_path = ferrisup_dir.join("metadata.toml");
    
    // Create or update metadata file
    let metadata_content = if metadata_path.exists() {
        fs::read_to_string(&metadata_path)?
    } else {
        String::new()
    };

    let mut metadata_doc = metadata_content
        .parse::<DocumentMut>()
        .unwrap_or_else(|_| DocumentMut::new());

    // Add component metadata
    let component_key = format!("component.{}", component_name);
    
    // Create the table if it doesn't exist
    if !metadata_doc.contains_key(&component_key) {
        metadata_doc[&component_key] = toml_edit::Item::Table(toml_edit::Table::new());
    }
    
    // Now we can safely access the table
    if let Some(table) = metadata_doc[&component_key].as_table_mut() {
        table.insert("template", value(template));
        if let Some(fw) = framework {
            table.insert("framework", value(fw));
        }
        table.insert("created_at", value(chrono::Local::now().to_rfc3339()));
    }

    // Write updated metadata
    fs::write(metadata_path, metadata_doc.to_string())?;
    
    Ok(())
}

// Store component type in component's Cargo.toml metadata
pub fn store_component_type_in_cargo(component_dir: &Path, component_type: &str) -> Result<()> {
    let cargo_path = component_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        return Ok(());
    }

    let cargo_content = fs::read_to_string(&cargo_path)?;
    let mut cargo_doc = cargo_content
        .parse::<DocumentMut>()
        .context("Failed to parse component Cargo.toml")?;

    // Add metadata section if it doesn't exist
    if cargo_doc.get("package").is_some() {
        if let Some(package) = cargo_doc.get_mut("package") {
            if let Some(table) = package.as_table_mut() {
                // Add metadata table if it doesn't exist
                if table.get("metadata").is_none() {
                    table.insert("metadata", toml_edit::Item::Table(toml_edit::Table::new()));
                }
                
                if let Some(metadata) = table.get_mut("metadata") {
                    if let Some(metadata_table) = metadata.as_table_mut() {
                        // Add ferrisup metadata table if it doesn't exist
                        if metadata_table.get("ferrisup").is_none() {
                            metadata_table.insert("ferrisup", toml_edit::Item::Table(toml_edit::Table::new()));
                        }
                        
                        if let Some(ferrisup) = metadata_table.get_mut("ferrisup") {
                            if let Some(ferrisup_table) = ferrisup.as_table_mut() {
                                ferrisup_table.insert("component_type", value(component_type));
                            }
                        }
                    }
                }
            }
        }

        // Write updated Cargo.toml
        fs::write(cargo_path, cargo_doc.to_string())?;
    }
    
    Ok(())
}

// Function to make a shared component accessible to all workspace members
pub fn make_shared_component_accessible(project_dir: &Path, component_name: &str) -> Result<()> {
    // Update workspace Cargo.toml to include the shared component in the members list
    // and add it to workspace.dependencies
    let workspace_cargo_path = project_dir.join("Cargo.toml");
    if !workspace_cargo_path.exists() {
        return Ok(());
    }

    let workspace_cargo_content = fs::read_to_string(&workspace_cargo_path)?;
    let mut workspace_doc = workspace_cargo_content
        .parse::<DocumentMut>()
        .context("Failed to parse workspace Cargo.toml")?;

    // Add the shared component to the members list
    if let Some(workspace) = workspace_doc.get_mut("workspace") {
        if let Some(workspace_table) = workspace.as_table_mut() {
            if let Some(members) = workspace_table.get_mut("members") {
                if let Some(members_array) = members.as_array_mut() {
                    // Check if the component is already in the members list
                    let component_exists = members_array.iter().any(|item| {
                        if let Some(member_str) = item.as_str() {
                            member_str == component_name
                        } else {
                            false
                        }
                    });
                    
                    // Add the component to the members list if it doesn't exist
                    if !component_exists {
                        members_array.push(component_name);
                        println!("{} {}", "Added".green(), format!("'{}' to workspace members", component_name).cyan());
                    }
                }
            }
            
            // Add the shared component to workspace.dependencies
            if workspace_table.get("dependencies").is_none() {
                workspace_table.insert("dependencies", toml_edit::Item::Table(toml_edit::Table::new()));
            }
            
            if let Some(deps) = workspace_table.get_mut("dependencies") {
                if let Some(deps_table) = deps.as_table_mut() {
                    // Create a path dependency to the shared component
                    let mut dep_table = toml_edit::Table::new();
                    dep_table.insert("path", value(format!("./{}" , component_name)));
                    deps_table.insert(component_name, toml_edit::Item::Table(dep_table));
                    println!("{} {}", "Added".green(), 
                        format!("'{}' to workspace.dependencies", component_name).cyan());
                }
            }
        }
    }

    // Write updated workspace Cargo.toml
    fs::write(workspace_cargo_path, workspace_doc.to_string())?;
    
    // Now find all other component directories and add the shared component as a workspace dependency
    // to each component's Cargo.toml
    if let Some(workspace) = workspace_doc.get("workspace") {
        if let Some(workspace_table) = workspace.as_table() {
            if let Some(members) = workspace_table.get("members") {
                if let Some(members_array) = members.as_array() {
                    for member in members_array {
                        if let Some(member_str) = member.as_str() {
                            // Skip the shared component itself
                            if member_str == component_name {
                                continue;
                            }
                            
                            // Add the shared component as a workspace dependency to this component
                            let component_cargo_path = project_dir.join(member_str).join("Cargo.toml");
                            if component_cargo_path.exists() {
                                add_shared_workspace_dependency_to_component(&component_cargo_path, component_name)?;
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

// Helper function to add a shared component as a workspace dependency to a component's Cargo.toml
// Uses proper TOML table syntax for dependencies: shared = { workspace = true }
fn add_shared_workspace_dependency_to_component(cargo_path: &Path, shared_component: &str) -> Result<()> {
    let cargo_content = fs::read_to_string(cargo_path)?;
    let mut doc = cargo_content
        .parse::<DocumentMut>()
        .context("Failed to parse component Cargo.toml")?;
    
    // Add the shared component as a workspace dependency
    if let Some(dependencies) = doc.get_mut("dependencies") {
        if let Some(deps_table) = dependencies.as_table_mut() {
            // Only add if it doesn't already exist
            if deps_table.get(shared_component).is_none() {
                // Create a workspace dependency
                let mut dep_table = toml_edit::Table::new();
                dep_table.insert("workspace", value(true));
                deps_table.insert(shared_component, toml_edit::Item::Table(dep_table));
                
                println!("{} {}", "Added".green(), 
                    format!("'{}' as workspace dependency to {}", 
                        shared_component, 
                        cargo_path.parent().unwrap().file_name().unwrap().to_string_lossy()).cyan());
                
                // Add import to source files
                add_import_to_source_files(cargo_path.parent().unwrap(), shared_component)?;
            }
        }
    } else {
        // Create dependencies section if it doesn't exist
        let mut deps_table = toml_edit::Table::new();
        
        // Create a workspace dependency
        let mut dep_table = toml_edit::Table::new();
        dep_table.insert("workspace", value(true));
        deps_table.insert(shared_component, toml_edit::Item::Table(dep_table));
        
        doc.insert("dependencies", toml_edit::Item::Table(deps_table));
        println!("{} {}", "Added".green(), 
            format!("'{}' as workspace dependency to {}", 
                shared_component, 
                cargo_path.parent().unwrap().file_name().unwrap().to_string_lossy()).cyan());
        
        // Add import to source files
        add_import_to_source_files(cargo_path.parent().unwrap(), shared_component)?;
    }
    
    // Write updated Cargo.toml
    fs::write(cargo_path, doc.to_string())?;
    
    Ok(())
}

// Helper function to add import statements to source files
fn add_import_to_source_files(component_dir: &Path, shared_component: &str) -> Result<()> {
    // Find main.rs and lib.rs files
    let main_rs_path = component_dir.join("src/main.rs");
    let lib_rs_path = component_dir.join("src/lib.rs");
    
    // Add import to main.rs if it exists
    if main_rs_path.exists() {
        add_import_to_file(&main_rs_path, shared_component)?;
    }
    
    // Add import to lib.rs if it exists
    if lib_rs_path.exists() {
        add_import_to_file(&lib_rs_path, shared_component)?;
    }
    
    Ok(())
}

// Helper function to add import statement to a specific file
fn add_import_to_file(file_path: &Path, shared_component: &str) -> Result<()> {
    let content = fs::read_to_string(file_path)?;
    
    // Check if import already exists
    let import_statement = format!("use {}::*;", shared_component);
    if content.contains(&import_statement) {
        return Ok(());
    }
    
    // Add import at the top of the file
    let updated_content = format!("{}\n\n{}", import_statement, content);
    fs::write(file_path, updated_content)?;
    
    println!("{} {}", "Added".green(), 
        format!("'{}::*' import to {}", shared_component, file_path.file_name().unwrap().to_string_lossy()).cyan());
    
    Ok(())
}

// Function to update path references in files kept at the root
pub fn update_root_file_references(project_dir: &Path, component_name: &str, files_to_keep_at_root: &[String]) -> Result<()> {
    println!("Updating references in files kept at root...");
    
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
            let src_regex = format!(r"(src/[\w\-\.\/]+)");
            let re = Regex::new(&src_regex).unwrap();
            
            updated_content = re.replace_all(&updated_content, |caps: &regex::Captures| {
                format!("{}/{}", component_name, &caps[1])
            }).to_string();
            
            // If content was modified, write it back
            if content != updated_content {
                fs::write(&file_path, updated_content)?;
            }
        }
    }
    
    Ok(())
}

// Special handling for .gitignore files using wildcards
pub fn update_gitignore_references(gitignore_path: &Path, component_name: &str) -> Result<()> {
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
        fs::write(gitignore_path, all_lines.join("\n"))?;
    }
    
    Ok(())
}
