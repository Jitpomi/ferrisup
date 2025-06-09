#!/bin/bash
# Script to download or generate sample data files for the polars-cli template

# Create a simple CSV file with the same data that was intended to be in the Parquet
echo "Creating sample CSV file..."
cat > example_data.csv << 'EOF'
id,name,department,salary,age
1,Alice,Engineering,75000,28
2,Bob,Sales,65000,35
3,Charlie,Marketing,60000,42
4,David,Engineering,78000,31
5,Eve,HR,55000,29
6,Frank,Sales,68000,38
7,Grace,Marketing,62000,33
8,Hannah,Engineering,79000,27
9,Ian,Finance,72000,45
10,Julia,HR,56000,36
EOF

echo "Sample CSV file created successfully: example_data.csv"

# Also create a sample JSON file
echo "Creating sample JSON file..."
cat > example_data.json << 'EOF'
[
  {"id": 1, "name": "Alice", "department": "Engineering", "salary": 75000, "age": 28},
  {"id": 2, "name": "Bob", "department": "Sales", "salary": 65000, "age": 35},
  {"id": 3, "name": "Charlie", "department": "Marketing", "salary": 60000, "age": 42},
  {"id": 4, "name": "David", "department": "Engineering", "salary": 78000, "age": 31},
  {"id": 5, "name": "Eve", "department": "HR", "salary": 55000, "age": 29},
  {"id": 6, "name": "Frank", "department": "Sales", "salary": 68000, "age": 38},
  {"id": 7, "name": "Grace", "department": "Marketing", "salary": 62000, "age": 33},
  {"id": 8, "name": "Hannah", "department": "Engineering", "salary": 79000, "age": 27},
  {"id": 9, "name": "Ian", "department": "Finance", "salary": 72000, "age": 45},
  {"id": 10, "name": "Julia", "department": "HR", "salary": 56000, "age": 36}
]
EOF

echo "Sample JSON file created successfully: example_data.json"

echo "All sample data files created successfully!"
