#!/bin/bash

# Add import for the import_fixer module
sed -i '' '8s/use crate::utils::{create_directory, update_workspace_members};/use crate::utils::{create_directory, update_workspace_members};\nuse crate::commands::import_fixer::fix_component_imports;/' src/commands/transform.rs

# Add call to fix_component_imports after updating the package name in Cargo.toml
sed -i '' '/fs::write(component_cargo_path, component_doc.to_string())?;/a\\
        \\
        // Fix imports in source files to use the new package name\\
        fix_component_imports(\\&component_dir, \\&component_name, project_name)?;\\
' src/commands/transform.rs

echo "Fixed import handling in transform.rs"
