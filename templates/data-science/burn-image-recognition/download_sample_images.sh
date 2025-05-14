#!/bin/bash
set -e

echo "Downloading sample MNIST digit images..."

# Create sample_images directory if it doesn't exist
mkdir -p sample_images

# Download sample images
for i in {0..9}; do
    curl -s -o "sample_images/digit_$i.png" "https://raw.githubusercontent.com/tracel-ai/burn/main/examples/mnist/sample_images/digit_$i.png"
done

echo "✅ Sample images downloaded successfully!"
echo "You can now run 'cargo run -- predict --model-path ./model.json --image-path sample_images/digit_0.png' to test prediction."
