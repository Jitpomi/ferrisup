# Image Classifier using Burn

This is a deep learning template for classifying images into categories using the [Burn](https://burn.dev/) framework. It provides a complete solution for training, evaluating, and using a convolutional neural network (CNN) for image classification tasks.

## What is Deep Learning? (For Beginners)

Deep learning is a type of artificial intelligence that teaches computers to learn from examples, similar to how humans learn. In this template:

1. We use a **Convolutional Neural Network (CNN)** - a special type of AI model that's excellent at understanding images
2. The model learns to recognize patterns in images through a process called **training**
3. Once trained, it can identify new images it has never seen before

Think of it like teaching a child to recognize animals by showing them many examples. After seeing enough cats and dogs, they can identify a new cat or dog they've never seen before.

## Why This Structure?

This template is organized in a modular way to make it easy to understand and customize:

- **Data Module** (`src/data.rs`): Handles loading and preparing images
- **Model Module** (`src/model.rs`): Defines the neural network structure
- **Config Module** (`src/config.rs`): Contains settings you can easily change
- **Visualization Module** (`src/visualization.rs`): Creates charts to help understand results
- **Main Application** (`src/main.rs`): Ties everything together with a user-friendly interface

This separation makes it easier to focus on one aspect at a time and customize the parts you need.

## Features

- **Complete CNN Architecture**: Pre-configured neural network optimized for image classification
- **Data Loading & Preprocessing**: Tools for loading and preparing images
- **Data Augmentation**: Automatically creates variations of your images to improve learning
- **Training Pipeline**: Full training process with progress tracking
- **Evaluation Tools**: Comprehensive testing with metrics and visualizations
- **Prediction Interface**: Simple commands for classifying new images
- **Visualization**: Training history plots and prediction charts
- **Sample Dataset**: Built-in sample data generator for quick testing
- **Unit Tests**: Comprehensive tests to ensure everything works correctly

## Getting Started

### Prerequisites

- Rust toolchain (1.70.0 or later recommended)
- Cargo package manager

### Quick Start

1. **Set up datasets interactively**:
   ```bash
   # This will guide you through setting up one or more datasets
   ./setup_dataset.sh
   ```
   
   Or choose a specific dataset manually:
   ```bash
   # Download and prepare the CIFAR-10 dataset (default)
   ./generate_sample_data.sh
   
   # Or choose a different dataset
   ./generate_sample_data.sh --dataset mnist
   ./generate_sample_data.sh --dataset fashion-mnist
   ./generate_sample_data.sh --dataset synthetic
   
   # For synthetic dataset, you can customize the number of classes and images
   ./generate_sample_data.sh --dataset synthetic --num-classes 5 --images-per-class 200
   ```

2. **Train the model**:
   ```bash
   # Train on the default dataset (linked as sample-data)
   cargo run --bin app -- train --data-dir sample-data
   
   # Or train on a specific dataset
   cargo run --bin app -- train --data-dir datasets/mnist
   ```

3. **Evaluate the model**:
   ```bash
   # Evaluate the trained model
   cargo run --bin app -- evaluate --model-path model.json --data-dir sample-data
   ```

4. **Predict on a single image**:
   ```bash
   # Predict the class of a single image
   cargo run --bin app -- predict --image-path path/to/your/image.jpg --model-path model.json
   ```

### Using Your Own Dataset

To use your own dataset instead of CIFAR-10:

1. **Organize your images** in a directory structure where each subdirectory represents a class:
   ```
   your-dataset/
   ├── class1/
   │   ├── image1.jpg
   │   ├── image2.jpg
   │   └── ...
   ├── class2/
   │   ├── image1.jpg
   │   ├── image2.jpg
   │   └── ...
   └── ...
   ```

2. **Update the configuration** in `src/config.rs` to match your dataset:
   - Set `NUM_CLASSES` to the number of classes in your dataset
   - Modify `CLASS_NAMES` to match your class names

3. **Train the model** on your dataset:
   ```bash
   cargo run --bin app -- train --data-dir path/to/your-dataset
   ```

### Customizing the Model

See the `CUSTOMIZATION.md` file for detailed instructions on how to modify the model architecture, training parameters, and other settings.

## Configuration

You can customize the model architecture, training parameters, and data preprocessing by editing the `src/config.rs` file. The main configurable parameters include:

- Image size and number of channels
- Number of classes and class names
- Model architecture (convolutional filters, fully connected layers)
- Training parameters (batch size, learning rate, epochs)
- Data augmentation options

## Examples

Check out the `examples/` directory for example code showing how to use the image classifier in your own projects.

## Visualization

The training process generates visualizations to help you understand the model's performance:

- `training_history.png`: Plot of training and validation loss and accuracy
- `confusion_matrix.png`: Confusion matrix showing model performance on each class
- `predictions.png`: Bar chart of top predictions for a single image

## Customization

For detailed information on how to customize this template for your specific needs, see the `CUSTOMIZATION.md` file.

## License

This template is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Burn Framework](https://burn.dev/) - The deep learning framework used
- [Image Crate](https://github.com/image-rs/image) - For image processing
- [Plotters](https://github.com/plotters-rs/plotters) - For visualization
