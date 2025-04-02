# {{project_name}}

A data analysis CLI application using Polars (similar to pandas in Python)

## Features

- Fast data processing with Polars (Rust implementation similar to pandas)
- Command-line interface for data analysis
{{#if (eq data_source "CSV files")}}
- Support for CSV files
{{/if}}
{{#if (eq data_source "Parquet files")}}
- Support for Parquet files
{{/if}}
{{#if (eq data_source "JSON data")}}
- Support for JSON data
{{/if}}
{{#if (eq data_source "Multiple sources")}}
- Support for CSV, Parquet, and JSON files
{{/if}}
- Statistical analysis capabilities
{{#if (eq visualization "yes")}}
- Data visualization with Plotters
{{/if}}
- Sample data generation
{{#if (eq data_source "Multiple sources")}}
- Format conversion (CSV ↔ Parquet ↔ JSON)
{{/if}}

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/{{project_name}}`.

## Usage

### Analyzing Data

```bash
# Basic usage
{{#if (eq data_source "CSV files")}}
cargo run -- analyze -f data.csv
{{/if}}
{{#if (eq data_source "Parquet files")}}
cargo run -- analyze -f data.parquet -t parquet
{{/if}}
{{#if (eq data_source "JSON data")}}
cargo run -- analyze -f data.json -t json
{{/if}}
{{#if (eq data_source "Multiple sources")}}
# Analyze files in different formats
cargo run -- analyze -f data.csv
cargo run -- analyze -f data.parquet -t parquet
cargo run -- analyze -f data.json -t json
{{/if}}

# Filtering data
cargo run -- analyze -f data.csv -c department -v Engineering

# Grouping and aggregation
cargo run -- analyze -f data.csv -g department -a salary -u mean

# Statistical analysis
cargo run -- analyze -f data.csv -s
```

### Generating Sample Data

```bash
# Generate 100 rows of sample data
{{#if (eq data_source "CSV files")}}
cargo run -- generate -r 100 -o sample_data.csv
{{/if}}
{{#if (eq data_source "Parquet files")}}
cargo run -- generate -r 100 -o sample_data.parquet -t parquet
{{/if}}
{{#if (eq data_source "JSON data")}}
cargo run -- generate -r 100 -o sample_data.json -t json
{{/if}}
{{#if (eq data_source "Multiple sources")}}
cargo run -- generate -r 100 -o sample_data.csv
cargo run -- generate -r 100 -o sample_data.parquet -t parquet
cargo run -- generate -r 100 -o sample_data.json -t json
{{/if}}
```

{{#if (eq data_source "Multiple sources")}}
### Converting Between Formats

```bash
# Convert CSV to Parquet
cargo run -- convert -i data.csv -o data.parquet -t parquet

# Convert Parquet to JSON
cargo run -- convert -i data.parquet -f parquet -o data.json -t json

# Convert JSON to CSV
cargo run -- convert -i data.json -f json -o data.csv -t csv
```
{{/if}}

## Command-Line Arguments

### Analyze Command

- `-f, --file <FILE>`: Path to the data file
- `-t, --format <FORMAT>`: File format (csv, json, parquet) [default: {{#if (eq data_source "Parquet files")}}parquet{{else}}{{#if (eq data_source "JSON data")}}json{{else}}csv{{/if}}{{/if}}]
- `-c, --filter-column <FILTER_COLUMN>`: Optional column to filter on
- `-v, --filter-value <FILTER_VALUE>`: Optional value to filter for
- `-g, --group-by <GROUP_BY>`: Optional column to group by
- `-a, --aggregate <AGGREGATE>`: Optional column to aggregate
- `-u, --agg-func <AGG_FUNC>`: Aggregation function (sum, mean, min, max, count) [default: count]
- `-s, --stats`: Perform statistical analysis
- `--confidence <CONFIDENCE>`: Confidence level for statistical tests (0.90, 0.95, 0.99) [default: 0.95]
{{#if (eq data_source "JSON data")}}
- `--json-format <JSON_FORMAT>`: JSON format (records, lines) [default: records]
{{/if}}
{{#if (eq data_source "Multiple sources")}}
- `--json-format <JSON_FORMAT>`: JSON format (records, lines) [default: records]
{{/if}}

### Generate Command

- `-r, --rows <ROWS>`: Number of rows to generate [default: 100]
- `-o, --output <OUTPUT>`: Output file path
- `-t, --format <FORMAT>`: Output format (csv, json, parquet) [default: {{#if (eq data_source "Parquet files")}}parquet{{else}}{{#if (eq data_source "JSON data")}}json{{else}}csv{{/if}}{{/if}}]

{{#if (eq data_source "Multiple sources")}}
### Convert Command

- `-i, --input <INPUT>`: Input file path
- `-f, --input-format <INPUT_FORMAT>`: Input file format (csv, json, parquet) [default: csv]
- `-o, --output <OUTPUT>`: Output file path
- `-t, --output-format <OUTPUT_FORMAT>`: Output file format (csv, json, parquet) [default: {{#if (eq data_source "Parquet files")}}parquet{{else}}{{#if (eq data_source "JSON data")}}json{{else}}csv{{/if}}{{/if}}]
- `--json-format <JSON_FORMAT>`: JSON format for input/output (records, lines) [default: records]
{{/if}}

## Examples

### Basic Analysis

```bash
{{#if (eq data_source "CSV files")}}
# Analyze a CSV file
cargo run -- analyze -f employee_data.csv
{{/if}}
{{#if (eq data_source "Parquet files")}}
# Analyze a Parquet file
cargo run -- analyze -f employee_data.parquet -t parquet
{{/if}}
{{#if (eq data_source "JSON data")}}
# Analyze a JSON file
cargo run -- analyze -f employee_data.json -t json
{{/if}}
{{#if (eq data_source "Multiple sources")}}
# Analyze files in different formats
cargo run -- analyze -f employee_data.csv
cargo run -- analyze -f employee_data.parquet -t parquet
cargo run -- analyze -f employee_data.json -t json
{{/if}}
```

### Filtering and Grouping

```bash
# Filter data where department is "Engineering"
cargo run -- analyze -f employee_data.csv -c department -v Engineering

# Group by department and calculate mean salary
cargo run -- analyze -f employee_data.csv -g department -a salary -u mean
```

### Statistical Analysis

```bash
# Perform statistical analysis with 95% confidence level
cargo run -- analyze -f employee_data.csv -s

# Perform statistical analysis with 99% confidence level
cargo run -- analyze -f employee_data.csv -s --confidence 0.99
```

{{#if (eq visualization "yes")}}
### Data Visualization

The application automatically generates histograms for numeric columns when using the `-s` flag for statistical analysis. The histograms are saved as PNG files in the same directory as the input file.

```bash
# Generate histograms for all numeric columns
cargo run -- analyze -f employee_data.csv -s
```
{{/if}}

## License

MIT
