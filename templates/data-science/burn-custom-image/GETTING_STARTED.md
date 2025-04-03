# Getting Started with Custom Image Classification in Rust

This guide will walk you through using this template to build your own image classifier in Rust.

## Prerequisites

- Rust installed on your system
- Basic understanding of programming concepts
- A set of images organized by category (e.g., cats vs. dogs, different flower types)
- No prior machine learning knowledge required!

## Step 1: Understanding the Project Structure

This template includes:

- **src/main.rs**: The main application with training and evaluation commands
- **src/model.rs**: The neural network architecture
- **src/data.rs**: Code for loading and processing images
- **Cargo.toml**: Project dependencies

## Step 2: Preparing Your Images

1. Create a data directory structure:
   ```
   data/
   ├── train/
   │   ├── category1/
   │   │   ├── image1.jpg
   │   │   ├── image2.jpg
   │   │   └── ...
   │   └── category2/
   │       ├── image1.jpg
   │       ├── image2.jpg
   │       └── ...
   └── test/
       ├── category1/
       │   └── ...
       └── category2/
           └── ...
   ```

2. Each category should be in its own folder, with the folder name being the category name
3. Use common image formats (JPG, PNG, BMP)
4. Aim for at least 50-100 images per category for decent results

## Step 3: Running Your First Training

1. Build and run the project:
   ```bash
   cargo run -- train --data-dir data/train --epochs 20
   ```

2. Watch the training progress:
   ```
   Epoch 1/20:
   [00:01:23] ████████████████████████████████████ 25/25 Train Loss: 0.6932, Train Accuracy: 0.5120
   [00:00:18] ████████████████████████████████████ 7/7 Valid Loss: 0.6931, Valid Accuracy: 0.5000
   ...
   ```

3. After training completes, a model file (model.json) and class names file (model.classes.json) will be saved

## Step 4: Evaluating Your Model

Test how well your model performs on new images:

```bash
cargo run -- evaluate --model model.json --data-dir data/test
```

You'll see results like:
```
Test Loss: 0.3421, Test Accuracy: 0.8750
```

## Step 5: Making Predictions

Classify a new image:

```bash
cargo run -- predict --model model.json --image path/to/image.jpg
```

You'll see results like:
```
Prediction: cat (0)
Confidence: 92.34%
Top predictions:
  1. cat - 92.34%
  2. dog - 7.66%
```

## Step 6: Improving Your Model

If you want better accuracy:

1. Add more training images
   - More diverse examples help the model generalize better
   - Try to have a balanced number of images for each category

2. Train for longer
   ```bash
   cargo run -- train --data-dir data/train --epochs 50
   ```

3. Adjust the model architecture in `model.rs`
   - Increase the number of filters in convolutional layers
   - Add more convolutional blocks
   - Change the dropout probability

## Troubleshooting

### Common Issues

1. **Out of memory errors**:
   - Reduce batch size: `cargo run -- train --data-dir data/train --batch-size 8`
   - Reduce image size: `cargo run -- train --data-dir data/train --image-size 128`

2. **Low accuracy**:
   - Add more training images
   - Make sure your categories are visually distinguishable
   - Try training for more epochs

3. **Slow training**:
   - Reduce image size
   - Use fewer images for initial testing
   - Consider enabling GPU support if available

## Next Steps

Once you're comfortable with this example:

1. Try adding data augmentation (rotating, flipping images) to improve robustness
2. Experiment with transfer learning using pre-trained models
3. Deploy your model to a web application or mobile device
4. Try classifying more complex image categories

## Need Help?

- Check the [Burn documentation](https://burn.dev/)
- Visit the [Rust Computer Vision community](https://github.com/rust-cv)
- Explore image classification tutorials online
