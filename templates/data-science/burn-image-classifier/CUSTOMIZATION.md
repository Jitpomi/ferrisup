# Customization Guide

This guide provides step-by-step instructions for adapting the Image Classifier template to your specific needs.

## Quick Customization Examples

### Example 1: Classifying Fruits (3 categories)

1. Edit `src/config.rs`:
   ```rust
   pub const NUM_CLASSES: usize = 3;
   pub const CLASS_NAMES: [&str; NUM_CLASSES] = ["apple", "banana", "orange"];
   ```

2. Organize your fruit images in folders:
   ```
   data/
   ├── train/
   │   ├── apple/
   │   ├── banana/
   │   └── orange/
   └── test/
       ├── apple/
       ├── banana/
       └── orange/
   ```

3. Train your model:
   ```bash
   cargo run -- train --data-dir data/train
   ```

### Example 2: Medical Image Analysis (higher resolution)

1. Edit `src/config.rs`:
   ```rust
   pub const IMAGE_SIZE: usize = 224;  // Higher resolution for medical images
   pub const NUM_CLASSES: usize = 2;
   pub const CLASS_NAMES: [&str; NUM_CLASSES] = ["normal", "abnormal"];
   
   // Slower learning rate for more precise training
   pub const LEARNING_RATE: f32 = 0.0005;
   pub const EPOCHS: usize = 50;  // More epochs for better learning
   ```

2. Train with your medical images:
   ```bash
   cargo run -- train --data-dir path/to/medical/images
   ```

### Example 3: Satellite Imagery (grayscale)

1. Edit `src/config.rs`:
   ```rust
   pub const NUM_CHANNELS: usize = 1;  // Grayscale images
   pub const IMAGE_SIZE: usize = 128;  // Larger size for satellite imagery
   pub const NUM_CLASSES: usize = 4;
   pub const CLASS_NAMES: [&str; NUM_CLASSES] = ["urban", "forest", "water", "agriculture"];
   ```

2. Edit `src/data.rs` to handle grayscale images:
   ```rust
   // Find the image_to_tensor function and modify it:
   fn image_to_tensor(img: &DynamicImage) -> Vec<f32> {
       // Convert to grayscale
       let img_gray = img.to_luma8();
       
       // Create a vector to hold the tensor data
       let mut tensor = vec![0.0; NUM_CHANNELS * IMAGE_SIZE * IMAGE_SIZE];
       
       // Fill the tensor with normalized pixel values
       for y in 0..IMAGE_SIZE {
           for x in 0..IMAGE_SIZE {
               let pixel = img_gray.get_pixel(x as u32, y as u32);
               
               // Normalize pixel value to the range [0, 1]
               let value = pixel[0] as f32 / 255.0;
               
               // Store in CHW format (channels, height, width)
               let idx = 0 * IMAGE_SIZE * IMAGE_SIZE + y * IMAGE_SIZE + x;
               tensor[idx] = value;
           }
       }
       
       tensor
   }
   ```

## Step-by-Step Customization Guide

### 1. Changing the Number of Classes

1. Edit `src/config.rs`:
   ```rust
   pub const NUM_CLASSES: usize = YOUR_NUMBER_OF_CLASSES;
   pub const CLASS_NAMES: [&str; NUM_CLASSES] = ["class1", "class2", ...];
   ```

2. No other changes are needed - the model will automatically adjust to output the correct number of classes.

### 2. Changing Image Size

1. Edit `src/config.rs`:
   ```rust
   pub const IMAGE_SIZE: usize = YOUR_DESIRED_SIZE;  // e.g., 64, 128, 224, etc.
   ```

2. Consider the tradeoffs:
   - Larger sizes capture more detail but require more memory and computation
   - Smaller sizes are faster but may miss important details

### 3. Modifying the Model Architecture

To create a deeper or wider network:

1. Edit `src/config.rs`:
   ```rust
   pub const CONV_FILTERS: [usize; 4] = [32, 64, 128, 256];  // Added one more layer
   pub const FC_LAYERS: [usize; 3] = [1024, 512, 128];  // Added one more layer
   ```

2. Edit `src/model.rs` to add the new layers:
   ```rust
   // Add a new convolutional layer
   let conv4 = Conv2dConfig::new([CONV_FILTERS[2], CONV_FILTERS[3]], [3, 3])
       .with_padding([1, 1])
       .init();
   let batch_norm4 = BatchNormConfig::new(CONV_FILTERS[3]).init();
   
   // Add to struct
   conv4: Conv2d<B>,
   batch_norm4: BatchNorm<B, 2>,
   
   // Update the forward method to use the new layer
   let x = self.conv4.forward(x);
   let x = self.batch_norm4.forward(x);
   let x = x.relu();
   ```

### 4. Customizing Data Augmentation

1. Edit `src/config.rs` to enable/disable existing augmentations:
   ```rust
   pub const USE_AUGMENTATION: bool = true;
   pub const RANDOM_FLIP: bool = true;
   pub const RANDOM_CROP: bool = true;
   pub const RANDOM_BRIGHTNESS: bool = true;
   ```

2. Add a new augmentation in `src/data.rs`:
   ```rust
   // In the apply_augmentation function, add:
   
   // Random rotation
   if rng.gen_bool(0.5) {
       let angle = rng.gen_range(-15.0..15.0);
       result = result.rotate(angle);
   }
   ```

3. Add the corresponding config option in `src/config.rs`:
   ```rust
   pub const RANDOM_ROTATION: bool = true;
   ```

### 5. Using Transfer Learning

To use a pre-trained model as a starting point:

1. Download a pre-trained model:
   ```bash
   cargo run -- download-pretrained
   ```

2. Fine-tune on your data:
   ```bash
   cargo run -- finetune --model pretrained.json --data-dir your/data/dir
   ```

### 6. Optimizing for Performance

1. For faster training, edit `src/config.rs`:
   ```rust
   pub const IMAGE_SIZE: usize = 64;  // Smaller images
   pub const BATCH_SIZE: usize = 128;  // Larger batch size if memory allows
   ```

2. For better accuracy, edit `src/config.rs`:
   ```rust
   pub const EPOCHS: usize = 50;  // More training epochs
   pub const LEARNING_RATE: f32 = 0.0005;  // Smaller learning rate
   ```

## Troubleshooting Common Issues

### Model Not Learning (Low Accuracy)

1. Try a lower learning rate:
   ```rust
   pub const LEARNING_RATE: f32 = 0.0001;
   ```

2. Train for more epochs:
   ```rust
   pub const EPOCHS: usize = 50;
   ```

3. Add more data augmentation to increase effective dataset size.

### Out of Memory Errors

1. Reduce batch size:
   ```rust
   pub const BATCH_SIZE: usize = 16;  // Smaller batch size
   ```

2. Reduce image size:
   ```rust
   pub const IMAGE_SIZE: usize = 32;  // Smaller images
   ```

3. Simplify the model architecture by reducing the number of filters or layers.

### Overfitting (Good Training Accuracy, Poor Validation Accuracy)

1. Increase dropout rate:
   ```rust
   pub const DROPOUT_RATE: f32 = 0.7;  // More aggressive dropout
   ```

2. Enable more data augmentation:
   ```rust
   pub const USE_AUGMENTATION: bool = true;
   pub const RANDOM_FLIP: bool = true;
   pub const RANDOM_CROP: bool = true;
   pub const RANDOM_BRIGHTNESS: bool = true;
   ```

3. Reduce model complexity or add L2 regularization.

## Advanced Customization Examples

### Creating a Multi-Label Classifier

For classifying images with multiple labels (e.g., an image can be both "sunny" and "beach"):

1. Edit `src/model.rs` to change the final activation function:
   ```rust
   // In the forward method, change the final layer to use sigmoid instead of softmax
   // (which is implicitly used in cross_entropy_with_logits)
   let output = self.fc3.forward(x);
   output.sigmoid()  // Use sigmoid for multi-label
   ```

2. Edit `src/main.rs` to change the loss function:
   ```rust
   // Change from cross_entropy_with_logits to binary_cross_entropy
   let loss = output.binary_cross_entropy(&batch.labels);
   ```

### Adding Attention Mechanisms

To add attention mechanisms for better feature focus:

1. Add the necessary imports in `src/model.rs`:
   ```rust
   use burn::nn::attention::{MultiHeadAttention, MultiHeadAttentionConfig};
   ```

2. Add attention layers to your model structure and forward pass.

## Exporting for Production

To export your model for production use:

1. Train your model as usual:
   ```bash
   cargo run -- train --data-dir your/data/dir
   ```

2. Export to ONNX format for deployment:
   ```bash
   cargo run -- export --model model.json --output model.onnx
   ```

3. Use the exported model with ONNX Runtime in your production application.

## Customizing the Image Classifier Template

This guide provides detailed instructions for customizing the image classifier template to fit your specific needs.

## Table of Contents

1. [Quick Customization](#quick-customization)
2. [Model Architecture](#model-architecture)
3. [Data Processing](#data-processing)
4. [Training Process](#training-process)
5. [Backend Selection](#backend-selection)
6. [Advanced Customization](#advanced-customization)

## Quick Customization

The fastest way to customize the template is by modifying the configuration parameters in `src/config.rs`:

```rust
// Image dimensions - adjust based on your dataset
pub const IMAGE_SIZE: usize = 224;
pub const NUM_CHANNELS: usize = 3;

// Model architecture - change these to adjust model complexity
pub const CONV_FILTERS: [usize; 3] = [32, 64, 128];
pub const FC_FEATURES: [usize; 2] = [512, 128];
pub const DROPOUT_RATE: f32 = 0.5;

// Training parameters - tune these for your specific task
pub const BATCH_SIZE: usize = 32;
pub const LEARNING_RATE: f32 = 0.001;
pub const EPOCHS: usize = 10;

// Data augmentation - enable/disable as needed
pub const USE_AUGMENTATION: bool = true;
pub const RANDOM_FLIP: bool = true;
pub const RANDOM_CROP: bool = true;
pub const RANDOM_BRIGHTNESS: bool = true;

// Default paths
pub const DEFAULT_DATA_DIR: &str = "./data";
pub const DEFAULT_MODEL_FILE: &str = "./model.bin";
```

## Model Architecture

To modify the neural network architecture, edit `src/model.rs`. The model is defined in the `ImageClassifierModel` struct and built in the `new` method.

### Changing the Convolutional Layers

Look for the `// Build convolutional layers` section:

```rust
// Build convolutional layers
let mut layers = vec![];
let mut in_channels = NUM_CHANNELS;

for &filters in CONV_FILTERS.iter() {
    // Add a convolutional block
    layers.push(Conv2d::new(
        in_channels, 
        filters, 
        3, // kernel size
        ConvConfig::default()
    ));
    layers.push(ReLU::new());
    layers.push(BatchNorm2d::new(filters));
    layers.push(MaxPool2d::new(2)); // 2x2 pooling
    
    in_channels = filters;
}
```

You can:
- Change the number of filters in each layer by modifying `CONV_FILTERS` in `config.rs`
- Adjust kernel sizes (currently 3x3)
- Change the pooling strategy or size
- Use different activation functions (currently ReLU)

### Changing the Fully Connected Layers

Look for the `// Build fully connected layers` section:

```rust
// Build fully connected layers
let mut fc_layers = vec![];
let mut in_features = flattened_size;

for &features in FC_FEATURES.iter() {
    fc_layers.push(Linear::new(in_features, features));
    fc_layers.push(ReLU::new());
    fc_layers.push(Dropout::new(DROPOUT_RATE));
    
    in_features = features;
}

// Output layer
fc_layers.push(Linear::new(in_features, num_classes));
```

You can:
- Change the number and size of hidden layers by modifying `FC_FEATURES` in `config.rs`
- Adjust the dropout rate by changing `DROPOUT_RATE`
- Use different activation functions

## Data Processing

To customize how images are loaded and processed, edit `src/data.rs`.

### Customizing Data Augmentation

Look for the `apply_augmentation` function:

```rust
// Apply data augmentation to an image
pub fn apply_augmentation(img: DynamicImage) -> DynamicImage {
    let mut rng = thread_rng();
    let mut result = img;
    
    // Random horizontal flip
    if RANDOM_FLIP && rng.gen::<bool>() {
        result = result.fliph();
    }
    
    // Random crop and resize
    if RANDOM_CROP && rng.gen::<bool>() {
        // Get dimensions
        let (width, height) = result.dimensions();
        let crop_factor = 0.8 + rng.gen::<f32>() * 0.2; // Crop between 80-100%
        
        let crop_width = (width as f32 * crop_factor) as u32;
        let crop_height = (height as f32 * crop_factor) as u32;
        
        let x = rng.gen_range(0..width - crop_width);
        let y = rng.gen_range(0..height - crop_height);
        
        // Crop and resize back to original dimensions
        result = result.crop_imm(x, y, crop_width, crop_height)
                       .resize(width, height, imageops::FilterType::Lanczos3);
    }
    
    // Random brightness adjustment
    if RANDOM_BRIGHTNESS && rng.gen::<bool>() {
        let factor = 0.8 + rng.gen::<f32>() * 0.4; // 0.8-1.2 brightness factor
        result = imageops::brighten(&result, (factor * 100.0 - 100.0) as i32);
    }
    
    result
}
```

You can add more augmentation techniques such as:
- Rotation
- Color jitter
- Contrast adjustment
- Gaussian noise

### Customizing Image Preprocessing

Look for the `image_to_tensor` function:

```rust
// Convert an image to a normalized tensor
pub fn image_to_tensor(img: &DynamicImage) -> Vec<f32> {
    // Resize image if needed
    let img = img.resize_exact(
        IMAGE_SIZE as u32, 
        IMAGE_SIZE as u32, 
        imageops::FilterType::Lanczos3
    );
    
    // Convert to RGB if not already
    let img_rgb = img.to_rgb8();
    
    // Create tensor data
    let mut tensor_data = Vec::with_capacity(NUM_CHANNELS * IMAGE_SIZE * IMAGE_SIZE);
    
    // Normalize pixel values to [-1, 1]
    for pixel in img_rgb.pixels() {
        // Red channel
        tensor_data.push(pixel[0] as f32 / 127.5 - 1.0);
        // Green channel
        tensor_data.push(pixel[1] as f32 / 127.5 - 1.0);
        // Blue channel
        tensor_data.push(pixel[2] as f32 / 127.5 - 1.0);
    }
    
    tensor_data
}
```

You can customize:
- Normalization strategy (currently normalizes to [-1, 1])
- Channel ordering
- Resizing method

## Training Process

To customize the training process, edit the `examples/image_classifier.rs` file.

### Changing the Optimizer

Look for the optimizer initialization:

```rust
// Initialize optimizer
let optimizer = Adam::new(LEARNING_RATE);
```

You can:
- Use a different optimizer (SGD, AdamW, etc.)
- Add weight decay
- Implement a learning rate scheduler

### Customizing the Training Loop

The training loop is in the `train` function:

```rust
// Training loop
println!("🚀 Starting training for {} epochs", epochs);

for epoch in 0..epochs {
    // Training phase
    let mut train_loss = 0.0;
    let mut train_accuracy = 0.0;
    let mut train_batches = 0;
    
    for batch_idx in 0..(train_dataset.len() / batch_size) {
        // Get batch of items
        let items = (0..batch_size)
            .map(|i| train_dataset.get(batch_idx * batch_size + i).unwrap())
            .collect();
        
        // Process batch
        let batch = train_batcher.batch(items);
        let output = train_step.step(&batch);
        
        // Update metrics
        train_loss += output.loss;
        train_accuracy += output.output.accuracy;
        train_batches += 1;
        
        // Update progress bar
        progress_bar.inc(1);
        progress_bar.set_message(format!("Epoch {}/{}", epoch + 1, epochs));
    }
    
    // Validation phase
    // ...
}
```

You can customize:
- The metrics being tracked
- Add early stopping
- Implement learning rate scheduling
- Add model checkpointing

## Backend Selection

The template supports multiple backends through Burn's backend system. To use a different backend:

1. Make sure the corresponding feature is enabled in `Cargo.toml`
2. Run the example with the appropriate feature flag:

```bash
# CPU backend (default)
cargo run --example image_classifier --features ndarray

# LibTorch CPU backend
cargo run --example image_classifier --features tch-cpu

# LibTorch GPU backend
cargo run --example image_classifier --features tch-gpu

# WebGPU backend
cargo run --example image_classifier --features wgpu
```

## Advanced Customization

### Creating a Custom Dataset Loader

If your data is organized differently, you can create a custom dataset loader:

1. Create a new function in `src/data.rs` similar to `load_image_dataset`
2. Implement your custom loading logic
3. Update the example to use your new loader

### Adding Transfer Learning

To implement transfer learning with a pre-trained model:

1. Add a function to load pre-trained weights in `src/model.rs`
2. Modify the model architecture to match the pre-trained model
3. Freeze early layers and only train later layers

### Implementing Custom Loss Functions

To use a different loss function:

1. Implement the custom loss in the `forward` method of `ImageClassifierModel`
2. Update the `TrainingStepHandler` to use your custom loss

### Adding Quantization Support

To add model quantization for faster inference:

1. Add quantization logic in the `save_file` method of `ImageClassifierModel`
2. Implement a separate loading function for quantized models
3. Update the inference code to handle quantized models
