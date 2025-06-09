#!/bin/bash

# This script updates the next_steps in template.json based on the selected data format
# Usage: ./update_next_steps.sh [data_format]

# Get the data source from the first argument
DATA_SOURCE="$1"

# Create a temporary file
TEMP_FILE=$(mktemp)

# Function to update next steps based on data format
update_next_steps() {
  local format=$1
  local file_ext=$2
  
  # Read the current template.json file
  cat template.json > "$TEMP_FILE"
  
  # Update the next_steps section with format-specific steps
  sed -i '' "s|\"next_steps\": \[.*\]|\"next_steps\": \[\n    \"📊 Try the example analysis: cd {{project_name}} \&\& cargo run -- analyze -f data/example_data.${file_ext}\",\n    \"📈 Run statistical analysis: cargo run -- analyze -f data/example_data.${file_ext} -s\",\n    \"🔍 Group data by department: cargo run -- analyze -f data/example_data.${file_ext} -g department -a salary -u mean\",\n    \"🧮 Generate sample data: cargo run -- generate -r 100 -o data/my_data.${file_ext}\",\n    \"📚 See all available commands: cargo run -- help\"\n  \]|" "$TEMP_FILE"
  
  # Replace the template.json file with the updated one
  mv "$TEMP_FILE" "template.json"
  echo "Updated next_steps for $format format"
}

# Select the appropriate template file based on the data source
if [ "$DATA_SOURCE" = "CSV files" ]; then
  update_next_steps "CSV" "csv"
elif [ "$DATA_SOURCE" = "JSON data" ]; then
  update_next_steps "JSON" "json"
elif [ "$DATA_SOURCE" = "Parquet files" ]; then
  update_next_steps "Parquet" "parquet"
else
  # Default to CSV if no valid option is provided
  update_next_steps "CSV" "csv"
fi
