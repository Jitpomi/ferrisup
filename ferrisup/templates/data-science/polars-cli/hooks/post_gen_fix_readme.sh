#!/bin/bash

# This script runs after the template is generated
# It fixes the README.md file to only show examples for the selected data format

# Get the data format from the template variables
DATA_SOURCE="${data_source}"
DATA_FORMAT="${data_format}"

# Path to the README.md file
README_FILE="README.md"

# Function to log messages if verbose mode is enabled
log_message() {
    if [ "${FERRISUP_VERBOSE}" = "true" ]; then
        echo "[post_gen_fix_readme] $1"
    fi
}

# Check if the README.md file exists
if [ -f "$README_FILE" ]; then
    log_message "Fixing README.md file for $DATA_SOURCE..."
    
    # Create a temporary file
    TEMP_FILE=$(mktemp)
    
    # Read the current README.md file
    cat "$README_FILE" > "$TEMP_FILE"
    
    # Update the Features section
    if [ "$DATA_SOURCE" = "CSV files" ]; then
        # Keep only CSV in features, remove JSON and Parquet
        sed -i '' '/- Support for JSON data/d' "$TEMP_FILE"
        sed -i '' '/- Support for Parquet files/d' "$TEMP_FILE"
    elif [ "$DATA_SOURCE" = "JSON data" ]; then
        # Keep only JSON in features, remove CSV and Parquet
        sed -i '' '/- Support for CSV files/d' "$TEMP_FILE"
        sed -i '' '/- Support for Parquet files/d' "$TEMP_FILE"
    elif [ "$DATA_SOURCE" = "Parquet files" ]; then
        # Keep only Parquet in features, remove CSV and JSON
        sed -i '' '/- Support for CSV files/d' "$TEMP_FILE"
        sed -i '' '/- Support for JSON data/d' "$TEMP_FILE"
    fi
    
    # Remove examples for other data formats
    if [ "$DATA_SOURCE" = "CSV files" ]; then
        # Keep only CSV examples, remove JSON and Parquet examples
        sed -i '' '/example_data\.json/d' "$TEMP_FILE"
        sed -i '' '/example_data\.parquet/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- analyze -f data\/example_data\.json/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- analyze -f data\/example_data\.parquet/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- generate -r 100 -o data\/my_data\.json/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- generate -r 100 -o data\/my_data\.parquet/d' "$TEMP_FILE"
        
        # Update the default format in the command-line arguments section
        sed -i '' 's/\[default: json\]/\[default: csv\]/g' "$TEMP_FILE"
        sed -i '' 's/\[default: parquet\]/\[default: csv\]/g' "$TEMP_FILE"
        
    elif [ "$DATA_SOURCE" = "JSON data" ]; then
        # Keep only JSON examples, remove CSV and Parquet examples
        sed -i '' '/example_data\.csv/d' "$TEMP_FILE"
        sed -i '' '/example_data\.parquet/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- analyze -f data\/example_data\.csv/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- analyze -f data\/example_data\.parquet/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- generate -r 100 -o data\/my_data\.csv/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- generate -r 100 -o data\/my_data\.parquet/d' "$TEMP_FILE"
        
        # Update the default format in the command-line arguments section
        sed -i '' 's/\[default: csv\]/\[default: json\]/g' "$TEMP_FILE"
        sed -i '' 's/\[default: parquet\]/\[default: json\]/g' "$TEMP_FILE"
        
    elif [ "$DATA_SOURCE" = "Parquet files" ]; then
        # Keep only Parquet examples, remove CSV and JSON examples
        sed -i '' '/example_data\.csv/d' "$TEMP_FILE"
        sed -i '' '/example_data\.json/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- analyze -f data\/example_data\.csv/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- analyze -f data\/example_data\.json/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- generate -r 100 -o data\/my_data\.csv/d' "$TEMP_FILE"
        sed -i '' '/cargo run -- generate -r 100 -o data\/my_data\.json/d' "$TEMP_FILE"
        
        # Update the default format in the command-line arguments section
        sed -i '' 's/\[default: csv\]/\[default: parquet\]/g' "$TEMP_FILE"
        sed -i '' 's/\[default: json\]/\[default: parquet\]/g' "$TEMP_FILE"
    fi
    
    # Update the format list to include parquet
    sed -i '' 's/Data format (csv, json)/Data format (csv, json, parquet)/g' "$TEMP_FILE"
    
    # Replace the README.md file with the updated one
    mv "$TEMP_FILE" "$README_FILE"
    
    log_message "README.md file fixed successfully!"
else
    log_message "README.md file not found!"
fi
