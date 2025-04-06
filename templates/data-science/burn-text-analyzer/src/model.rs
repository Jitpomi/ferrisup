// Text Sentiment Analyzer Model Architecture
// This file defines the neural network model for text sentiment analysis

use burn::module::Module;
use burn::nn::{
    Embedding, EmbeddingConfig,
    Linear, LinearConfig,
    RNN, RNNConfig, RNNConfig_Type,
    Dropout, DropoutConfig,
};
use burn::tensor::{backend::Backend, Tensor};
use burn::config::Config;

// Import our configuration parameters
use crate::config::{
    VOCAB_SIZE, EMBEDDING_DIM, LSTM_UNITS, 
    FC_LAYERS, DROPOUT_RATE, NUM_CLASSES
};

// Model configuration
#[derive(Config)]
pub struct TextAnalyzerConfig {
    // CUSTOMIZE HERE: You can add or modify configuration parameters
    pub vocab_size: usize,
    pub embedding_dim: usize,
    pub lstm_units: usize,
    pub num_classes: usize,
}

impl TextAnalyzerConfig {
    pub fn new(
        vocab_size: usize,
        embedding_dim: usize,
        lstm_units: usize,
        num_classes: usize,
    ) -> Self {
        Self {
            vocab_size,
            embedding_dim,
            lstm_units,
            num_classes,
        }
    }
    
    // Default configuration using values from config.rs
    pub fn default() -> Self {
        Self {
            vocab_size: VOCAB_SIZE,
            embedding_dim: EMBEDDING_DIM,
            lstm_units: LSTM_UNITS,
            num_classes: NUM_CLASSES,
        }
    }
}

// The RNN model for text classification
pub struct TextAnalyzerModel<B: Backend> {
    // Word embedding layer
    embedding: Embedding<B>,
    
    // LSTM layer
    lstm: RNN<B>,
    
    // Fully connected layers
    fc1: Linear<B>,
    fc2: Linear<B>,
    fc3: Linear<B>,
    
    // Dropout for regularization
    dropout: Dropout,
}

impl<B: Backend> TextAnalyzerModel<B> {
    pub fn new(config: &TextAnalyzerConfig) -> Self {
        // CUSTOMIZE HERE: Modify the model architecture
        
        // Word embedding layer
        let embedding = EmbeddingConfig::new(config.vocab_size, config.embedding_dim)
            .init();
        
        // LSTM layer
        let lstm = RNNConfig::new(config.embedding_dim, config.lstm_units)
            .with_type(RNNConfig_Type::LSTM)
            .with_bidirectional(true) // Use bidirectional LSTM
            .init();
        
        // Calculate the size of the LSTM output
        // Bidirectional LSTM has 2x the units
        let lstm_output_size = config.lstm_units * 2;
        
        // Fully connected layers
        let fc1 = LinearConfig::new(lstm_output_size, FC_LAYERS[0]).init();
        let fc2 = LinearConfig::new(FC_LAYERS[0], FC_LAYERS[1]).init();
        let fc3 = LinearConfig::new(FC_LAYERS[1], config.num_classes).init();
        
        // Dropout for regularization
        let dropout = DropoutConfig::new(DROPOUT_RATE).init();
        
        Self {
            embedding,
            lstm,
            fc1,
            fc2,
            fc3,
            dropout,
        }
    }
}

// Implement the Module trait for our model
impl<B: Backend> Module<Tensor<B, 2>> for TextAnalyzerModel<B> {
    type Output = Tensor<B, 2>;
    
    fn forward(&self, input: Tensor<B, 2>) -> Self::Output {
        // CUSTOMIZE HERE: Modify the forward pass logic
        
        // Convert token IDs to embeddings
        // Shape: [batch_size, seq_len] -> [batch_size, seq_len, embedding_dim]
        let x = self.embedding.forward(input);
        
        // Pass through LSTM
        // Shape: [batch_size, seq_len, embedding_dim] -> [batch_size, seq_len, lstm_units*2]
        let lstm_out = self.lstm.forward(x);
        
        // Get the last output of the LSTM
        // Shape: [batch_size, seq_len, lstm_units*2] -> [batch_size, lstm_units*2]
        let [batch_size, seq_len, hidden_size] = lstm_out.shape().dims;
        let last_output = lstm_out.slice([0..batch_size, [seq_len - 1], 0..hidden_size]);
        let last_output = last_output.reshape([batch_size, hidden_size]);
        
        // Fully connected layers
        let x = self.fc1.forward(last_output);
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
pub fn create_default_model<B: Backend>() -> TextAnalyzerModel<B> {
    let config = TextAnalyzerConfig::default();
    TextAnalyzerModel::new(&config)
}
