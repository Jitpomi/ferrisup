# Getting Started with Value Prediction in Rust

This guide will walk you through using this template to build your first value prediction system in Rust.

## Prerequisites

- Rust installed on your system
- Basic understanding of programming concepts
- No prior machine learning knowledge required!

## Step 1: Understanding the Project Structure

This template includes:

- **src/main.rs**: The main application with training and prediction commands
- **src/model.rs**: The neural network architecture
- **src/data.rs**: Code for loading and processing CSV data
- **Cargo.toml**: Project dependencies

## Step 2: Preparing Your Data

1. Create a data directory:
   ```bash
   mkdir -p data
   ```

2. Generate a sample dataset (or use your own CSV file):
   ```rust
   // Add this code to the end of main.rs to generate a sample dataset
   fn main() -> anyhow::Result<()> {
       // ... existing code ...
       
       // Create a sample dataset for testing
       data::create_sample_csv("data/housing.csv", 1000, 5)?;
       println!("Created sample dataset at data/housing.csv");
       
       Ok(())
   }
   ```

3. Your CSV should have numeric columns with the target value in the last column:
   ```
   feature_0,feature_1,feature_2,feature_3,feature_4,target
   0.5,0.2,-0.3,0.1,0.7,1.2
   -0.1,0.8,0.4,-0.5,0.3,0.9
   ...
   ```

## Step 3: Running Your First Training

1. Build and run the project:
   ```bash
   cargo run -- train --epochs 100 --data data/housing.csv
   ```

2. Watch the training progress:
   ```
   Epoch: 1/100, Train MSE: 0.8521, Valid MSE: 0.8234
   Epoch: 10/100, Train MSE: 0.4123, Valid MSE: 0.3987
   ...
   ```

3. After training completes, a model file (model.json) will be saved

## Step 4: Making Predictions

Make predictions on new data:

```bash
cargo run -- predict --model model.json --input data/test.csv
```

You'll see results like:
```
Input Features | Predicted Value
---------------|----------------
[0.5, 0.2, -0.3, 0.1, 0.7] | 1.1823
[-0.1, 0.8, 0.4, -0.5, 0.3] | 0.8976
```

## Step 5: Using Your Own Data

To use your own data:

1. Prepare your CSV file:
   - Make sure all columns contain numeric values
   - The last column should be the target value you want to predict
   - No missing values (or handle them before using this template)

2. Run training with your data:
   ```bash
   cargo run -- train --epochs 100 --data path/to/your/data.csv
   ```

## Step 6: Improving Your Model

If you want better predictions:

1. Increase training time:
   ```bash
   cargo run -- train --epochs 500
   ```

2. Modify the model architecture in `model.rs`:
   - Add more hidden layers
   - Change the number of neurons per layer
   - Try different activation functions

3. Preprocess your data:
   - Normalize features to have similar scales
   - Remove outliers
   - Engineer new features that might be relevant

## Troubleshooting

### Common Issues

1. **CSV parsing errors**:
   - Check your CSV file format
   - Make sure all values are numeric
   - Handle any missing values

2. **Poor prediction accuracy**:
   - Try normalizing your data
   - Increase the model complexity
   - Train for more epochs

3. **Dependency issues**:
   - If you encounter issues with Burn, consider using Linfa as mentioned in the README

## Next Steps

Once you're comfortable with this example:

1. Try different neural network architectures
2. Implement cross-validation to better evaluate your model
3. Add feature importance analysis
4. Explore other regression algorithms like Random Forest or Gradient Boosting

## Need Help?

- Check the [Burn documentation](https://burn.dev/)
- Visit the [Rust Machine Learning community](https://github.com/rust-ml)
- Explore other examples in the Burn repository
