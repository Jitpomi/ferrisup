# Text Sentiment Analyzer with Burn

A highly customizable text sentiment analysis template using the Burn deep learning framework for Rust.

## Overview

This template provides a complete, customizable solution for text sentiment analysis tasks. It's designed to be:

- **Beginner-friendly**: Clear code structure with extensive documentation
- **Highly customizable**: Easily adaptable for your specific text analysis needs
- **Production-ready**: Includes training, evaluation, and inference capabilities

## Features

- LSTM (Long Short-Term Memory) architecture for text sentiment analysis
- Support for custom text datasets with any number of sentiment classes
- Comprehensive tokenization and text preprocessing
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

This template is structured as a library with example applications. The main example is a CLI tool for training and using text sentiment analysis models.

To train a model on your own dataset:

```bash
# Prepare your CSV data file with text and sentiment labels:
# text,sentiment
# "This product is amazing!",positive
# "I'm disappointed with the quality.",negative
# "It works as expected.",neutral
# ...

# Train the model (with CPU backend)
cargo run --example text_analyzer --features ndarray -- train --data-file ./data.csv --epochs 10

# Train with GPU acceleration (if available)
cargo run --example text_analyzer --features tch-gpu -- train --data-file ./data.csv --epochs 10
```

To evaluate the model:

```bash
cargo run --example text_analyzer --features ndarray -- evaluate --model ./model.bin --data-file ./test_data.csv
```

To analyze a single text:

```bash
cargo run --example text_analyzer --features ndarray -- analyze --model ./model.bin --text "This product is amazing!"
```

## Customization

This template is designed to be easily customizable. See the [CUSTOMIZATION.md](CUSTOMIZATION.md) file for detailed instructions on how to adapt this template for your specific needs.

Key customization points include:

- Model architecture (LSTM layers, embedding dimensions, etc.)
- Training parameters (learning rate, batch size, etc.)
- Tokenization and text preprocessing
- Vocabulary size and handling

## Project Structure

```
src/
├── lib.rs          # Library entry point and exports
├── config.rs       # Configuration parameters
├── data.rs         # Text loading and processing
├── model.rs        # Neural network architecture
└── training.rs     # Training and evaluation logic

examples/
└── text_analyzer.rs  # CLI application for training and inference
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
