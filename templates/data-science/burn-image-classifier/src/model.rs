// Image Classifier Model Architecture
// This file defines the neural network model for image classification

use burn::module::Module;
use burn::nn::{
    conv::{Conv2d, Conv2dConfig},
    pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig, MaxPool2d, MaxPool2dConfig},
    BatchNorm, BatchNormConfig, Linear, LinearConfig, ReLU, Dropout, DropoutConfig,
};
use burn::tensor::{backend::Backend, Tensor};
use burn::config::Config;

// Import our configuration parameters
use crate::config::{CONV_FILTERS, FC_LAYERS, DROPOUT_RATE, NUM_CLASSES, IMAGE_SIZE, NUM_CHANNELS};

// Model configuration
#[derive(Config)]
pub struct ImageClassifierConfig {
    // CUSTOMIZE HERE: You can add or modify configuration parameters
    pub num_classes: usize,
}

impl ImageClassifierConfig {
    pub fn new(num_classes: usize) -> Self {
        Self { num_classes }
    }
    
    // Default configuration using values from config.rs
    pub fn default() -> Self {
        Self { num_classes: NUM_CLASSES }
    }
}

// The CNN model for image classification
pub struct ImageClassifierModel<B: Backend> {
    // Convolutional layers
    conv1: Conv2d<B>,
    batch_norm1: BatchNorm<B, 2>,
    conv2: Conv2d<B>,
    batch_norm2: BatchNorm<B, 2>,
    conv3: Conv2d<B>,
    batch_norm3: BatchNorm<B, 2>,
    
    // Pooling layers
    pool: MaxPool2d,
    adaptive_pool: AdaptiveAvgPool2d,
    
    // Fully connected layers
    fc1: Linear<B>,
    fc2: Linear<B>,
    fc3: Linear<B>,
    
    // Dropout for regularization
    dropout: Dropout,
}

impl<B: Backend> ImageClassifierModel<B> {
    pub fn new(config: &ImageClassifierConfig) -> Self {
        // CUSTOMIZE HERE: Modify the model architecture
        
        // First convolutional block
        let conv1 = Conv2dConfig::new([NUM_CHANNELS, CONV_FILTERS[0]], [3, 3])
            .with_padding([1, 1])
            .init();
        let batch_norm1 = BatchNormConfig::new(CONV_FILTERS[0]).init();
        
        // Second convolutional block
        let conv2 = Conv2dConfig::new([CONV_FILTERS[0], CONV_FILTERS[1]], [3, 3])
            .with_padding([1, 1])
            .init();
        let batch_norm2 = BatchNormConfig::new(CONV_FILTERS[1]).init();
        
        // Third convolutional block
        let conv3 = Conv2dConfig::new([CONV_FILTERS[1], CONV_FILTERS[2]], [3, 3])
            .with_padding([1, 1])
            .init();
        let batch_norm3 = BatchNormConfig::new(CONV_FILTERS[2]).init();
        
        // Pooling layers
        let pool = MaxPool2dConfig::new([2, 2]).init();
        let adaptive_pool = AdaptiveAvgPool2dConfig::new([1, 1]).init();
        
        // Calculate the flattened size after convolutions and pooling
        // For a 32x32 input, after 3 conv+pool layers, we'll have a 4x4 feature map
        // with CONV_FILTERS[2] channels
        let flattened_size = CONV_FILTERS[2]; // After adaptive pooling, this is 1x1xCONV_FILTERS[2]
        
        // Fully connected layers
        let fc1 = LinearConfig::new(flattened_size, FC_LAYERS[0]).init();
        let fc2 = LinearConfig::new(FC_LAYERS[0], FC_LAYERS[1]).init();
        let fc3 = LinearConfig::new(FC_LAYERS[1], config.num_classes).init();
        
        // Dropout for regularization
        let dropout = DropoutConfig::new(DROPOUT_RATE).init();
        
        Self {
            conv1,
            batch_norm1,
            conv2,
            batch_norm2,
            conv3,
            batch_norm3,
            pool,
            adaptive_pool,
            fc1,
            fc2,
            fc3,
            dropout,
        }
    }
}

// Implement the Module trait for our model
impl<B: Backend> Module<Tensor<B, 4>> for ImageClassifierModel<B> {
    type Output = Tensor<B, 2>;
    
    fn forward(&self, input: Tensor<B, 4>) -> Self::Output {
        // CUSTOMIZE HERE: Modify the forward pass logic
        
        // First convolutional block
        let x = self.conv1.forward(input);
        let x = self.batch_norm1.forward(x);
        let x = x.relu();
        let x = self.pool.forward(x);
        
        // Second convolutional block
        let x = self.conv2.forward(x);
        let x = self.batch_norm2.forward(x);
        let x = x.relu();
        let x = self.pool.forward(x);
        
        // Third convolutional block
        let x = self.conv3.forward(x);
        let x = self.batch_norm3.forward(x);
        let x = x.relu();
        
        // Global average pooling
        let x = self.adaptive_pool.forward(x);
        
        // Flatten the tensor for the fully connected layers
        // Shape goes from [batch_size, channels, 1, 1] to [batch_size, channels]
        let [batch_size, channels, _, _] = x.shape().dims;
        let x = x.reshape([batch_size, channels]);
        
        // Fully connected layers
        let x = self.fc1.forward(x);
        let x = x.relu();
        let x = self.dropout.forward(x);
        
        let x = self.fc2.forward(x);
        let x = x.relu();
        let x = self.dropout.forward(x);
        
        // Final classification layer
        self.fc3.forward(x)
    }
}

// Helper function to create a model with default configuration
pub fn create_default_model<B: Backend>() -> ImageClassifierModel<B> {
    let config = ImageClassifierConfig::default();
    ImageClassifierModel::new(&config)
}
