#!/bin/bash
set -e

echo "Cleaning up unused function and reinstalling FerrisUp..."

# Find the line number where the unused function starts
START_LINE=$(grep -n "fn fix_component_imports_after_rename" src/commands/transform.rs | cut -d: -f1)

if [ -z "$START_LINE" ]; then
  echo "Function already removed, skipping cleanup."
else
  # Find where the function ends (looking for closing brace at the beginning of a line)
  END_LINE=$(tail -n +$START_LINE src/commands/transform.rs | grep -n "^}" | head -1 | cut -d: -f1)
  END_LINE=$((START_LINE + END_LINE - 1))
  
  # Create a temporary file with the function removed
  sed -i '' "${START_LINE},${END_LINE}d" src/commands/transform.rs
  
  echo "Removed unused function from lines $START_LINE to $END_LINE"
fi

# Reinstall FerrisUp
echo "Reinstalling FerrisUp..."
cargo install --path .

echo "Cleanup and reinstallation complete!"
