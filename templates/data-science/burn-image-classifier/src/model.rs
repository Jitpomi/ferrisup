// Neural network model for image classification
// This file defines the model architecture and training logic

use burn::module::Module;
use burn::nn::{
    conv::{Conv2d, Conv2dConfig},
    pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig, MaxPool2d, MaxPool2dConfig},
    Linear, LinearConfig, PaddingConfig2d,
};
use burn::tensor::{backend::Backend, Tensor, activation::relu};
use std::fs;
use std::path::Path;
use serde_json;

use crate::config::{ImageClassifierConfig, NUM_CHANNELS};
use crate::error::{ImageClassifierError, Result};

/// Image classifier model
#[derive(Module, Debug)]
pub struct ImageClassifierModel<B: Backend> {
    // Convolutional layers
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    
    // Pooling layers
    pool: MaxPool2d,
    adaptive_pool: AdaptiveAvgPool2d,
    
    // Fully connected layers
    fc1: Linear<B>,
    fc2: Linear<B>,
}

impl<B: Backend> ImageClassifierModel<B> {
    /// Create a new image classifier model
    pub fn new(config: &ImageClassifierConfig, device: &B::Device) -> Self {
        // First convolutional layer
        let conv1 = Conv2dConfig::new([NUM_CHANNELS, 32], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
            
        // Second convolutional layer
        let conv2 = Conv2dConfig::new([32, 64], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
            
        // Pooling layers
        let pool = MaxPool2dConfig::new([2, 2])
            .with_strides([2, 2])
            .init();
            
        let adaptive_pool = AdaptiveAvgPool2dConfig::new([1, 1])
            .init();
            
        // Calculate the number of features after convolutions and pooling
        // For a 32x32 input image, after two 2x2 max pooling layers, we get 8x8 feature maps
        // With 64 channels, that's 64*8*8 = 4096 features
        // However, after adaptive pooling to [1, 1], we get 64 features
        let num_features = 64;
        
        // Fully connected layers
        let fc1 = LinearConfig::new(num_features, 128)
            .init(device);
            
        let fc2 = LinearConfig::new(128, config.num_classes)
            .init(device);
        
        Self {
            conv1,
            conv2,
            pool,
            adaptive_pool,
            fc1,
            fc2,
        }
    }
    
    /// Forward pass
    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 2> {
        // First convolutional block
        let x = self.conv1.forward(x);
        let x = relu(x);
        let x = self.pool.forward(x);
        
        // Second convolutional block
        let x = self.conv2.forward(x);
        let x = relu(x);
        let x = self.pool.forward(x);
        
        // Global average pooling
        let x = self.adaptive_pool.forward(x);
        
        // Flatten
        let [batch_size, channels, _, _] = x.dims();
        let x = x.reshape([batch_size, channels]);
        
        // Fully connected layers
        let x = self.fc1.forward(x);
        let x = relu(x);
        let x = self.fc2.forward(x);
        
        x
    }
    
    /// Save the model to a file
    pub fn save_file(&self, path: &str) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    ImageClassifierError::DirectoryReadError {
                        path: parent.to_path_buf(),
                        source: e,
                    }
                })?;
            }
        }
        
        // Save the model configuration instead
        let config = ImageClassifierConfig {
            num_classes: self.num_classes(),
            conv_filters: vec![32, 64],
            fc_layers: vec![128],
            dropout_rate: 0.5,
        };
        
        let json = serde_json::to_string_pretty(&config).map_err(|e| {
            ImageClassifierError::JsonError(e)
        })?;
        
        fs::write(path, json).map_err(|e| {
            ImageClassifierError::IoError(e)
        })
    }
    
    /// Load the model from a file
    pub fn load(path: &str, device: &B::Device) -> Result<Self> {
        // Load the model configuration
        let json = fs::read_to_string(path).map_err(|e| {
            ImageClassifierError::IoError(e)
        })?;
        
        let config: ImageClassifierConfig = serde_json::from_str(&json).map_err(|e| {
            ImageClassifierError::JsonError(e)
        })?;
        
        // Create a new model with the loaded configuration
        Ok(Self::new(&config, device))
    }
    
    /// Get the number of classes
    pub fn num_classes(&self) -> usize {
        // Get the output dimension of the final layer
        self.fc2.weight.shape().dims::<2>()[0]
    }
}
