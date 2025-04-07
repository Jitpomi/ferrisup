# Image Recognition with Linfa

This project demonstrates how to use Linfa, Rust's machine learning framework, for image recognition tasks. Specifically, it focuses on handwritten digit recognition, similar to the MNIST dataset.

## Features

- **Training**: Train a K-Nearest Neighbors model on synthetic or real data
- **Evaluation**: Evaluate model performance with accuracy metrics and confusion matrix
- **Prediction**: Recognize digits in individual images
- **Synthetic Data**: Generate synthetic data for training and testing when real data is unavailable

## Getting Started

### Training a Model

```bash
# Train with default parameters (k=5)
cargo run -- train

# Train with custom parameters
cargo run -- train -k 3 -o my_model.json
```

### Evaluating a Model

```bash
# Evaluate the model
cargo run -- evaluate -m model.json
```

### Making Predictions

```bash
# Predict the digit in an image
cargo run -- predict -m model.json -i path/to/image.png
```

## How It Works

This application uses the K-Nearest Neighbors algorithm from Linfa to classify handwritten digits. The algorithm works by:

1. Storing all training examples with their known labels
2. For a new image, finding the k closest training examples (in terms of pixel similarity)
3. Assigning the most common label among those neighbors

## Project Structure

- `src/main.rs`: Main application logic and CLI interface
- `src/data.rs`: Data handling functions for loading and preprocessing images

## Dependencies

- **Linfa**: Rust's machine learning framework
- **ndarray**: N-dimensional arrays for numerical computation
- **image**: Image processing library
- **clap**: Command-line argument parsing

## Extending the Project

You can extend this project in several ways:

1. Add support for downloading and using the real MNIST dataset
2. Implement other classification algorithms (SVM, Random Forest, etc.)
3. Add visualization of the input images and predictions
4. Create a web interface for uploading and classifying images

## License

This project is licensed under the MIT License - see the LICENSE file for details.
