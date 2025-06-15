#!/bin/bash
# Script to insert the workspace dependencies code

# Path to the file to modify
FILE="/Users/samsonssali/RustroverProjects/tools/ferrisup/ferrisup/src/commands/transform/utils.rs"

# Create a temporary file
TMP_FILE=$(mktemp)

# Line number where to insert the code
LINE_NUM=187

# Read the file up to the line where we want to insert
head -n $LINE_NUM "$FILE" > "$TMP_FILE"

# Append the code to insert
cat >> "$TMP_FILE" << 'INSERTCODE'
            
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
INSERTCODE

# Append the rest of the original file
tail -n +$((LINE_NUM + 1)) "$FILE" >> "$TMP_FILE"

# Replace the original file with the modified one
mv "$TMP_FILE" "$FILE"
