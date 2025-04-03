# Image Recognition with Rust

This template helps you build an image recognition system using Rust. It's designed to recognize handwritten digits using the MNIST dataset, but you can adapt it for your own image classification needs.

## 🔍 What This Template Does

- Loads and processes the MNIST dataset (handwritten digits)
- Trains a neural network to recognize digits
- Evaluates the model's accuracy
- Allows you to make predictions on new images

## 🚀 Getting Started

### Running the Example

1. Train the model:
   ```bash
   cargo run -- train --epochs 10
   ```

2. Evaluate the model:
   ```bash
   cargo run -- evaluate --model model.json
   ```

## 📊 Understanding the Code

### Model Architecture

This template uses a Convolutional Neural Network (CNN) with:

- **Convolutional layers**: Extract features from images
- **Batch normalization**: Stabilize training
- **Dropout**: Prevent overfitting
- **Fully connected layers**: Make the final classification

### Key Components

- **data.rs**: Handles loading and processing the MNIST dataset
- **model.rs**: Defines the neural network architecture
- **main.rs**: Contains the training and evaluation logic

## 🔧 Customizing for Your Needs

### Using Your Own Images

To use your own images:
1. Prepare your images in 28x28 pixel grayscale format
2. Modify the data loading code to use your dataset
3. Adjust the number of output classes if needed

### Adjusting the Model

- Change the number of layers or their sizes in `model.rs`
- Modify the learning rate in `main.rs` to adjust training speed
- Increase epochs for potentially better accuracy

## 🛠️ Troubleshooting

### Dependency Issues

If you encounter issues with the Burn framework, you have two options:

1. **Use an older version**: Try specifying `burn = "0.8.0"` in Cargo.toml
2. **Switch to Linfa**: Linfa is a more stable alternative Rust ML library
   ```toml
   # Replace Burn with Linfa in Cargo.toml
   linfa = "0.7.0"
   linfa-nn = "0.7.0"
   linfa-datasets = "0.7.0"
   ndarray = "0.15.6"
   ```

### Memory Usage

If you run into memory issues:
- Reduce the batch size in `main.rs`
- Use a simpler model architecture
- Process images at a lower resolution

## 📚 Learning Resources

- [Burn Documentation](https://burn.dev/)
- [Linfa Documentation](https://github.com/rust-ml/linfa)
- [Neural Networks Explained](https://www.3blue1brown.com/topics/neural-networks)
- [Convolutional Neural Networks Tutorial](https://cs231n.github.io/convolutional-networks/)

## 📝 License

This template is based on examples from the Burn framework and is available under the same license as Burn.
