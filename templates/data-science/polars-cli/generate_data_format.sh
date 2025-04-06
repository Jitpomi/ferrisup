#!/bin/bash

# This script generates all data files from scratch based on user's selection
# It will be called during project generation by the template manager

# Add debugging
echo "Starting data file generation script"
echo "Current directory: $(pwd)"
echo "Script arguments: $@"

# Default to CSV if no format specified
FORMAT="${1:-csv}"
echo "Using format: $FORMAT"

# Ensure data directory exists
mkdir -p data
echo "Created data directory"

# Generate the CSV file (our source of truth)
cat > "data/example_data.csv" << EOF
id,name,age,salary,department,date
1,John Doe,35,85000,Engineering,2023-01-15
2,Jane Smith,42,92000,Engineering,2023-02-10
3,Bob Johnson,28,72000,Marketing,2023-01-05
4,Alice Brown,31,78000,Marketing,2023-03-20
5,Charlie Davis,45,110000,Sales,2023-02-28
6,Diana Evans,38,95000,Sales,2023-01-12
7,Edward Foster,29,68000,HR,2023-03-05
8,Fiona Garcia,36,82000,HR,2023-02-15
9,George Harris,52,125000,Finance,2023-01-30
10,Helen Irwin,41,105000,Finance,2023-03-10
11,Ian Jackson,33,79000,Engineering,2023-02-05
12,Julia Kim,27,65000,Marketing,2023-01-25
13,Kevin Lee,44,98000,Sales,2023-03-15
14,Laura Miller,39,88000,HR,2023-02-20
15,Mike Nelson,30,75000,Finance,2023-01-08
EOF

echo "CSV file created successfully at data/example_data.csv"
ls -la data/

# For JSON format, we can use a simple conversion without needing to compile Rust
if [ "$FORMAT" = "json" ]; then
    # Generate JSON directly
    cat > "data/example_data.json" << EOF
[
  {
    "id": 1,
    "name": "John Doe",
    "age": 35,
    "salary": 85000,
    "department": "Engineering",
    "date": "2023-01-15"
  },
  {
    "id": 2,
    "name": "Jane Smith",
    "age": 42,
    "salary": 92000,
    "department": "Engineering",
    "date": "2023-02-10"
  },
  {
    "id": 3,
    "name": "Bob Johnson",
    "age": 28,
    "salary": 72000,
    "department": "Marketing",
    "date": "2023-01-05"
  },
  {
    "id": 4,
    "name": "Alice Brown",
    "age": 31,
    "salary": 78000,
    "department": "Marketing",
    "date": "2023-03-20"
  },
  {
    "id": 5,
    "name": "Charlie Davis",
    "age": 45,
    "salary": 110000,
    "department": "Sales",
    "date": "2023-02-28"
  },
  {
    "id": 6,
    "name": "Diana Evans",
    "age": 38,
    "salary": 95000,
    "department": "Sales",
    "date": "2023-01-12"
  },
  {
    "id": 7,
    "name": "Edward Foster",
    "age": 29,
    "salary": 68000,
    "department": "HR",
    "date": "2023-03-05"
  },
  {
    "id": 8,
    "name": "Fiona Garcia",
    "age": 36,
    "salary": 82000,
    "department": "HR",
    "date": "2023-02-15"
  },
  {
    "id": 9,
    "name": "George Harris",
    "age": 52,
    "salary": 125000,
    "department": "Finance",
    "date": "2023-01-30"
  },
  {
    "id": 10,
    "name": "Helen Irwin",
    "age": 41,
    "salary": 105000,
    "department": "Finance",
    "date": "2023-03-10"
  },
  {
    "id": 11,
    "name": "Ian Jackson",
    "age": 33,
    "salary": 79000,
    "department": "Engineering",
    "date": "2023-02-05"
  },
  {
    "id": 12,
    "name": "Julia Kim",
    "age": 27,
    "salary": 65000,
    "department": "Marketing",
    "date": "2023-01-25"
  },
  {
    "id": 13,
    "name": "Kevin Lee",
    "age": 44,
    "salary": 98000,
    "department": "Sales",
    "date": "2023-03-15"
  },
  {
    "id": 14,
    "name": "Laura Miller",
    "age": 39,
    "salary": 88000,
    "department": "HR",
    "date": "2023-02-20"
  },
  {
    "id": 15,
    "name": "Mike Nelson",
    "age": 30,
    "salary": 75000,
    "department": "Finance",
    "date": "2023-01-08"
  }
]
EOF
    echo "JSON file created successfully at data/example_data.json"
    ls -la data/
fi

# For Parquet format, we need to use a Rust program
if [ "$FORMAT" = "parquet" ]; then
    # Check if we can use cargo to compile and run the conversion
    if command -v cargo &> /dev/null; then
        echo "Found cargo, creating temporary project for Parquet conversion"
        # Create a temporary directory for the conversion project
        TEMP_DIR=$(mktemp -d)
        echo "Temporary directory: $TEMP_DIR"
        
        # Create Cargo.toml
        cat > "$TEMP_DIR/Cargo.toml" << EOF
[package]
name = "csv_to_parquet"
version = "0.1.0"
edition = "2021"

[dependencies]
polars = { version = "0.35.0", features = ["parquet", "csv"] }
EOF

        # Create src directory
        mkdir -p "$TEMP_DIR/src"
        
        # Create main.rs
        cat > "$TEMP_DIR/src/main.rs" << EOF
use polars::prelude::*;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the CSV file
    let df = CsvReader::from_path("data/example_data.csv")?
        .has_header(true)
        .finish()?;
    
    println!("CSV data loaded successfully");
    
    // Write to Parquet file
    let mut file = File::create("data/example_data.parquet")?;
    ParquetWriter::new(&mut file).finish(&df)?;
    
    println!("Parquet file created successfully");
    
    Ok(())
}
EOF

        # Run the conversion
        echo "Running cargo to convert CSV to Parquet"
        (cd "$TEMP_DIR" && cargo run)
        
        # Clean up
        rm -rf "$TEMP_DIR"
        echo "Parquet file created successfully at data/example_data.parquet"
        ls -la data/
    else
        echo "Warning: cargo not found, Parquet conversion skipped"
        echo "The CSV file will be used instead"
    fi
fi

echo "Data format generation completed"
echo "Final data directory contents:"
ls -la data/
