#!/bin/bash

# This script runs before the template is generated
# It updates the next_steps in template.json based on the selected data format

# Get the data source from the template variables
DATA_SOURCE="${data_source}"

# Get the template directory
TEMPLATE_DIR="$(dirname "$0")/.."
cd "$TEMPLATE_DIR"

# Silently update next_steps for $DATA_SOURCE

# Create a temporary file
TEMP_FILE=$(mktemp)

# Function to update next steps based on data format
update_next_steps() {
  local format=$1
  local next_steps_file=$2
  
  # Read the current template.json file
  cat template.json > "$TEMP_FILE"
  
  # Read the next steps from the appropriate file
  NEXT_STEPS=$(cat "$next_steps_file" | sed 's/"/\\"/g' | sed 's/^/    "/g' | sed 's/$/",/g' | sed '$ s/,$//')
  
  # Update the next_steps section with format-specific steps
  sed -i '' "s|\"next_steps\": \[.*\]|\"next_steps\": \[\n$NEXT_STEPS\n  \]|" "$TEMP_FILE"
  
  # Replace the template.json file with the updated one
  mv "$TEMP_FILE" "template.json"
  # Silently updated next_steps for $format format
}

# Select the appropriate next steps file based on the data source
if [ "$DATA_SOURCE" = "CSV files" ]; then
  update_next_steps "CSV" "next_steps_csv.txt"
elif [ "$DATA_SOURCE" = "JSON data" ]; then
  update_next_steps "JSON" "next_steps_json.txt"
elif [ "$DATA_SOURCE" = "Parquet files" ]; then
  update_next_steps "Parquet" "next_steps_parquet.txt"
else
  # Default to CSV if no valid option is provided
  update_next_steps "CSV" "next_steps_csv.txt"
fi
