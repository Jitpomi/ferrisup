#!/bin/bash

# This script removes unwanted data files based on user selection
# It should be run manually after project creation

# Get the data source from command line argument
DATA_SOURCE="$1"

if [ -z "$DATA_SOURCE" ]; then
    echo "Usage: $0 <data-format>"
    echo "Where <data-format> is one of: csv, json, parquet"
    exit 1
fi

echo "Cleaning up unwanted data files..."
echo "Selected data format: $DATA_SOURCE"
echo "Current directory: $(pwd)"

# Create data directory if it doesn't exist
mkdir -p data

# Keep only the selected data format and remove others
if [ "$DATA_SOURCE" = "csv" ]; then
    echo "Keeping CSV format, removing others..."
    rm -f data/example_data.json data/example_data.parquet
elif [ "$DATA_SOURCE" = "json" ]; then
    echo "Keeping JSON format, removing others..."
    rm -f data/example_data.csv data/example_data.parquet
elif [ "$DATA_SOURCE" = "parquet" ]; then
    echo "Keeping Parquet format, removing others..."
    rm -f data/example_data.csv data/example_data.json
else
    echo "Unknown data format: $DATA_SOURCE"
    echo "Please specify one of: csv, json, parquet"
    exit 1
fi

# List the data directory to confirm
echo "Remaining data files:"
ls -la data/

echo "Cleanup complete"
