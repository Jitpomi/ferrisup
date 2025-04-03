#!/bin/bash
# Script to generate Burn framework templates using cargo-generate

# Check if cargo-generate is installed
if ! command -v cargo-generate &> /dev/null; then
    echo "cargo-generate is not installed. Installing now..."
    cargo install cargo-generate
fi

# Display available templates
echo "Available Burn Templates:"
echo "1. Image Recognition (MNIST)"
echo "2. Value Prediction (Regression)"
echo "3. Text Classification"
echo "4. Custom Image Dataset"
echo "5. Custom CSV Dataset"
echo "6. Custom Training Loop (Advanced)"
echo "7. Web-Based Image Classification"
echo ""

# Get user choice
read -p "Select a template (1-7): " choice
read -p "Enter a name for your project: " project_name

# Create the project based on user choice
case $choice in
    1)
        echo "Generating Image Recognition (MNIST) template..."
        cargo generate --git https://github.com/tracel-ai/burn.git --name "$project_name" examples/mnist
        ;;
    2)
        echo "Generating Value Prediction (Regression) template..."
        cargo generate --git https://github.com/tracel-ai/burn.git --name "$project_name" examples/simple-regression
        ;;
    3)
        echo "Generating Text Classification template..."
        cargo generate --git https://github.com/tracel-ai/burn.git --name "$project_name" examples/text-classification
        ;;
    4)
        echo "Generating Custom Image Dataset template..."
        cargo generate --git https://github.com/tracel-ai/burn.git --name "$project_name" examples/custom-image-dataset
        ;;
    5)
        echo "Generating Custom CSV Dataset template..."
        cargo generate --git https://github.com/tracel-ai/burn.git --name "$project_name" examples/custom-csv-dataset
        ;;
    6)
        echo "Generating Custom Training Loop template..."
        cargo generate --git https://github.com/tracel-ai/burn.git --name "$project_name" examples/custom-training-loop
        ;;
    7)
        echo "Generating Web-Based Image Classification template..."
        cargo generate --git https://github.com/tracel-ai/burn.git --name "$project_name" examples/image-classification-web
        ;;
    *)
        echo "Invalid choice. Please run the script again and select a number from 1-7."
        exit 1
        ;;
esac

echo ""
echo "Template generated successfully in ./$project_name"
echo "To get started with your project:"
echo "cd $project_name"
echo "cargo build"
echo ""
echo "For more information, refer to the README.md file in your project directory."
