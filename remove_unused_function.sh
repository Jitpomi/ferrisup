#!/bin/bash

# Path to the file to modify
FILE="/Users/samsonssali/RustroverProjects/tools/ferrisup/ferrisup/src/commands/transform/utils.rs"

# Create a temporary file
TMP_FILE=$(mktemp)

# Find the start and end line numbers of the function to remove
START_LINE=$(grep -n "fn add_shared_dependency_to_component" "$FILE" | cut -d: -f1)
START_LINE=$((START_LINE - 1))  # Include the comment line
END_LINE=$((START_LINE + 47))   # Function is approximately 47 lines long

# Extract the content before the function
head -n $((START_LINE - 1)) "$FILE" > "$TMP_FILE"

# Extract the content after the function
tail -n +$((END_LINE + 1)) "$FILE" >> "$TMP_FILE"

# Replace the original file with the modified one
mv "$TMP_FILE" "$FILE"
