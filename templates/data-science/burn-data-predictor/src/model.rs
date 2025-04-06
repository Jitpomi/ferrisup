// Data Predictor Model Architecture
// This file defines the neural network model for numerical prediction

use burn::module::Module;
use burn::nn::{
    Linear, LinearConfig,
    Dropout, DropoutConfig,
};
use burn::tensor::{backend::Backend, Tensor};
use burn::config::Config;

// Import our configuration parameters
use crate::config::{
    HIDDEN_LAYERS, DROPOUT_RATE, ACTIVATION
};

// Model configuration
#[derive(Config)]
pub struct PredictorConfig {
    // CUSTOMIZE HERE: You can add or modify configuration parameters
    pub input_size: usize,
    pub hidden_layers: Vec<usize>,
    pub output_size: usize,
    pub dropout_rate: f32,
    pub activation: String,
}

impl PredictorConfig {
    pub fn new(
        input_size: usize,
        hidden_layers: Vec<usize>,
        output_size: usize,
        dropout_rate: f32,
        activation: String,
    ) -> Self {
        Self {
            input_size,
            hidden_layers,
            output_size,
            dropout_rate,
            activation,
        }
    }
    
    // Default configuration using values from config.rs
    pub fn default(input_size: usize, output_size: usize) -> Self {
        Self {
            input_size,
            hidden_layers: HIDDEN_LAYERS.to_vec(),
            output_size,
            dropout_rate: DROPOUT_RATE,
            activation: ACTIVATION.to_string(),
        }
    }
}

// The neural network model for numerical prediction
pub struct PredictorModel<B: Backend> {
    // Input layer
    input_layer: Linear<B>,
    
    // Hidden layers
    hidden_layers: Vec<Linear<B>>,
    
    // Output layer
    output_layer: Linear<B>,
    
    // Dropout for regularization
    dropout: Dropout,
    
    // Activation function to use
    activation: String,
}

impl<B: Backend> PredictorModel<B> {
    pub fn new(config: &PredictorConfig) -> Self {
        // CUSTOMIZE HERE: Modify the model architecture
        
        // Create the input layer
        let input_layer = LinearConfig::new(config.input_size, config.hidden_layers[0])
            .init();
        
        // Create hidden layers
        let mut hidden_layers = Vec::new();
        for i in 0..config.hidden_layers.len() - 1 {
            let layer = LinearConfig::new(config.hidden_layers[i], config.hidden_layers[i + 1])
                .init();
            hidden_layers.push(layer);
        }
        
        // Create the output layer
        let output_layer = LinearConfig::new(
            *config.hidden_layers.last().unwrap_or(&config.input_size),
            config.output_size
        ).init();
        
        // Create dropout
        let dropout = DropoutConfig::new(config.dropout_rate).init();
        
        Self {
            input_layer,
            hidden_layers,
            output_layer,
            dropout,
            activation: config.activation.clone(),
        }
    }
    
    // Apply the activation function based on the configuration
    fn apply_activation(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        match self.activation.as_str() {
            "relu" => x.relu(),
            "tanh" => x.tanh(),
            "sigmoid" => x.sigmoid(),
            _ => x.relu(), // Default to ReLU
        }
    }
}

// Implement the Module trait for our model
impl<B: Backend> Module<Tensor<B, 2>> for PredictorModel<B> {
    type Output = Tensor<B, 2>;
    
    fn forward(&self, input: Tensor<B, 2>) -> Self::Output {
        // CUSTOMIZE HERE: Modify the forward pass logic
        
        // Input layer
        let mut x = self.input_layer.forward(input);
        x = self.apply_activation(x);
        x = self.dropout.forward(x);
        
        // Hidden layers
        for layer in &self.hidden_layers {
            x = layer.forward(x);
            x = self.apply_activation(x);
            x = self.dropout.forward(x);
        }
        
        // Output layer (no activation for regression)
        self.output_layer.forward(x)
    }
}

// Helper function to create a model with default configuration
pub fn create_default_model<B: Backend>(input_size: usize, output_size: usize) -> PredictorModel<B> {
    let config = PredictorConfig::default(input_size, output_size);
    PredictorModel::new(&config)
}
