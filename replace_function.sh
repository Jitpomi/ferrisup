#!/bin/bash

# Get the line numbers for the start and end of the function
START_LINE=$(grep -n "^pub fn make_shared_component_accessible" ferrisup/src/commands/transform/utils.rs | cut -d: -f1)
END_LINE=$(tail -n +$START_LINE ferrisup/src/commands/transform/utils.rs | grep -n "^}" | head -1 | cut -d: -f1)
END_LINE=$((START_LINE + END_LINE - 1))

# Create a temporary file with the content before the function
head -n $((START_LINE - 1)) ferrisup/src/commands/transform/utils.rs > temp_file.rs

# Append the new function implementation
cat fixed_make_shared.rs >> temp_file.rs

# Append the content after the function
tail -n +$((END_LINE + 1)) ferrisup/src/commands/transform/utils.rs >> temp_file.rs

# Replace the original file
mv temp_file.rs ferrisup/src/commands/transform/utils.rs
