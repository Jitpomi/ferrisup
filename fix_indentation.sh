#!/bin/bash
# Script to fix the indentation in the utils.rs file

# Path to the file to modify
FILE="/Users/samsonssali/RustroverProjects/tools/ferrisup/ferrisup/src/commands/transform/utils.rs"

# Create a temporary file
TMP_FILE=$(mktemp)

# Use sed to fix the indentation
sed '188,203s/^        //' "$FILE" > "$TMP_FILE"

# Replace the original file with the fixed one
mv "$TMP_FILE" "$FILE"
