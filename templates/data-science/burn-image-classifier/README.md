# Image Classifier with Burn

A highly customizable image classification template using the Burn deep learning framework for Rust.

## Overview

This template provides a complete, customizable solution for image classification tasks. It's designed to be:

- **Beginner-friendly**: Clear code structure with extensive documentation
- **Highly customizable**: Easily adaptable for your specific image classification needs
- **Production-ready**: Includes training, evaluation, and inference capabilities

## Features

- Convolutional Neural Network (CNN) architecture for image classification
- Support for custom datasets with any number of classes
- Data augmentation to improve model generalization
- Comprehensive metrics and evaluation tools
- Multiple backend support (CPU, GPU via CUDA/MPS, WebGPU)

## Getting Started

### Prerequisites

- Rust toolchain (1.71.0 or newer)
- For GPU acceleration:
  - CUDA toolkit (for NVIDIA GPUs)
  - Metal (for Apple Silicon)
  - WebGPU-compatible system

### Running the Examples

This template is structured as a library with example applications. The main example is a CLI tool for training and using image classification models.

To train a model on your own dataset:

```bash
# First, organize your images in class subdirectories:
# data/
#   ├── class1/
#   │   ├── image1.jpg
#   │   ├── image2.jpg
#   │   └── ...
#   ├── class2/
#   │   ├── image1.jpg
#   │   └── ...
#   └── ...

# Train the model (with CPU backend)
cargo run --example image_classifier --features ndarray -- train --data-dir ./data --epochs 10

# Train with GPU acceleration (if available)
cargo run --example image_classifier --features tch-gpu -- train --data-dir ./data --epochs 10
```

To evaluate the model:

```bash
cargo run --example image_classifier --features ndarray -- evaluate --model ./model.bin --data-dir ./test_data
```

To classify a single image:

```bash
cargo run --example image_classifier --features ndarray -- predict --model ./model.bin --image ./my_image.jpg
```

## Customization

This template is designed to be easily customizable. See the [CUSTOMIZATION.md](CUSTOMIZATION.md) file for detailed instructions on how to adapt this template for your specific needs.

Key customization points include:

- Model architecture (layers, activation functions, etc.)
- Training parameters (learning rate, batch size, etc.)
- Data augmentation techniques
- Input image size and preprocessing

## Project Structure

```
src/
├── lib.rs          # Library entry point and exports
├── config.rs       # Configuration parameters
├── data.rs         # Data loading and processing
├── model.rs        # Neural network architecture
└── training.rs     # Training and evaluation logic

examples/
└── image_classifier.rs  # CLI application for training and inference
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
