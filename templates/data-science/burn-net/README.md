# Burn Neural Network Example

This is a machine learning example using the [Burn](https://github.com/tracel-ai/burn) framework for Rust. Burn is a deep learning framework written in pure Rust that supports automatic differentiation, GPU acceleration, and various neural network architectures.

## Features

- Train a convolutional neural network (CNN) on the MNIST dataset
- Evaluate the model on test data
- Save and load trained models
- Based on the official Burn examples

## Usage

### Training a Model

To train a model, use the `train` command:

```bash
cargo run -- train --epochs 10 --output model.json --batch-size 32
```

Options:
- `--epochs`: Number of training epochs (default: 10)
- `--output`: Path to save the trained model (default: model.json)
- `--batch-size`: Batch size for training (default: 32)

### Evaluating a Model

To evaluate a trained model on the MNIST test dataset:

```bash
cargo run -- evaluate --model model.json
```

Options:
- `--model`: Path to the trained model file

## Model Architecture

This example implements a CNN with the following architecture:

1. Three convolutional blocks, each containing:
   - 2D Convolution layer
   - Batch normalization
   - GELU activation

2. A fully connected network with:
   - Dropout for regularization
   - Two linear layers
   - GELU activation

## GPU Acceleration

By default, this example uses the CPU backend. To enable GPU acceleration, uncomment the GPU-related dependencies in `Cargo.toml` and modify the backend type in `main.rs`.

## Learn More

To learn more about the Burn framework:

- [Burn GitHub Repository](https://github.com/tracel-ai/burn)
- [Burn Documentation](https://burn.dev/)
- [The Burn Book](https://burn.dev/burn-book/)

This template is based on the official Burn examples and follows best practices for machine learning in Rust.
