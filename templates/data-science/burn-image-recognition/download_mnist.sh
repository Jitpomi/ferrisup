#!/bin/bash

# Create directory for MNIST dataset
mkdir -p data/mnist

echo "Downloading MNIST dataset..."

# Use a more reliable mirror for the MNIST dataset
MNIST_BASE_URL="https://storage.googleapis.com/cvdf-datasets/mnist"

# Download MNIST dataset files
curl -L -o data/mnist/train-images-idx3-ubyte.gz "${MNIST_BASE_URL}/train-images-idx3-ubyte.gz"
curl -L -o data/mnist/train-labels-idx1-ubyte.gz "${MNIST_BASE_URL}/train-labels-idx1-ubyte.gz"
curl -L -o data/mnist/t10k-images-idx3-ubyte.gz "${MNIST_BASE_URL}/t10k-images-idx3-ubyte.gz"
curl -L -o data/mnist/t10k-labels-idx1-ubyte.gz "${MNIST_BASE_URL}/t10k-labels-idx1-ubyte.gz"

# Extract the files
echo "Extracting MNIST dataset files..."
gunzip -f data/mnist/train-images-idx3-ubyte.gz
gunzip -f data/mnist/train-labels-idx1-ubyte.gz
gunzip -f data/mnist/t10k-images-idx3-ubyte.gz
gunzip -f data/mnist/t10k-labels-idx1-ubyte.gz

# Also download the pre-trained model file
cd ..
if [ ! -f "model.mpk" ]; then
  echo "Downloading pre-trained model file..."
  # This is just a placeholder URL - replace with actual model hosting location
  curl -L -o model.mpk "https://example.com/models/mnist_model.mpk" || {
    echo "Could not download model file. You'll need to train your own model using:"
    echo "cargo run -- train"
  }
else
  echo "Model file already exists. Skipping."
fi

echo " MNIST dataset downloaded and extracted to data/mnist/ directory"
echo "You can now train the model with: cargo run -- train"
