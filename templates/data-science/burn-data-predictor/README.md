# Data Predictor with Burn

A highly customizable numerical data prediction template using the Burn deep learning framework for Rust.

## Overview

This template provides a complete, customizable solution for numerical data prediction tasks. It's designed to be:

- **Beginner-friendly**: Clear code structure with extensive documentation
- **Highly customizable**: Easily adaptable for your specific prediction needs
- **Production-ready**: Includes training, evaluation, and inference capabilities

## Features

- Neural network architecture for numerical data prediction
- Support for custom datasets with any number of features and targets
- Comprehensive data preprocessing and normalization
- Detailed metrics and evaluation tools
- Multiple backend support (CPU, GPU via CUDA/MPS, WebGPU)

## Getting Started

### Prerequisites

- Rust toolchain (1.71.0 or newer)
- For GPU acceleration:
  - CUDA toolkit (for NVIDIA GPUs)
  - Metal (for Apple Silicon)
  - WebGPU-compatible system

### Running the Examples

This template is structured as a library with example applications. The main example is a CLI tool for training and using data prediction models.

To train a model on your own dataset:

```bash
# Prepare your CSV data file with features and target values:
# feature1,feature2,feature3,...,target
# 1.2,3.4,5.6,...,7.8
# 2.3,4.5,6.7,...,8.9
# ...

# Train the model (with CPU backend)
cargo run --example data_predictor --features ndarray -- train --data-file ./data.csv --epochs 10

# Train with GPU acceleration (if available)
cargo run --example data_predictor --features tch-gpu -- train --data-file ./data.csv --epochs 10
```

To evaluate the model:

```bash
cargo run --example data_predictor --features ndarray -- evaluate --model ./model.bin --data-file ./test_data.csv
```

To make predictions:

```bash
cargo run --example data_predictor --features ndarray -- predict --model ./model.bin --data-file ./new_data.csv --output predictions.csv
```

## Customization

This template is designed to be easily customizable. See the [CUSTOMIZATION.md](CUSTOMIZATION.md) file for detailed instructions on how to adapt this template for your specific needs.

Key customization points include:

- Model architecture (layers, activation functions, etc.)
- Training parameters (learning rate, batch size, etc.)
- Data preprocessing and normalization
- Input and output dimensions

## Project Structure

```
src/
├── lib.rs          # Library entry point and exports
├── config.rs       # Configuration parameters
├── data.rs         # Data loading and processing
├── model.rs        # Neural network architecture
└── training.rs     # Training and evaluation logic

examples/
└── data_predictor.rs  # CLI application for training and inference
```

## Backend Options

This template supports multiple backends through Burn's backend system:

- `ndarray`: CPU-based computation (default)
- `tch-cpu`: LibTorch CPU backend
- `tch-gpu`: LibTorch GPU backend (CUDA/MPS)
- `wgpu`: WebGPU backend for cross-platform GPU acceleration

Select a backend using the corresponding feature flag when running examples.

## License

This template is licensed under the MIT License - see the LICENSE file for details.
