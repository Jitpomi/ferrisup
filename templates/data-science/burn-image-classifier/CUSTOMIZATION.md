# Customizing the Image Classifier

This guide provides detailed instructions on how to customize the image classifier template for your specific needs.

## Table of Contents

1. [Changing the Model Architecture](#changing-the-model-architecture)
2. [Adapting to Your Dataset](#adapting-to-your-dataset)
3. [Tuning Training Parameters](#tuning-training-parameters)
4. [Customizing Data Augmentation](#customizing-data-augmentation)
5. [Using Different Backends](#using-different-backends)
6. [Adding Custom Metrics](#adding-custom-metrics)
7. [Extending the Visualization](#extending-the-visualization)

## Changing the Model Architecture

The model architecture is defined in `src/model.rs` and configured in `src/config.rs`. To modify the architecture:

1. Edit the `CONV_FILTERS` and `FC_LAYERS` arrays in `src/config.rs` to change the number and size of layers.
2. Adjust the `DROPOUT_RATE` to control regularization.
3. For more significant changes, modify the `ImageClassifierModel` struct in `src/model.rs`.

Example of a deeper network:

```rust
// In src/config.rs
pub const CONV_FILTERS: [usize; 4] = [32, 64, 128, 256];  // Added another layer
pub const FC_LAYERS: [usize; 3] = [1024, 512, 256];       // Added another layer
```

## Adapting to Your Dataset

To adapt the classifier to your specific dataset:

1. Update `NUM_CLASSES` in `src/config.rs` to match the number of categories in your dataset.
2. Change `CLASS_NAMES` to match your category names.
3. Adjust `IMAGE_SIZE` and `NUM_CHANNELS` if your images have different dimensions or color channels.

Example for a grayscale medical image dataset with 5 classes:

```rust
// In src/config.rs
pub const IMAGE_SIZE: usize = 64;         // Larger images
pub const NUM_CHANNELS: usize = 1;        // Grayscale
pub const NUM_CLASSES: usize = 5;         // 5 categories

pub const CLASS_NAMES: [&str; NUM_CLASSES] = [
    "normal", "benign", "malignant", "suspicious", "artifact"
];
```

## Tuning Training Parameters

Training parameters can be adjusted in `src/config.rs`:

1. Increase `BATCH_SIZE` for faster training (if you have enough memory).
2. Adjust `LEARNING_RATE` to control the speed of learning.
3. Increase `EPOCHS` for longer training.

For more advanced optimization:

```rust
// In src/config.rs
pub const BATCH_SIZE: usize = 128;        // Larger batch size
pub const LEARNING_RATE: f32 = 0.0005;    // Smaller learning rate
pub const EPOCHS: usize = 50;             // More epochs
```

You can also modify the optimizer in `src/main.rs`:

```rust
// In src/main.rs, inside the train function
let optimizer = AdamConfig::new()
    .with_learning_rate(learning_rate)
    .with_weight_decay(0.0001)  // Add weight decay
    .init();
```

## Customizing Data Augmentation

Data augmentation options can be configured in `src/config.rs`:

1. Set `USE_AUGMENTATION` to `true` or `false` to enable/disable all augmentation.
2. Toggle specific augmentations with `RANDOM_FLIP`, `RANDOM_CROP`, and `RANDOM_BRIGHTNESS`.

To add new augmentation techniques, modify the `apply_augmentation` function in `src/data.rs`:

```rust
// In src/data.rs
fn apply_augmentation(img: DynamicImage) -> DynamicImage {
    // Existing code...
    
    // Add rotation augmentation
    if rng.gen_bool(0.5) {
        let angle = rng.gen_range(-15.0..15.0);
        img = img.rotate(angle);
    }
    
    img
}
```

## Using Different Backends

By default, the template uses the `burn-ndarray` backend. To use a different backend:

1. Uncomment the appropriate backend in `Cargo.toml`.
2. Modify the type alias in `src/main.rs`:

```rust
// For PyTorch backend
type B = burn_tch::TchBackend<f32>;

// For WebGPU backend
type B = burn_wgpu::WgpuBackend<f32>;
```

3. Update the device creation if needed:

```rust
// For GPU with PyTorch
let device = burn_tch::TchDevice::Cuda(0); // Use first CUDA device

// For WebGPU
let device = burn_wgpu::WgpuDevice::default();
```

## Adding Custom Metrics

To add custom evaluation metrics:

1. Define your metric in a new file, e.g., `src/metrics.rs`.
2. Implement the `Metric` trait from Burn.
3. Use your metric in the training and evaluation loops.

Example of a custom F1 score metric:

```rust
// In src/metrics.rs
use burn::train::metric::Metric;

pub struct F1ScoreMetric {
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
}

impl F1ScoreMetric {
    pub fn new() -> Self {
        Self {
            true_positives: 0,
            false_positives: 0,
            false_negatives: 0,
        }
    }
}

impl Metric for F1ScoreMetric {
    type Input = ClassificationOutput<B>;

    fn update(&mut self, output: Self::Input, _batch_size: usize) {
        // Implementation details...
    }

    fn value(&self) -> f32 {
        let precision = if self.true_positives + self.false_positives > 0 {
            self.true_positives as f32 / (self.true_positives + self.false_positives) as f32
        } else {
            0.0
        };
        
        let recall = if self.true_positives + self.false_negatives > 0 {
            self.true_positives as f32 / (self.true_positives + self.false_negatives) as f32
        } else {
            0.0
        };
        
        if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        }
    }

    fn reset(&mut self) {
        self.true_positives = 0;
        self.false_positives = 0;
        self.false_negatives = 0;
    }
}
```

## Extending the Visualization

To add new visualizations:

1. Add new functions to `src/visualization.rs`.
2. Call these functions from appropriate places in `src/main.rs`.

Example of adding a learning rate visualization:

```rust
// In src/visualization.rs
pub fn plot_learning_rate(learning_rates: &[f32], output_path: &str) -> Result<()> {
    // Implementation details...
}
```

Then use it in your training loop:

```rust
// In src/main.rs
let mut learning_rates = Vec::new();

for epoch in 1..=epochs {
    // Record learning rate
    learning_rates.push(current_learning_rate);
    
    // Existing training code...
    
    // Visualize learning rate
    visualization::plot_learning_rate(&learning_rates, "learning_rate.png")?;
}
```
