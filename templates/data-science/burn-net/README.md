# Burn-Net - Deep Learning in Rust

This project provides a foundation for deep learning in Rust using [Burn](https://github.com/burn-rs/burn), a deep learning framework written in pure Rust (similar to PyTorch in Python).

## Features

- Define neural network models with Burn
- Train models on CPU or GPU
- Load and save model weights
- Work with the MNIST dataset
- Make predictions on new images

## Getting Started

### Training a Model

```bash
# Train a model on MNIST for 10 epochs
cargo run -- train -e 10 -b 32 -l 0.001 -o model.burn

# Train with custom parameters
cargo run -- train --epochs 20 --batch-size 64 --learning-rate 0.0005 --output my_model.burn
```

### Evaluating a Model

```bash
# Evaluate on MNIST test set
cargo run -- evaluate -m model.burn

# Evaluate on a single image
cargo run -- evaluate -m model.burn -i path/to/image.png
```

### Making Predictions

```bash
# Predict the digit in an image
cargo run -- predict -m model.burn -i path/to/digit.png
```

## Model Architecture

The default model is a Convolutional Neural Network (CNN) with:

- 2 convolutional layers
- Adaptive average pooling
- 2 fully connected layers
- ReLU activations

You can modify the architecture in `src/model.rs` to suit your needs.

## Extending

This template provides a foundation for deep learning in Rust. To extend it:

1. **Add new models**: Define new neural network architectures in `src/model.rs`
2. **Work with different datasets**: Implement dataset loaders in `src/dataset.rs`
3. **Experiment with training**: Modify the training loop in `src/train.rs`
4. **Add GPU support**: Change the backend to use CUDA or Metal

## GPU Support

To enable GPU support, modify the backend type in `src/main.rs`:

```rust
// For CUDA
type MyBackend = burn_cuda::Cuda<f32>;

// For Metal (macOS)
type MyBackend = burn_metal::Metal<f32>;
```

And update your Cargo.toml to include the appropriate backend:

```toml
[dependencies]
# For CUDA
burn-cuda = "0.12.1"

# For Metal
burn-metal = "0.12.1"
```

## Resources

- [Burn Documentation](https://burn-rs.github.io/book/)
- [Burn Examples](https://github.com/burn-rs/burn/tree/main/examples)
- [Rust Deep Learning](https://github.com/LaurentMazare/tch-rs)
