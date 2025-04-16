
#!/bin/bash
# Script to download and organize dataset for the image classifier

# Default dataset is CIFAR-10
DATASET="cifar10"
OUTPUT_DIR="sample-data"
NUM_CLASSES=10
IMAGES_PER_CLASS=100

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --dataset)
      DATASET="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --num-classes)
      NUM_CLASSES="$2"
      shift 2
      ;;
    --images-per-class)
      IMAGES_PER_CLASS="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  --dataset <dataset>    Dataset to download (default: cifar10)"
      echo "                         Supported datasets: cifar10, mnist, fashion-mnist, synthetic"
      echo "  --output-dir <dir>     Output directory (default: sample-data)"
      echo "  --num-classes <n>      Number of classes for synthetic dataset (default: 10)"
      echo "  --images-per-class <n> Number of images per class for synthetic dataset (default: 100)"
      echo "  --help                 Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

echo "Setting up dataset for the image classifier..."
echo "Dataset: $DATASET"
echo "Output directory: $OUTPUT_DIR"

# Create the output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Download and prepare the dataset
if [ "$DATASET" = "cifar10" ]; then
    echo "Downloading CIFAR-10 dataset..."
    cargo run --bin download_dataset -- --dataset cifar10 --output-dir "$OUTPUT_DIR"
elif [ "$DATASET" = "mnist" ]; then
    echo "Downloading MNIST dataset..."
    cargo run --bin download_dataset -- --dataset mnist --output-dir "$OUTPUT_DIR"
elif [ "$DATASET" = "fashion-mnist" ]; then
    echo "Downloading Fashion-MNIST dataset..."
    cargo run --bin download_dataset -- --dataset fashion-mnist --output-dir "$OUTPUT_DIR"
elif [ "$DATASET" = "synthetic" ]; then
    echo "Generating synthetic dataset..."
    cargo run --bin download_dataset -- --dataset synthetic --output-dir "$OUTPUT_DIR" --num-classes "$NUM_CLASSES" --images-per-class "$IMAGES_PER_CLASS"
else
    echo "Error: Unsupported dataset '$DATASET'"
    echo "Supported datasets: cifar10, mnist, fashion-mnist, synthetic"
    exit 1
fi

echo "Sample data setup complete!"
echo "You can find the sample data in the '$OUTPUT_DIR' directory."
echo ""
echo "Next steps:"
echo "1. Train the model: cargo run --bin app -- train --data-dir $OUTPUT_DIR"
echo "2. Evaluate the model: cargo run --bin app -- evaluate --model-path model.json --data-dir $OUTPUT_DIR"
