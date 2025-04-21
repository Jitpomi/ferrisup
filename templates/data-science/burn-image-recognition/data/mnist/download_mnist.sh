#!/bin/bash
# Script to download MNIST dataset files

MNIST_URL="http://yann.lecun.com/exdb/mnist"
FILES=(
  "train-images-idx3-ubyte.gz"
  "train-labels-idx1-ubyte.gz"
  "t10k-images-idx3-ubyte.gz"
  "t10k-labels-idx1-ubyte.gz"
)

mkdir -p "$(dirname "$0")"
cd "$(dirname "$0")" || exit

echo "Downloading MNIST dataset files..."
for file in "${FILES[@]}"; do
  if [ ! -f "${file%.gz}" ]; then
    echo "Downloading $file..."
    curl -O "$MNIST_URL/$file"
    echo "Extracting $file..."
    gunzip "$file"
  else
    echo "File ${file%.gz} already exists. Skipping."
  fi
done

echo "MNIST dataset files downloaded and extracted successfully!"
