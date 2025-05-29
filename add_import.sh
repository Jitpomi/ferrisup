#!/bin/bash
set -e

# Add the import for direct_component
echo 'use crate::commands::direct_component::add_component_direct;' > import_line.txt
sed -i '' '9i\
use crate::commands::direct_component::add_component_direct;
' src/commands/transform.rs

echo "Added import for add_component_direct to transform.rs"
