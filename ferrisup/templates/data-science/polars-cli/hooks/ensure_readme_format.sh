#!/bin/bash

# This script ensures the README.md file has the correct data format
# It runs automatically during project creation

# Get the data format from the template variables
DATA_SOURCE="${data_source}"
README_FILE="README.md"

# Determine the file extension based on the data source
if [ "$DATA_SOURCE" = "CSV files" ]; then
    FILE_EXT="csv"
    FORMAT_NAME="CSV"
elif [ "$DATA_SOURCE" = "JSON data" ]; then
    FILE_EXT="json"
    FORMAT_NAME="JSON"
elif [ "$DATA_SOURCE" = "Parquet files" ]; then
    FILE_EXT="parquet"
    FORMAT_NAME="Parquet"
else
    echo "Unknown data source: $DATA_SOURCE"
    exit 1
fi

# Check if the README.md file exists
if [ -f "$README_FILE" ]; then
    echo "Ensuring README.md file has correct $FORMAT_NAME format..."
    
    # Create a temporary file
    TEMP_FILE=$(mktemp)
    
    # Read the current README.md file
    cat "$README_FILE" > "$TEMP_FILE"
    
    # Update the Features section
    sed -i '' 's/- Support for CSV files//g' "$TEMP_FILE"
    sed -i '' 's/- Support for JSON data//g' "$TEMP_FILE"
    sed -i '' 's/- Support for Parquet files//g' "$TEMP_FILE"
    sed -i '' "s/- Support for.*$/- Support for $FORMAT_NAME files/g" "$TEMP_FILE"
    
    # Update file examples
    # Replace all example_data.* with example_data.$FILE_EXT
    sed -i '' "s/example_data\\.[a-z]*/example_data.$FILE_EXT/g" "$TEMP_FILE"
    # Replace all my_data.* with my_data.$FILE_EXT
    sed -i '' "s/my_data\\.[a-z]*/my_data.$FILE_EXT/g" "$TEMP_FILE"
    # Update default format
    sed -i '' "s/\\[default: [a-z]*\\]/\\[default: $FILE_EXT\\]/g" "$TEMP_FILE"
    # Update file description
    sed -i '' "s/A [A-Za-z]* file with employee records/A $FORMAT_NAME file with employee records/g" "$TEMP_FILE"
    
    # Replace the README.md file with the updated one
    mv "$TEMP_FILE" "$README_FILE"
    
    echo "README.md file updated successfully!"
else
    echo "README.md file not found!"
fi
