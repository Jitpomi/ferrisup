// Neural Network Model for Image Classification
// This file defines the structure of our Convolutional Neural Network (CNN)
// that will learn to classify your custom images

use burn::module::Module;
use burn::nn::{
    conv::{Conv2d, Conv2dConfig},
    pool::{MaxPool2d, MaxPool2dConfig},
    BatchNorm, BatchNormConfig, Dropout, DropoutConfig, Linear, LinearConfig, ReLU,
};
use burn::tensor::{backend::Backend, Tensor};
use burn::train::ClassificationOutput;
use burn::config::Config;

// Configuration for our image classifier model
// This defines the structure and parameters of our neural network
#[derive(Config)]
pub struct ImageClassifierConfig {
    // First convolutional layer parameters
    pub conv1_channels: usize,
    // Second convolutional layer parameters
    pub conv2_channels: usize,
    // Third convolutional layer parameters
    pub conv3_channels: usize,
    // Number of neurons in the fully connected layer
    pub fc1_features: usize,
    // Number of output classes (categories)
    pub num_classes: usize,
    // Dropout probability (to prevent overfitting)
    pub dropout_prob: f64,
}

impl ImageClassifierConfig {
    // Create a new configuration with default values
    pub fn new(num_classes: usize) -> Self {
        Self {
            // 32 filters in the first convolutional layer
            conv1_channels: 32,
            // 64 filters in the second convolutional layer
            conv2_channels: 64,
            // 128 filters in the third convolutional layer
            conv3_channels: 128,
            // 512 neurons in the fully connected layer
            fc1_features: 512,
            // Number of output classes (categories to classify)
            num_classes,
            // 50% dropout probability (helps prevent overfitting)
            dropout_prob: 0.5,
        }
    }
}

// Convolutional Block - a reusable component of our neural network
// Each block contains a convolutional layer, batch normalization, ReLU activation, and max pooling
#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    // Convolutional layer - extracts features from the input
    pub conv: Conv2d<B>,
    // Batch normalization - stabilizes and accelerates training
    pub batch_norm: BatchNorm<B, 2>,
    // ReLU activation - introduces non-linearity
    pub relu: ReLU,
    // Max pooling - reduces dimensions while keeping important features
    pub pool: MaxPool2d,
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
        
        // Configure the max pooling layer
        let pool_config = MaxPool2dConfig::new([2, 2]) // 2x2 pooling window
            .with_stride([2, 2]); // Move 2 pixels at a time
        
        Self {
            // Create the layers based on the configurations
            conv: Conv2d::new(conv_config),
            batch_norm: BatchNorm::new(batch_norm_config),
            relu: ReLU::new(),
            pool: MaxPool2d::new(pool_config),
        }
    }
    
    // Forward pass - process the input through all layers in the block
    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        // Apply convolution
        let x = self.conv.forward(input);
        // Apply batch normalization
        let x = self.batch_norm.forward(x);
        // Apply ReLU activation
        let x = self.relu.forward(x);
        // Apply max pooling
        let x = self.pool.forward(x);
        x
    }
}

// Image Classifier Model - the complete neural network for image classification
#[derive(Module, Debug)]
pub struct ImageClassifierModel<B: Backend> {
    // First convolutional block
    pub conv_block1: ConvBlock<B>,
    // Second convolutional block
    pub conv_block2: ConvBlock<B>,
    // Third convolutional block
    pub conv_block3: ConvBlock<B>,
    // Dropout layer - randomly deactivates neurons during training to prevent overfitting
    pub dropout: Dropout,
    // First fully connected layer
    pub fc1: Linear<B>,
    // Output layer - produces the final classification
    pub fc2: Linear<B>,
    // Number of output classes
    num_classes: usize,
}

impl<B: Backend> ImageClassifierModel<B> {
    // Create a new image classifier model based on the provided configuration
    pub fn new(config: &ImageClassifierConfig) -> Self {
        // Create the convolutional blocks
        let conv_block1 = ConvBlock::new(3, config.conv1_channels); // 3 input channels for RGB images
        let conv_block2 = ConvBlock::new(config.conv1_channels, config.conv2_channels);
        let conv_block3 = ConvBlock::new(config.conv2_channels, config.conv3_channels);
        
        // Configure the dropout layer
        let dropout = Dropout::new(DropoutConfig::new(config.dropout_prob));
        
        // Calculate the size of the flattened features after the convolutional blocks
        // Starting with 224x224 image, after 3 max pooling layers (each dividing by 2):
        // 224 -> 112 -> 56 -> 28
        // So we have 28x28 feature maps with conv3_channels channels
        let flattened_size = 28 * 28 * config.conv3_channels;
        
        // Configure the fully connected layers
        let fc1_config = LinearConfig::new(flattened_size, config.fc1_features);
        let fc2_config = LinearConfig::new(config.fc1_features, config.num_classes);
        
        Self {
            // Create the layers based on the configurations
            conv_block1,
            conv_block2,
            conv_block3,
            dropout,
            fc1: Linear::new(fc1_config),
            fc2: Linear::new(fc2_config),
            num_classes: config.num_classes,
        }
    }
    
    // Get the number of output classes
    pub fn num_classes(&self) -> usize {
        self.num_classes
    }
    
    // Forward pass - process the input through the entire network
    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 2> {
        // Apply the first convolutional block
        let x = self.conv_block1.forward(input);
        // Apply the second convolutional block
        let x = self.conv_block2.forward(x);
        // Apply the third convolutional block
        let x = self.conv_block3.forward(x);
        
        // Flatten the output for the fully connected layers
        let [batch_size, channels, height, width] = x.dims();
        let x = x.reshape([batch_size, channels * height * width]);
        
        // Apply dropout (only active during training)
        let x = self.dropout.forward(x);
        
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
