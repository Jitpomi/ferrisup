# Getting Started with Image Recognition in Rust

This guide will walk you through using this template to build your first image recognition system in Rust.

## Prerequisites

- Rust installed on your system
- Basic understanding of programming concepts
- No prior machine learning knowledge required!

## Step 1: Understanding the Project Structure

This template includes:

- **src/main.rs**: The main application with training and evaluation commands
- **src/model.rs**: The neural network architecture
- **src/data.rs**: Code for loading and processing the dataset
- **Cargo.toml**: Project dependencies

## Step 2: Running Your First Training

1. Build and run the project:
   ```bash
   cargo run -- train --epochs 5
   ```

2. Watch the training progress:
   ```
   Epoch: 1/5, Train Loss: 2.3021, Train Accuracy: 0.1135, Valid Loss: 2.2891, Valid Accuracy: 0.1250
   Epoch: 2/5, Train Loss: 2.2891, Train Accuracy: 0.1250, Valid Loss: 2.2761, Valid Accuracy: 0.1365
   ...
   ```

3. After training completes, a model file (model.json) will be saved

## Step 3: Evaluating Your Model

Test how well your model performs:

```bash
cargo run -- evaluate --model model.json
```

You'll see results like:
```
Test Loss: 2.2631, Test Accuracy: 0.1480
```

> Note: With synthetic data, accuracy will be low. With real MNIST data, you should see 90%+ accuracy.

## Step 4: Using Your Own Images

To use your own images:

1. Prepare your images:
   - Convert to grayscale
   - Resize to 28x28 pixels
   - Normalize pixel values to [0, 1] range

2. Modify `data.rs` to load your images:
   ```rust
   // Replace the synthetic data generation with your own data loading
   pub fn load_my_dataset() -> impl Dataset<RawMnistItem> {
       // Your code to load images from a directory
       // ...
   }
   ```

## Step 5: Improving Your Model

If you want better performance:

1. Increase training time:
   ```bash
   cargo run -- train --epochs 20
   ```

2. Modify the model architecture in `model.rs`:
   - Add more convolutional layers
   - Change the number of filters
   - Adjust the fully connected layer size

3. Use real data instead of synthetic data

## Troubleshooting

### Common Issues

1. **Out of memory errors**:
   - Reduce batch size: `cargo run -- train --batch-size 16`
   - Simplify the model in `model.rs`

2. **Slow training**:
   - Consider enabling GPU support if available
   - Reduce the dataset size for testing

3. **Dependency issues**:
   - If you encounter issues with Burn, consider using Linfa as mentioned in the README

## Next Steps

Once you're comfortable with this example:

1. Try different neural network architectures
2. Experiment with your own image datasets
3. Implement data augmentation to improve model performance
4. Explore transfer learning with pre-trained models

## Need Help?

- Check the [Burn documentation](https://burn.dev/)
- Visit the [Rust Machine Learning community](https://github.com/rust-ml)
- Explore other examples in the Burn repository
