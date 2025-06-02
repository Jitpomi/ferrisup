#!/bin/bash

# This script removes unwanted data files based on user selection
# It is called by the post-generation hook in template.json

# Exit on error
set -e

# Debug mode for troubleshooting
set -x

# Get the data source from command line argument
DATA_SOURCE="$1"

# Print debug info
echo "Cleaning up unwanted data files..."
echo "Selected data format: $DATA_SOURCE"
echo "Current directory: $(pwd)"
echo "Files in data directory before cleanup:"
ls -la data/

# Create data directory if it doesn't exist
mkdir -p data

# Keep only the selected data format and remove others
if [ "$DATA_SOURCE" = "CSV files" ]; then
    echo "Keeping CSV format, removing others..."
    rm -f data/example_data.json data/example_data.parquet
elif [ "$DATA_SOURCE" = "JSON data" ]; then
    echo "Keeping JSON format, removing others..."
    rm -f data/example_data.csv data/example_data.parquet
elif [ "$DATA_SOURCE" = "Parquet files" ]; then
    echo "Keeping Parquet format, removing others..."
    rm -f data/example_data.csv data/example_data.json
else
    echo "Unknown data source: $DATA_SOURCE"
    echo "Defaulting to CSV, removing others..."
    rm -f data/example_data.json data/example_data.parquet
fi

# List the data directory to confirm
echo "Files in data directory after cleanup:"
ls -la data/

echo "Cleanup complete"

# Exit with success
exit 0
