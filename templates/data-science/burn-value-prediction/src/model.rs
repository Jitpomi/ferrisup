// Neural Network Model for Value Prediction
// This file defines the structure of our neural network
// that will learn to predict numeric values

use burn::module::Module;
use burn::nn::{Linear, LinearConfig, ReLU};
use burn::tensor::{backend::Backend, Tensor};
use burn::train::RegressionOutput;
use burn::config::Config;

// Configuration for our regression model
// This defines the structure and parameters of our neural network
#[derive(Config)]
pub struct RegressionConfig {
    // Number of input features (e.g., house size, number of bedrooms)
    pub input_features: usize,
    // Number of neurons in the first hidden layer
    pub hidden1_features: usize,
    // Number of neurons in the second hidden layer
    pub hidden2_features: usize,
    // Number of output values to predict (usually 1 for regression)
    pub output_features: usize,
}

impl RegressionConfig {
    // Create a new configuration with default values
    pub fn new(input_features: usize) -> Self {
        Self {
            // Number of input features (from the dataset)
            input_features,
            // 64 neurons in the first hidden layer
            hidden1_features: 64,
            // 32 neurons in the second hidden layer
            hidden2_features: 32,
            // 1 output (the value we're predicting)
            output_features: 1,
        }
    }
}

// Regression Model - the neural network for value prediction
#[derive(Module, Debug)]
pub struct RegressionModel<B: Backend> {
    // First fully connected layer
    pub fc1: Linear<B>,
    // Second fully connected layer
    pub fc2: Linear<B>,
    // Output layer
    pub fc3: Linear<B>,
    // ReLU activation function
    pub relu: ReLU,
}

impl<B: Backend> RegressionModel<B> {
    // Create a new regression model based on the provided configuration
    pub fn new(config: &RegressionConfig) -> Self {
        // Configure the fully connected layers
        let fc1_config = LinearConfig::new(config.input_features, config.hidden1_features);
        let fc2_config = LinearConfig::new(config.hidden1_features, config.hidden2_features);
        let fc3_config = LinearConfig::new(config.hidden2_features, config.output_features);
        
        Self {
            // Create the layers based on the configurations
            fc1: Linear::new(fc1_config),
            fc2: Linear::new(fc2_config),
            fc3: Linear::new(fc3_config),
            relu: ReLU::new(),
        }
    }
    
    // Forward pass - process the input through the entire network
    pub fn forward(&self, input: Tensor<B, 2>) -> Tensor<B, 2> {
        // Apply the first layer followed by ReLU activation
        let x = self.fc1.forward(input);
        let x = self.relu.forward(x);
        
        // Apply the second layer followed by ReLU activation
        let x = self.fc2.forward(x);
        let x = self.relu.forward(x);
        
        // Apply the output layer (no activation for regression)
        let x = self.fc3.forward(x);
        x
    }
    
    // Forward pass for regression - includes loss calculation
    pub fn forward_regression(
        &self,
        features: Tensor<B, 2>,
        targets: Tensor<B, 2>,
    ) -> RegressionOutput {
        // Get the model's predictions
        let output = self.forward(features);
        
        // Calculate the Mean Squared Error loss
        // This measures how far off our predictions are from the actual values
        let loss = output.mse_loss(targets);
        
        // Return the loss
        RegressionOutput::new(loss)
    }
}
