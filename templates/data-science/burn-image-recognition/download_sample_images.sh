#!/bin/bash

# Create directory for sample images
mkdir -p sample_images

# Download 10 sample MNIST images (one for each digit)
echo "Downloading sample MNIST images..."

# Use a more reliable source for sample MNIST images
# These are hosted on GitHub in a public repository
URLS=(
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/0.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/1.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/2.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/3.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/4.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/5.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/6.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/7.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/8.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/9.png"
)

# Download each image
for i in {0..9}; do
  echo "Downloading digit $i sample..."
  curl -L -s "${URLS[$i]}" -o "sample_images/digit_$i.png"
done

echo "✅ Sample images downloaded to sample_images/ directory"
echo "You can use these images for testing with the predict command:"
echo "cargo run -- predict --model-path model.json --image-path sample_images/digit_0.png"
