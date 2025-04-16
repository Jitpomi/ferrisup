# MNIST Digit Recognition with Burn

This project demonstrates how to build a handwritten digit recognition system using the Burn deep learning framework in Rust. It uses the MNIST dataset, which contains 70,000 images of handwritten digits (60,000 for training and 10,000 for testing).

## Features

- **Convolutional Neural Network (CNN)** architecture for image classification
- **MNIST dataset** handling with automatic download
- **Training pipeline** with validation
- **Model evaluation** with accuracy metrics
- **Single image prediction** for testing with your own handwritten digits

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version recommended)
- The `wasm32-unknown-unknown` target (required by Burn)

### Downloading the Dataset

Before training, you need to download the MNIST dataset:

```bash
./download_mnist.sh
```

This script will download and extract the MNIST dataset files to the `data/mnist` directory.

### Training a Model

To train a new model on the MNIST dataset:

```bash
cargo run -- train
```

This will:
1. Load the MNIST dataset from the `data/mnist` directory
2. Train a CNN model for 10 epochs
3. Save the trained model to `model.json`

You can customize the training with these options:
- `--epochs` or `-e`: Number of training epochs (default: 10)
- `--batch-size` or `-b`: Batch size for training (default: 64)
- `--learning-rate` or `-l`: Learning rate for the optimizer (default: 0.001)
- `--model-path` or `-m`: Path to save the model (default: ./model.json)

Example with custom parameters:
```bash
cargo run -- train --epochs 20 --batch-size 128 --learning-rate 0.0005
```

### Evaluating the Model

To evaluate a trained model on the MNIST test set:

```bash
cargo run -- evaluate --model-path ./model.json
```

This will output the accuracy and loss metrics for the model on the test set.

### Predicting with Your Own Images

To test the model with sample MNIST images:

```bash
./download_sample_images.sh
```

This will download 10 sample images (one for each digit) to the `sample_images` directory.

To predict the digit in an image:

```bash
cargo run -- predict --model-path ./model.json --image-path sample_images/digit_0.png
```

For best results with your own images:
- Use a white digit on a black background
- Center the digit in the image
- The image will be automatically resized to 28x28 pixels

## Model Architecture

The CNN architecture used in this project consists of:

1. Three convolutional blocks, each containing:
   - 2D convolution layer
   - Batch normalization
   - GELU activation function

2. Fully connected layers:
   - Dropout (0.5) for regularization
   - Hidden layer with 32 neurons
   - Output layer with 10 neurons (one for each digit)

## Customization

You can customize the model architecture by modifying the `model.rs` file:

- Change the number of convolutional layers
- Adjust the number of filters in each layer
- Modify the kernel sizes
- Change the activation functions
- Adjust the dropout rate

## Performance

With the default settings, the model typically achieves:
- Training accuracy: ~99%
- Test accuracy: ~98%

## License

This project is open source and available under the MIT License.

## Acknowledgments

- The [Burn Framework](https://github.com/tracel-ai/burn) for providing a powerful deep learning library in Rust
- The MNIST dataset creators for providing this benchmark dataset
