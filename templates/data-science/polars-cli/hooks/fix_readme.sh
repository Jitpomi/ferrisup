#!/bin/bash

# This script can be run after project creation to fix the README.md file
# Usage: ./fix_readme.sh <data_source>

DATA_SOURCE="$1"
README_FILE="README.md"

if [ -z "$DATA_SOURCE" ]; then
    echo "Please provide a data source (CSV files, JSON data, or Parquet files)"
    exit 1
fi

if [ ! -f "$README_FILE" ]; then
    echo "README.md file not found!"
    exit 1
fi

echo "Fixing README.md file for $DATA_SOURCE..."

# Create a temporary file
TEMP_FILE=$(mktemp)

# Read the current README.md file
cat "$README_FILE" > "$TEMP_FILE"

# Update the Features section
if [ "$DATA_SOURCE" = "CSV files" ]; then
    # Keep only CSV in features, remove JSON and Parquet
    sed -i '' 's/- Support for JSON data//g' "$TEMP_FILE"
    sed -i '' 's/- Support for Parquet files//g' "$TEMP_FILE"
    sed -i '' 's/- Support for.*$/- Support for CSV files/g' "$TEMP_FILE"
elif [ "$DATA_SOURCE" = "JSON data" ]; then
    # Keep only JSON in features, remove CSV and Parquet
    sed -i '' 's/- Support for CSV files//g' "$TEMP_FILE"
    sed -i '' 's/- Support for Parquet files//g' "$TEMP_FILE"
    sed -i '' 's/- Support for.*$/- Support for JSON data/g' "$TEMP_FILE"
elif [ "$DATA_SOURCE" = "Parquet files" ]; then
    # Keep only Parquet in features, remove CSV and JSON
    sed -i '' 's/- Support for CSV files//g' "$TEMP_FILE"
    sed -i '' 's/- Support for JSON data//g' "$TEMP_FILE"
    sed -i '' 's/- Support for.*$/- Support for Parquet files/g' "$TEMP_FILE"
fi

# Update file examples
if [ "$DATA_SOURCE" = "CSV files" ]; then
    # Replace all example_data.* with example_data.csv
    sed -i '' 's/example_data\.[a-z]*/example_data.csv/g' "$TEMP_FILE"
    # Replace all my_data.* with my_data.csv
    sed -i '' 's/my_data\.[a-z]*/my_data.csv/g' "$TEMP_FILE"
    # Update default format
    sed -i '' 's/\[default: [a-z]*\]/\[default: csv\]/g' "$TEMP_FILE"
elif [ "$DATA_SOURCE" = "JSON data" ]; then
    # Replace all example_data.* with example_data.json
    sed -i '' 's/example_data\.[a-z]*/example_data.json/g' "$TEMP_FILE"
    # Replace all my_data.* with my_data.json
    sed -i '' 's/my_data\.[a-z]*/my_data.json/g' "$TEMP_FILE"
    # Update default format
    sed -i '' 's/\[default: [a-z]*\]/\[default: json\]/g' "$TEMP_FILE"
elif [ "$DATA_SOURCE" = "Parquet files" ]; then
    # Replace all example_data.* with example_data.parquet
    sed -i '' 's/example_data\.[a-z]*/example_data.parquet/g' "$TEMP_FILE"
    # Replace all my_data.* with my_data.parquet
    sed -i '' 's/my_data\.[a-z]*/my_data.parquet/g' "$TEMP_FILE"
    # Update default format
    sed -i '' 's/\[default: [a-z]*\]/\[default: parquet\]/g' "$TEMP_FILE"
fi

# Replace the README.md file with the updated one
mv "$TEMP_FILE" "$README_FILE"

echo "README.md file fixed successfully!"
