#!/bin/bash

# This script is run before the template is applied
# It updates the template.json file based on the selected data format

# Get the data source from the environment variable
DATA_SOURCE="${DATA_SOURCE}"

# Run the select_template.sh script with the data source
./select_template.sh "$DATA_SOURCE"
