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
                    dep_table.insert("path", value(format!("./{}", component_name)));
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
