#!/bin/bash
set -e

# This script fixes the issue with component imports in the transform command
# It adds code to update imports after changing the package name in Cargo.toml

# Step 1: Create the import_fixer.rs file if it doesn't exist
cat > src/commands/import_fixer.rs << 'EOF'
use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use std::fs;
use regex::Regex;
use walkdir::WalkDir;

/// Fixes imports in a component after the package name has been updated
/// 
/// This function recursively searches through all Rust files in the component
/// and updates import statements to use the new package name.
/// 
/// For example, if a component was created with name "client" but the package
/// was renamed to "app_client" in Cargo.toml, this function will update all
/// imports from "use client::*" to "use app_client::*".
pub fn fix_component_imports(component_dir: &Path, component_name: &str, project_name: &str) -> Result<()> {
    println!("{}", format!("Fixing imports in component: {}", component_name).blue());
    
    // Get all Rust files in the component directory
    let src_dir = component_dir.join("src");
    
    if !src_dir.exists() {
        return Ok(());
    }
    
    // Process all Rust files in the src directory recursively
    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file()) {
            
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "rs" {
                // Read file content
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                
                // Replace imports like "use client::*;" with "use app_client::*;"
                let re = match Regex::new(&format!(r"use\s+{}(::|\s+)", regex::escape(component_name))) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                    
                let new_package_name = format!("{}_{}", project_name, component_name);
                let updated_content = re.replace_all(&content, format!("use {}{}", new_package_name, "$1"));
                
                // Write updated content back to file if changes were made
                if content != updated_content {
                    if let Err(_) = fs::write(path, updated_content.as_bytes()) {
                        continue;
                    }
                    println!("  Fixed imports in: {}", path.display());
                }
            }
        }
    }
    
    Ok(())
}
EOF

# Step 2: Update mod.rs to include the import_fixer module
if ! grep -q "pub mod import_fixer" src/commands/mod.rs; then
    sed -i '' '/pub mod unused_features;/a\\
pub mod import_fixer;' src/commands/mod.rs
fi

# Step 3: Add the import to transform.rs
if ! grep -q "use crate::commands::import_fixer::fix_component_imports" src/commands/transform.rs; then
    sed -i '' '/use crate::utils::{create_directory, update_workspace_members};/a\\
use crate::commands::import_fixer::fix_component_imports;' src/commands/transform.rs
fi

# Step 4: Add the dependencies to Cargo.toml if they don't exist
if ! grep -q "walkdir" Cargo.toml; then
    cargo add walkdir
fi

if ! grep -q "regex" Cargo.toml; then
    cargo add regex
fi

# Step 5: Add the call to fix_component_imports after updating the package name in Cargo.toml
# This is the trickiest part, so we'll use a more robust approach
cat > fix_transform.patch << 'EOF'
--- transform.rs.orig
+++ transform.rs
@@ -643,6 +643,9 @@
         // Write updated Cargo.toml
         fs::write(component_cargo_path, component_doc.to_string())?;
     }
+    
+    // Fix imports in source files to use the new package name
+    fix_component_imports(&component_dir, &component_name, &project_name.to_string())?;
 
     // Update workspace Cargo.toml to include the new component
     // Note: The update_workspace_members function will automatically detect and add the component
EOF

# Apply the patch - we'll use a more robust approach that works even if the file has been modified
line_number=$(grep -n "// Write updated Cargo.toml" src/commands/transform.rs | grep -A 2 "fs::write(component_cargo_path" | head -1 | cut -d: -f1)
if [ -n "$line_number" ]; then
    # Find the line with the closing brace after fs::write
    end_line=$((line_number + 3))
    
    # Insert our code after that line
    sed -i '' "${end_line}a\\
\\
\\ \\ \\ \\ \\/\\/\\ Fix\\ imports\\ in\\ source\\ files\\ to\\ use\\ the\\ new\\ package\\ name\\
\\ \\ \\ \\ fix_component_imports\\(\\&component_dir,\\ \\&component_name,\\ \\&project_name.to_string\\(\\)\\)\\?;" src/commands/transform.rs
    
    echo "Successfully added import fixing code to transform.rs"
else
    echo "Could not find the right location to insert the code in transform.rs"
    echo "Please manually add the following code after the 'fs::write(component_cargo_path, component_doc.to_string())?;' line:"
    echo ""
    echo "    // Fix imports in source files to use the new package name"
    echo "    fix_component_imports(&component_dir, &component_name, &project_name.to_string())?"
    echo ""
fi

echo "Done! Now install FerrisUp with 'cargo install --path .'"
