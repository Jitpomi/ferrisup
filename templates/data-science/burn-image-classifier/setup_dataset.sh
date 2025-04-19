#!/bin/bash
# Interactive script to set up datasets for the image classifier

# Function to display a menu and get user selection
function show_menu() {
    local prompt=$1
    shift
    local options=("$@")
    local selected=0
    
    echo "$prompt"
    for i in "${!options[@]}"; do
        echo "  $((i+1)). ${options[$i]}"
    done
    
    read -p "Enter your choice (1-${#options[@]}): " choice
    
    # Validate input
    if [[ $choice =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le "${#options[@]}" ]; then
        selected=$((choice-1))
    else
        echo "Invalid choice. Using default option 1."
        selected=0
    fi
    
    echo "${options[$selected]}"
}

# Function to ask yes/no questions
function ask_yes_no() {
    local prompt=$1
    local default=$2
    
    if [ "$default" = "Y" ]; then
        read -p "$prompt [Y/n]: " choice
        [ -z "$choice" ] && choice="Y"
    else
        read -p "$prompt [y/N]: " choice
        [ -z "$choice" ] && choice="N"
    fi
    
    case "$choice" in
        [Yy]*)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Welcome message
echo "🧠 Image Classifier Dataset Setup"
echo "================================="
echo "This script will help you set up datasets for your image classifier."
echo ""

# Ask which datasets to generate
echo "Available datasets:"
echo "  1. CIFAR-10 (10 classes, 60,000 images of objects)"
echo "  2. MNIST (10 classes, 70,000 handwritten digits)"
echo "  3. Fashion-MNIST (10 classes, 70,000 fashion items)"
echo "  4. Synthetic (customizable generated dataset)"
echo "  5. All of the above"
echo "  6. None (skip dataset generation for now)"
echo ""

read -p "Which dataset(s) would you like to generate? (1-6, default: 1): " dataset_choice
[ -z "$dataset_choice" ] && dataset_choice="1"

# Set up output directories
base_dir="datasets"
mkdir -p "$base_dir"

# Process dataset choices
case "$dataset_choice" in
    1)
        # CIFAR-10
        echo "Generating CIFAR-10 dataset..."
        ./generate_sample_data.sh --dataset cifar10 --output-dir "$base_dir/cifar10"
        echo "Default dataset set to CIFAR-10"
        ln -sf "$base_dir/cifar10" sample-data
        ;;
    2)
        # MNIST
        echo "Generating MNIST dataset..."
        ./generate_sample_data.sh --dataset mnist --output-dir "$base_dir/mnist"
        echo "Default dataset set to MNIST"
        ln -sf "$base_dir/mnist" sample-data
        ;;
    3)
        # Fashion-MNIST
        echo "Generating Fashion-MNIST dataset..."
        ./generate_sample_data.sh --dataset fashion-mnist --output-dir "$base_dir/fashion-mnist"
        echo "Default dataset set to Fashion-MNIST"
        ln -sf "$base_dir/fashion-mnist" sample-data
        ;;
    4)
        # Synthetic
        echo "Generating Synthetic dataset..."
        
        # Ask for number of classes
        read -p "How many classes would you like? (default: 10): " num_classes
        [ -z "$num_classes" ] && num_classes=10
        
        # Ask for images per class
        read -p "How many images per class? (default: 100): " images_per_class
        [ -z "$images_per_class" ] && images_per_class=100
        
        ./generate_sample_data.sh --dataset synthetic --output-dir "$base_dir/synthetic" --num-classes "$num_classes" --images-per-class "$images_per_class"
        echo "Default dataset set to Synthetic"
        ln -sf "$base_dir/synthetic" sample-data
        ;;
    5)
        # All datasets
        echo "Generating all datasets (this may take a while)..."
        
        # CIFAR-10
        echo "Generating CIFAR-10 dataset..."
        ./generate_sample_data.sh --dataset cifar10 --output-dir "$base_dir/cifar10"
        
        # MNIST
        echo "Generating MNIST dataset..."
        ./generate_sample_data.sh --dataset mnist --output-dir "$base_dir/mnist"
        
        # Fashion-MNIST
        echo "Generating Fashion-MNIST dataset..."
        ./generate_sample_data.sh --dataset fashion-mnist --output-dir "$base_dir/fashion-mnist"
        
        # Synthetic
        echo "Generating Synthetic dataset..."
        ./generate_sample_data.sh --dataset synthetic --output-dir "$base_dir/synthetic"
        
        echo "Default dataset set to CIFAR-10"
        ln -sf "$base_dir/cifar10" sample-data
        ;;
    6)
        # Skip dataset generation
        echo "Skipping dataset generation."
        echo "You can generate datasets later using ./generate_sample_data.sh"
        ;;
    *)
        # Invalid choice, default to CIFAR-10
        echo "Invalid choice. Generating CIFAR-10 dataset..."
        ./generate_sample_data.sh --dataset cifar10 --output-dir "$base_dir/cifar10"
        echo "Default dataset set to CIFAR-10"
        ln -sf "$base_dir/cifar10" sample-data
        ;;
esac

# Ask if user wants to train the model now
if [ "$dataset_choice" != "6" ]; then
    if ask_yes_no "Would you like to train the model on the default dataset now?" "N"; then
        echo "Starting training with 5 epochs..."
        cargo run --bin $(basename $(pwd)) -- train --data-dir datasets/mnist --epochs 5
    fi
    
    echo "Dataset setup complete!"
    echo "To train the model:"
    echo "  cargo run --bin $(basename $(pwd)) -- train --data-dir datasets/mnist"
    echo "or:"
    echo "  cargo run --bin $(basename $(pwd)) -- train --data-dir datasets/cifar10"
fi

echo ""
echo "Setup complete! 🎉"
echo ""
echo "Available datasets:"
ls -la "$base_dir"
echo ""
echo "Next steps:"
echo "  1. Train the model: cargo run --bin $(basename $(pwd)) -- train --data-dir datasets/mnist"
echo "  2. Evaluate the model: cargo run --bin $(basename $(pwd)) -- evaluate --model-path model.json --data-dir datasets/mnist"
echo "  3. Customize the model: Edit src/config.rs to adjust parameters"
echo "  4. Learn more: Read CUSTOMIZATION.md for detailed customization options"
