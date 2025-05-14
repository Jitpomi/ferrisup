#!/bin/bash
set -e

echo "Downloading MNIST dataset..."

# Create data directory if it doesn't exist
mkdir -p data/mnist

# Download MNIST dataset files
curl -s -o data/mnist/train-images-idx3-ubyte.gz http://yann.lecun.com/exdb/mnist/train-images-idx3-ubyte.gz
curl -s -o data/mnist/train-labels-idx1-ubyte.gz http://yann.lecun.com/exdb/mnist/train-labels-idx1-ubyte.gz
curl -s -o data/mnist/t10k-images-idx3-ubyte.gz http://yann.lecun.com/exdb/mnist/t10k-images-idx3-ubyte.gz
curl -s -o data/mnist/t10k-labels-idx1-ubyte.gz http://yann.lecun.com/exdb/mnist/t10k-labels-idx1-ubyte.gz

# Extract the files
gunzip -f data/mnist/train-images-idx3-ubyte.gz
gunzip -f data/mnist/train-labels-idx1-ubyte.gz
gunzip -f data/mnist/t10k-images-idx3-ubyte.gz
gunzip -f data/mnist/t10k-labels-idx1-ubyte.gz

# Create a placeholder model.json file if it doesn't exist
if [ ! -f model.json ]; then
    echo "{}" > model.json
fi

echo " MNIST dataset downloaded and extracted successfully!"
echo "You can now run 'cargo run -- train' to train the model."
