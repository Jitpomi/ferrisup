# Value Prediction with Rust

This template helps you build a system that predicts numeric values (like house prices, stock values, or temperatures) using Rust. It uses machine learning to find patterns in your data and make predictions.

## 🔍 What This Template Does

- Loads and processes tabular data (like CSV files)
- Trains a neural network to predict values
- Evaluates the model's accuracy
- Makes predictions on new data

## 🚀 Getting Started

### Running the Example

1. Train the model:
   ```bash
   cargo run -- train --epochs 100 --data data/housing.csv
   ```

2. Make predictions:
   ```bash
   cargo run -- predict --model model.json --input data/test.csv
   ```

## 📊 Understanding the Code

### Model Architecture

This template uses a simple Multi-Layer Perceptron (MLP) with:

- **Input layer**: Takes your data features
- **Hidden layers**: Learn patterns in the data
- **Output layer**: Produces the predicted value

### Key Components

- **data.rs**: Handles loading and processing CSV data
- **model.rs**: Defines the neural network architecture
- **main.rs**: Contains the training and prediction logic

## 🔧 Customizing for Your Needs

### Using Your Own Data

To use your own data:
1. Prepare your data in CSV format
2. Make sure numeric columns don't have missing values
3. Modify the data loading code to match your column names
4. Adjust the number of input features if needed

### Adjusting the Model

- Change the number of hidden layers or their sizes in `model.rs`
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
   linfa-linear = "0.7.0"
   linfa-datasets = "0.7.0"
   ndarray = "0.15.6"
   ```

### Training Issues

If your model isn't learning well:
- Normalize your data (scale all features to similar ranges)
- Try different model architectures
- Increase the number of training examples

## 📚 Learning Resources

- [Burn Documentation](https://burn.dev/)
- [Linfa Documentation](https://github.com/rust-ml/linfa)
- [Neural Networks Explained](https://www.3blue1brown.com/topics/neural-networks)
- [Regression Analysis Tutorial](https://www.kaggle.com/learn/intro-to-machine-learning)

## 📝 License

This template is based on examples from the Burn framework and is available under the same license as Burn.
