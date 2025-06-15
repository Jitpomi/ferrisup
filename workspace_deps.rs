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
