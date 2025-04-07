# Customizing the Data Predictor

This guide explains how to customize the Data Predictor template for your specific needs. All major customization points are clearly marked with `// CUSTOMIZE HERE` comments in the code.

## Quick Customization

For most use cases, you only need to modify the `config.rs` file, which centralizes all tweakable parameters.

## 1. Adjusting Data Processing

In `config.rs`, you can modify:

```rust
// Data processing parameters
pub const FEATURE_COLUMNS: [&str; 5] = ["feature1", "feature2", "feature3", "feature4", "feature5"];
pub const TARGET_COLUMN: &str = "target";
pub const NORMALIZE_FEATURES: bool = true;     // Whether to normalize input features
pub const NORMALIZE_TARGET: bool = true;       // Whether to normalize target values
pub const TEST_SPLIT_RATIO: f32 = 0.2;         // Portion of data to use for testing
```

- **Change `FEATURE_COLUMNS`** to match your CSV column names
- **Change `TARGET_COLUMN`** to your prediction target column
- **Set `NORMALIZE_FEATURES` or `NORMALIZE_TARGET` to false** if your data is already normalized
- **Adjust `TEST_SPLIT_RATIO`** to change the train/test split

## 2. Changing the Model Architecture

In `config.rs`, you can adjust:

```rust
// Model architecture
pub const HIDDEN_LAYERS: [usize; 2] = [64, 32]; // Size of hidden layers
pub const ACTIVATION: &str = "relu";           // Activation function: "relu", "tanh", "sigmoid"
pub const DROPOUT_RATE: f32 = 0.1;             // Dropout rate (0.0 to 1.0)
```

- **Modify `HIDDEN_LAYERS`** to add more layers or change their size
- **Change `ACTIVATION`** to use a different activation function
- **Adjust `DROPOUT_RATE`** to control overfitting (higher = more regularization)

## 3. Training Parameters

Adjust training behavior in `config.rs`:

```rust
// Training parameters
pub const BATCH_SIZE: usize = 32;              // Number of samples per batch
pub const LEARNING_RATE: f32 = 0.001;          // Learning rate for optimizer
pub const EPOCHS: usize = 100;                 // Number of training cycles
pub const EARLY_STOPPING_PATIENCE: usize = 10; // Stop training if no improvement
```

- **Increase `BATCH_SIZE`** for faster training (requires more memory)
- **Decrease `LEARNING_RATE`** for more stable training
- **Increase `EPOCHS`** for longer training
- **Adjust `EARLY_STOPPING_PATIENCE`** to control when training stops

## 4. Optimizer and Regularization

Control optimization in `config.rs`:

```rust
// Advanced options
pub const OPTIMIZER: &str = "adam";            // "adam", "sgd", or "rmsprop"
pub const WEIGHT_DECAY: f32 = 0.0001;          // L2 regularization strength
pub const CLIP_GRADIENT: f32 = 1.0;            // Gradient clipping threshold
```

- **Change `OPTIMIZER`** to use a different optimization algorithm
- **Increase `WEIGHT_DECAY`** for stronger regularization
- **Adjust `CLIP_GRADIENT`** to prevent exploding gradients

## 5. Data Augmentation

Control data augmentation in `config.rs`:

```rust
// Data augmentation options
pub const USE_AUGMENTATION: bool = false;      // Whether to use data augmentation
pub const NOISE_LEVEL: f32 = 0.05;             // Level of Gaussian noise to add
pub const FEATURE_DROPOUT: f32 = 0.1;          // Randomly zero out features
```

## Advanced Customization

### Custom Data Loading

To support different data formats, modify the data loading functions in `data.rs`:

```rust
// Load a numerical dataset from a CSV file
pub fn load_csv_dataset(file_path: &str) -> Result<NumericalDataset> {
    // CUSTOMIZE HERE: Modify how CSV data is loaded
    
    // Implementation...
}
```

You can add support for:
- Different file formats (JSON, Excel, etc.)
- Different data sources (databases, APIs)
- Custom preprocessing steps

### Custom Model Architecture

For more advanced model changes, modify the `PredictorModel` struct and its implementation in `model.rs`:

```rust
// The neural network model for numerical prediction
pub struct PredictorModel<B: Backend> {
    // Input layer
    input_layer: Linear<B>,
    
    // Hidden layers
    hidden_layers: Vec<Linear<B>>,
    
    // Output layer
    output_layer: Linear<B>,
    
    // Dropout for regularization
    dropout: Dropout,
    
    // Activation function to use
    activation: String,
}
```

You can:
- Add different layer types (convolutional, recurrent)
- Implement more complex architectures
- Add skip connections or attention mechanisms

## Example: Adapting for Temperature Prediction

Here's how to adapt the template for temperature prediction based on weather data:

1. Update `config.rs`:
```rust
pub const FEATURE_COLUMNS: [&str; 5] = ["humidity", "pressure", "wind_speed", "cloud_cover", "season"];
pub const TARGET_COLUMN: &str = "temperature";
```

2. Prepare your CSV data:
```csv
humidity,pressure,wind_speed,cloud_cover,season,temperature
75,1013,10,80,3,22.5
65,1015,5,20,2,28.3
80,1010,15,100,4,15.7
```

3. Train the model:
```bash
cargo run -- train --data-file path/to/your/weather_data.csv
```

## Example: Creating a Deeper Network

To create a deeper network with more layers:

1. Update `config.rs`:
```rust
pub const HIDDEN_LAYERS: [usize; 4] = [128, 64, 32, 16];
```

2. You may need to adjust other parameters:
```rust
pub const LEARNING_RATE: f32 = 0.0005;  // Lower learning rate for deeper networks
pub const DROPOUT_RATE: f32 = 0.2;      // Higher dropout for more regularization
```

## Getting Help

If you need more help customizing this template, check out:
- [Burn Documentation](https://github.com/tracel-ai/burn)
- [Rust Machine Learning Resources](https://github.com/rust-ml)
