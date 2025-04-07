// Neural Network Model for Image Recognition
// This file defines the structure of our Convolutional Neural Network (CNN)
// that will learn to recognize handwritten digits

use burn::module::Module;
use burn::nn::{
    conv::{Conv2d, Conv2dConfig},
    pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig},
    BatchNorm, BatchNormConfig, Linear, LinearConfig, ReLU,
};
use burn::tensor::{backend::Backend, Tensor};
use burn::train::ClassificationOutput;
use burn::config::Config;

// Configuration for our MNIST model
// This defines the structure and parameters of our neural network
#[derive(Config)]
pub struct MnistConfig {
    // First convolutional layer parameters
    pub conv1_channels: usize,
    // Second convolutional layer parameters
    pub conv2_channels: usize,
    // Number of neurons in the fully connected layer
    pub fc1_features: usize,
    // Number of output classes (digits 0-9)
    pub num_classes: usize,
}

impl MnistConfig {
    // Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            // 32 filters in the first convolutional layer
            conv1_channels: 32,
            // 64 filters in the second convolutional layer
            conv2_channels: 64,
            // 1024 neurons in the fully connected layer
            fc1_features: 1024,
            // 10 output classes (digits 0-9)
            num_classes: 10,
        }
    }
}

// Convolutional Block - a reusable component of our neural network
// Each block contains a convolutional layer, batch normalization, and ReLU activation
#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    // Convolutional layer - extracts features from the input
    pub conv: Conv2d<B>,
    // Batch normalization - stabilizes and accelerates training
    pub batch_norm: BatchNorm<B, 2>,
    // ReLU activation - introduces non-linearity
    pub relu: ReLU,
}

impl<B: Backend> ConvBlock<B> {
    // Create a new convolutional block
    pub fn new(in_channels: usize, out_channels: usize) -> Self {
        // Configure the convolutional layer
        let conv_config = Conv2dConfig::new(in_channels, out_channels)
            .with_kernel_size(3) // 3x3 filter
            .with_stride(1)      // Move 1 pixel at a time
            .with_padding(1);    // Add 1 pixel padding around the edges
        
        // Configure the batch normalization layer
        let batch_norm_config = BatchNormConfig::new(out_channels);
        
        Self {
            // Create the layers based on the configurations
            conv: Conv2d::new(conv_config),
            batch_norm: BatchNorm::new(batch_norm_config),
            relu: ReLU::new(),
        }
    }
    
    // Forward pass - process the input through all layers in the block
    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        // Apply convolution, then batch normalization, then ReLU activation
        let x = self.conv.forward(input);
        let x = self.batch_norm.forward(x);
        let x = self.relu.forward(x);
        x
    }
}

// MNIST Model - the complete neural network for digit recognition
#[derive(Module, Debug)]
pub struct MnistModel<B: Backend> {
    // First convolutional block
    pub conv_block1: ConvBlock<B>,
    // Second convolutional block
    pub conv_block2: ConvBlock<B>,
    // Adaptive average pooling - resizes feature maps to a fixed size
    pub adaptive_pool: AdaptiveAvgPool2d,
    // First fully connected layer
    pub fc1: Linear<B>,
    // Output layer - produces the final classification
    pub fc2: Linear<B>,
}

impl<B: Backend> MnistModel<B> {
    // Create a new MNIST model based on the provided configuration
    pub fn new(config: &MnistConfig) -> Self {
        // Create the convolutional blocks
        let conv_block1 = ConvBlock::new(1, config.conv1_channels);
        let conv_block2 = ConvBlock::new(config.conv1_channels, config.conv2_channels);
        
        // Configure the adaptive pooling layer
        let adaptive_pool = AdaptiveAvgPool2d::new(AdaptiveAvgPool2dConfig::new([1, 1]));
        
        // Configure the fully connected layers
        let fc1_config = LinearConfig::new(config.conv2_channels, config.fc1_features);
        let fc2_config = LinearConfig::new(config.fc1_features, config.num_classes);
        
        Self {
            // Create the layers based on the configurations
            conv_block1,
            conv_block2,
            adaptive_pool,
            fc1: Linear::new(fc1_config),
            fc2: Linear::new(fc2_config),
        }
    }
    
    // Forward pass - process the input through the entire network
    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 2> {
        // Apply the first convolutional block
        let x = self.conv_block1.forward(input);
        // Apply the second convolutional block
        let x = self.conv_block2.forward(x);
        // Apply adaptive pooling to get a fixed-size output
        let x = self.adaptive_pool.forward(x);
        // Flatten the output for the fully connected layers
        let [batch_size, channels, _, _] = x.dims();
        let x = x.reshape([batch_size, channels]);
        // Apply the first fully connected layer with ReLU activation
        let x = self.fc1.forward(x).relu();
        // Apply the output layer
        let x = self.fc2.forward(x);
        x
    }
    
    // Forward pass for classification - includes loss calculation
    pub fn forward_classification(
        &self,
        images: Tensor<B, 4>,
        targets: Tensor<B, 1, usize>,
    ) -> ClassificationOutput {
        // Get the model's predictions
        let output = self.forward(images);
        
        // Calculate the loss (how far off our predictions are)
        let loss = output.cross_entropy_loss(targets);
        
        // Calculate the accuracy (percentage of correct predictions)
        let accuracy = output.accuracy(targets);
        
        // Return both loss and accuracy
        ClassificationOutput::new(loss, accuracy)
    }
}
