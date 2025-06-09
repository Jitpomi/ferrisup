#!/bin/bash

# This script sets up the correct data file based on user selection
# It is called by the post-generation hook in template.json

# Exit on error
set -e

# Debug mode
set -x

# Get the template directory and data source
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_SOURCE="$1"

# Print debug info
echo "Script directory: $SCRIPT_DIR"
echo "Data source: $DATA_SOURCE"
echo "Current directory: $(pwd)"

# Create data directory
mkdir -p data
echo "Created data directory"

# Copy the appropriate data file based on the data source
if [ "$DATA_SOURCE" = "CSV files" ]; then
    cp "$SCRIPT_DIR/example_data_csv.csv" data/example_data.csv
    echo "Created CSV data file"
elif [ "$DATA_SOURCE" = "JSON data" ]; then
    cp "$SCRIPT_DIR/example_data_json.json" data/example_data.json
    echo "Created JSON data file"
elif [ "$DATA_SOURCE" = "Parquet files" ]; then
    cp "$SCRIPT_DIR/example_data_parquet.parquet" data/example_data.parquet
    echo "Created Parquet data file"
else
    echo "Unknown data source: $DATA_SOURCE"
    echo "Defaulting to CSV"
    cp "$SCRIPT_DIR/example_data_csv.csv" data/example_data.csv
fi

# List the data directory to confirm
ls -la data/
echo "Setup complete"

# Clean up the script and unused data files
rm -f "$SCRIPT_DIR/example_data_csv.csv" "$SCRIPT_DIR/example_data_json.json" "$SCRIPT_DIR/example_data_parquet.parquet"
rm -f "$SCRIPT_DIR/setup_data.sh"

# Exit with success
exit 0
