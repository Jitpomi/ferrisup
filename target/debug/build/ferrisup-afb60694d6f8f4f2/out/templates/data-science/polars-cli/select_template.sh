#!/bin/bash

# Get the data source from the first argument
DATA_SOURCE="$1"

# Create a temporary file
TEMP_FILE=$(mktemp)

# Function to update template.json with format-specific next steps
update_template() {
  local format=$1
  local file_ext=$2
  
  cat > "$TEMP_FILE" << EOL
{
  "name": "polars-cli",
  "description": "Data analysis CLI using Polars (similar to pandas in Python)",
  "type": "binary",
  "files": [
    {
      "source": "src/main.rs.template",
      "target": "src/main.rs"
    },
    {
      "source": "Cargo.toml.template",
      "target": "Cargo.toml"
    },
    {
      "source": "README.md",
      "target": "README.md"
    },
    {
      "source": "data/example_data_csv.csv",
      "target": "data/example_data.csv",
      "condition": "data_source == \"CSV files\""
    },
    {
      "source": "data/example_data_json.json",
      "target": "data/example_data.json",
      "condition": "data_source == \"JSON data\""
    },
    {
      "source": "data/example_data_parquet.parquet",
      "target": "data/example_data.parquet",
      "condition": "data_source == \"Parquet files\""
    }
  ],
  "prompts": [
    {
      "name": "data_source",
      "question": "What type of data will you be working with?",
      "options": [
        "CSV files",
        "Parquet files",
        "JSON data"
      ],
      "default": "CSV files"
    },
    {
      "name": "analysis_type",
      "question": "What type of analysis do you plan to perform?",
      "options": [
        "Exploratory data analysis",
        "Statistical analysis",
        "Time series analysis",
        "Text analysis",
        "Machine learning preprocessing"
      ],
      "default": "Exploratory data analysis"
    },
    {
      "name": "visualization",
      "question": "Do you need data visualization capabilities?",
      "options": [
        "yes",
        "no"
      ],
      "default": "yes"
    }
  ],
  "variables": {
    "data_format": "{{#if (eq data_source \"CSV files\")}}csv{{else}}{{#if (eq data_source \"JSON data\")}}json{{else}}parquet{{/if}}{{/if}}"
  },
  "next_steps": [
    "📊 Try the example analysis: cd {{project_name}} && cargo run -- analyze -f data/example_data.${file_ext}",
    "📈 Run statistical analysis: cargo run -- analyze -f data/example_data.${file_ext} -s",
    "🔍 Group data by department: cargo run -- analyze -f data/example_data.${file_ext} -g department -a salary -u mean",
    "🧮 Generate sample data: cargo run -- generate -r 100 -o data/my_data.${file_ext}",
    "📚 See all available commands: cargo run -- help"
  ]
}
EOL

  # Replace the template.json file with the updated one
  mv "$TEMP_FILE" "template.json"
  echo "Updated template.json for $format format"
}

# Select the appropriate template file based on the data source
if [ "$DATA_SOURCE" = "CSV files" ]; then
  update_template "CSV" "csv"
elif [ "$DATA_SOURCE" = "JSON data" ]; then
  update_template "JSON" "json"
elif [ "$DATA_SOURCE" = "Parquet files" ]; then
  update_template "Parquet" "parquet"
else
  # Default to CSV if no valid option is provided
  update_template "CSV" "csv"
fi
