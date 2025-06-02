#!/bin/bash

# This script directly creates a new README.md file based on the selected data source
# Usage: ./fix_readme_direct.sh <data_source> <project_name>

# Enable debugging
set -x

# Log to a file for debugging
exec > /tmp/fix_readme_debug.log 2>&1

DATA_SOURCE="$1"
PROJECT_NAME="$2"
README_FILE="README.md"

if [ -z "$DATA_SOURCE" ]; then
    echo "Please provide a data source (CSV files, JSON data, or Parquet files)"
    exit 1
fi

if [ -z "$PROJECT_NAME" ]; then
    PROJECT_NAME="data-science-project"
fi

echo "Creating README.md file for $DATA_SOURCE..."

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

# Create a new README.md file with the correct format
cat > "$README_FILE" << EOF
# $PROJECT_NAME

A data analysis CLI application using Polars (similar to pandas in Python)

## Features

- Fast data processing with Polars (Rust implementation similar to pandas)
- Command-line interface for data analysis
- Support for $FORMAT_NAME files
- Statistical analysis capabilities
- Data visualization with Plotters
- Sample data generation

## Installation

\`\`\`bash
cargo build --release
\`\`\`

The binary will be available at \`target/release/$PROJECT_NAME\`.

## Usage

### Generating Sample Data

The project includes a script to generate sample data files for testing:

\`\`\`bash
cd data
./download_sample_data.sh
\`\`\`

This will create:
- example_data.$FILE_EXT - A $FORMAT_NAME file with employee records

### Analyzing Data

\`\`\`bash
# Basic usage
cargo run -- analyze -f data/example_data.$FILE_EXT

# Filtering data
cargo run -- analyze -f data/example_data.$FILE_EXT -c department -v Engineering

# Grouping and aggregation
cargo run -- analyze -f data/example_data.$FILE_EXT -g department -a salary -u mean

# Statistical analysis
cargo run -- analyze -f data/example_data.$FILE_EXT -s
\`\`\`

### Generating Sample Data

\`\`\`bash
# Generate 100 rows of sample data
cargo run -- generate -r 100 -o data/my_data.$FILE_EXT
\`\`\`

## Command-Line Arguments

### Analyze Command

- \`-f, --file <FILE>\`: Path to the data file
- \`-t, --format <FORMAT>\`: File format (csv, json, parquet) [default: $FILE_EXT]
- \`-c, --filter-column <FILTER_COLUMN>\`: Optional column to filter on
- \`-v, --filter-value <FILTER_VALUE>\`: Optional value to filter for
- \`-g, --group-by <GROUP_BY>\`: Optional column to group by
- \`-a, --aggregate <AGGREGATE>\`: Optional column to aggregate
- \`-u, --agg-func <AGG_FUNC>\`: Aggregation function (sum, mean, min, max, count) [default: count]
- \`-s, --stats\`: Perform statistical analysis
- \`--confidence <CONFIDENCE>\`: Confidence level for statistical tests (0.90, 0.95, 0.99) [default: 0.95]
- \`--json-format <JSON_FORMAT>\`: JSON format (records, lines) [default: records]

### Generate Command

- \`-r, --rows <ROWS>\`: Number of rows to generate [default: 100]
- \`-o, --output <o>\`: Output file path
- \`-t, --format <FORMAT>\`: Output format (csv, json, parquet) [default: $FILE_EXT]

## Examples

### Basic Analysis

\`\`\`bash
# Analyze the example data file
cargo run -- analyze -f data/example_data.$FILE_EXT
\`\`\`

### Filtering and Grouping

\`\`\`bash
# Filter data where department is "Engineering"
cargo run -- analyze -f data/example_data.$FILE_EXT -c department -v Engineering

# Group by department and calculate mean salary
cargo run -- analyze -f data/example_data.$FILE_EXT -g department -a salary -u mean
\`\`\`

### Statistical Analysis

\`\`\`bash
# Perform statistical analysis with 95% confidence level
cargo run -- analyze -f data/example_data.$FILE_EXT -s

# Perform statistical analysis with 99% confidence level
cargo run -- analyze -f data/example_data.$FILE_EXT -s --confidence 0.99
\`\`\`

### Data Visualization

The application automatically generates histograms for numeric columns when using the \`-s\` flag for statistical analysis. The histograms are saved as PNG files in the same directory as the input file.

\`\`\`bash
# Generate histograms for all numeric columns
cargo run -- analyze -f data/example_data.$FILE_EXT -s
\`\`\`

## License

MIT
EOF

echo "README.md file created successfully!"
