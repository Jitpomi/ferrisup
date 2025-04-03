# Custom Image Classifier with Rust

This template helps you build an image classification system using your own images with the Burn framework. You can train it to recognize cats vs. dogs, different types of flowers, or any other image categories you want.

## 🔍 What This Template Does

- Loads and processes images from your own folders
- Trains a convolutional neural network (CNN) to classify images
- Evaluates the model's accuracy
- Makes predictions on new images

## 🚀 Getting Started

### Preparing Your Data

1. Organize your images in folders, with each folder named after the category:
   ```
   data/
   ├── train/
   │   ├── cat/
   │   │   ├── cat1.jpg
   │   │   ├── cat2.jpg
   │   │   └── ...
   │   └── dog/
   │       ├── dog1.jpg
   │       ├── dog2.jpg
   │       └── ...
   └── test/
       ├── cat/
       │   └── ...
       └── dog/
           └── ...
   ```

2. Make sure your images are in common formats (JPG, PNG)

### Running the Example

1. Train the model:
   ```bash
   cargo run -- train --data-dir data/train --epochs 20
   ```

2. Evaluate the model:
   ```bash
   cargo run -- evaluate --model model.json --data-dir data/test
   ```

3. Classify a new image:
   ```bash
   cargo run -- predict --model model.json --image path/to/image.jpg
   ```

## 📊 Understanding the Code

### Model Architecture

This template uses a Convolutional Neural Network (CNN) with:

- **Convolutional layers**: Extract features from images
- **Batch normalization**: Stabilize training
- **Max pooling**: Reduce image dimensions while keeping important features
- **Dropout**: Prevent overfitting
- **Fully connected layers**: Make the final classification

### Key Components

- **data.rs**: Handles loading and processing images
- **model.rs**: Defines the neural network architecture
- **main.rs**: Contains the training and evaluation logic

## 🔧 Customizing for Your Needs

### Adjusting for Your Images

- Modify the number of classes in `model.rs` to match your categories
- Adjust the image size in `data.rs` if needed
- Add data augmentation for better performance

### Improving Performance

- Increase training time with more epochs
- Use a larger model by adding more layers or filters
- Try different learning rates

## 🛠️ Troubleshooting

### Common Issues

If you encounter issues with the Burn framework, try:

1. **Use an older version**: Try specifying `burn = "0.8.0"` in Cargo.toml
2. **Reduce batch size**: If you run out of memory, use a smaller batch size

### Image Loading Issues

- Make sure your images are in standard formats (JPG, PNG)
- Check that your folder structure matches the expected format
- Try resizing your images to a smaller size if they're very large

## 📚 Learning Resources

- [Burn Documentation](https://burn.dev/)
- [Convolutional Neural Networks Explained](https://cs231n.github.io/convolutional-networks/)
- [Image Classification Tutorial](https://www.tensorflow.org/tutorials/images/classification)

## 📝 License

This template is based on examples from the Burn framework and is available under the same license as Burn.
