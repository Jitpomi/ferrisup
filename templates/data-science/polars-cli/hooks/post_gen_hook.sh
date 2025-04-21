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

# Write the next steps to the .ferrisup_next_steps.json file
echo "{" > .ferrisup_next_steps.json
echo "  \"next_steps\": [" >> .ferrisup_next_steps.json

# Add each step with proper JSON escaping and the correct project name
for i in "${!NEXT_STEPS[@]}"; do
    # Replace {{project_name}} with the actual project name
    STEP="${NEXT_STEPS[$i]}"
    STEP="${STEP//{{project_name}}/${project_name}}"
    
    # Add comma for all but the last item
    if [ $i -lt $(( ${#NEXT_STEPS[@]} - 1 )) ]; then
        echo "    \"$STEP\"," >> .ferrisup_next_steps.json
    else
        echo "    \"$STEP\"" >> .ferrisup_next_steps.json
    fi
done

echo "  ]" >> .ferrisup_next_steps.json
echo "}" >> .ferrisup_next_steps.json

echo "✅ Updated next steps with correct file extensions"
