#!/bin/bash

# This script runs after the template is generated
# It updates the next steps displayed to the user based on the selected data format

# Get the data format from the template variables
DATA_FORMAT="${data_format}"

# Update the next steps in the template.json file
if [ "$DATA_FORMAT" == "csv" ]; then
    echo "CSV format selected"
    NEXT_STEPS=(
        "📊 Try the example analysis: cd {{project_name}} && cargo run -- analyze -f data/example_data.csv"
        "📈 Run statistical analysis: cargo run -- analyze -f data/example_data.csv -s"
        "🔍 Group data by department: cargo run -- analyze -f data/example_data.csv -g department -a salary -u mean"
        "🧮 Generate sample data: cargo run -- generate -r 100 -o data/my_data.csv"
        "📚 See all available commands: cargo run -- help"
    )
elif [ "$DATA_FORMAT" == "json" ]; then
    echo "JSON format selected"
    NEXT_STEPS=(
        "📊 Try the example analysis: cd {{project_name}} && cargo run -- analyze -f data/example_data.json"
        "📈 Run statistical analysis: cargo run -- analyze -f data/example_data.json -s"
        "🔍 Group data by department: cargo run -- analyze -f data/example_data.json -g department -a salary -u mean"
        "🧮 Generate sample data: cargo run -- generate -r 100 -o data/my_data.json"
        "📚 See all available commands: cargo run -- help"
    )
else
    echo "Parquet format selected"
    NEXT_STEPS=(
        "📊 Try the example analysis: cd {{project_name}} && cargo run -- analyze -f data/example_data.parquet"
        "📈 Run statistical analysis: cargo run -- analyze -f data/example_data.parquet -s"
        "🔍 Group data by department: cargo run -- analyze -f data/example_data.parquet -g department -a salary -u mean"
        "🧮 Generate sample data: cargo run -- generate -r 100 -o data/my_data.parquet"
        "📚 See all available commands: cargo run -- help"
    )
fi

# Print the next steps for debugging
echo "Next steps:"
for step in "${NEXT_STEPS[@]}"; do
    echo "  $step"
done
