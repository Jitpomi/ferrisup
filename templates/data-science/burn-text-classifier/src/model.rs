// Neural Network Model for Text Classification
// This file defines the structure of our Recurrent Neural Network (RNN)
// that will learn to classify text into different categories

use burn::module::Module;
use burn::nn::{
    embedding::{Embedding, EmbeddingConfig},
    lstm::{Lstm, LstmConfig},
    Dropout, DropoutConfig, Linear, LinearConfig, ReLU,
};
use burn::tensor::{backend::Backend, Tensor};
use burn::train::ClassificationOutput;
use burn::config::Config;

// Configuration for our text classifier model
// This defines the structure and parameters of our neural network
#[derive(Config)]
pub struct TextClassifierConfig {
    // Size of the vocabulary (number of unique words/tokens)
    pub vocab_size: usize,
    // Maximum sequence length (number of tokens per text)
    pub max_length: usize,
    // Dimension of word embeddings
    pub embedding_dim: usize,
    // Number of hidden units in the LSTM layer
    pub hidden_size: usize,
    // Number of LSTM layers
    pub num_layers: usize,
    // Dropout probability (to prevent overfitting)
    pub dropout_prob: f64,
    // Number of output classes (categories)
    pub num_classes: usize,
}

impl TextClassifierConfig {
    // Create a new configuration with default values
    pub fn new(vocab_size: usize, max_length: usize, num_classes: usize) -> Self {
        Self {
            // Size of the vocabulary (number of unique words/tokens)
            vocab_size,
            // Maximum sequence length (number of tokens per text)
            max_length,
            // 100-dimensional word embeddings
            embedding_dim: 100,
            // 128 hidden units in the LSTM layer
            hidden_size: 128,
            // 2 LSTM layers
            num_layers: 2,
            // 50% dropout probability (helps prevent overfitting)
            dropout_prob: 0.5,
            // Number of output classes (categories to classify)
            num_classes,
        }
    }
}

// Text Classifier Model - the neural network for text classification
#[derive(Module, Debug)]
pub struct TextClassifierModel<B: Backend> {
    // Embedding layer - converts token IDs to vectors
    pub embedding: Embedding<B>,
    // LSTM layer - processes sequences of word embeddings
    pub lstm: Lstm<B>,
    // Dropout layer - randomly deactivates neurons during training to prevent overfitting
    pub dropout: Dropout,
    // Output layer - produces the final classification
    pub output: Linear<B>,
    // Number of output classes
    num_classes: usize,
}

impl<B: Backend> TextClassifierModel<B> {
    // Create a new text classifier model based on the provided configuration
    pub fn new(config: &TextClassifierConfig) -> Self {
        // Configure the embedding layer
        let embedding_config = EmbeddingConfig::new(config.vocab_size, config.embedding_dim);
        
        // Configure the LSTM layer
        let lstm_config = LstmConfig::new(config.embedding_dim, config.hidden_size)
            .with_num_layers(config.num_layers);
        
        // Configure the dropout layer
        let dropout = Dropout::new(DropoutConfig::new(config.dropout_prob));
        
        // Configure the output layer
        let output_config = LinearConfig::new(config.hidden_size, config.num_classes);
        
        Self {
            // Create the layers based on the configurations
            embedding: Embedding::new(embedding_config),
            lstm: Lstm::new(lstm_config),
            dropout,
            output: Linear::new(output_config),
            num_classes: config.num_classes,
        }
    }
    
    // Get the number of output classes
    pub fn num_classes(&self) -> usize {
        self.num_classes
    }
    
    // Forward pass - process the input through the entire network
    pub fn forward(&self, tokens: Tensor<B, 2, usize>) -> Tensor<B, 2> {
        // Convert token IDs to embeddings
        // [batch_size, seq_len] -> [batch_size, seq_len, embedding_dim]
        let x = self.embedding.forward(tokens);
        
        // Process the sequence with LSTM
        // [batch_size, seq_len, embedding_dim] -> [batch_size, seq_len, hidden_size]
        let x = self.lstm.forward(x);
        
        // Get the last output from the sequence
        // [batch_size, seq_len, hidden_size] -> [batch_size, hidden_size]
        let [batch_size, seq_len, hidden_size] = x.dims();
        let last_output = x.narrow(1, seq_len - 1, 1).reshape([batch_size, hidden_size]);
        
        // Apply dropout to prevent overfitting
        let x = self.dropout.forward(last_output);
        
        // Apply the output layer to get class scores
        // [batch_size, hidden_size] -> [batch_size, num_classes]
        let x = self.output.forward(x);
        x
    }
    
    // Forward pass for classification - includes loss calculation
    pub fn forward_classification(
        &self,
        tokens: Tensor<B, 2, usize>,
        targets: Tensor<B, 1, usize>,
    ) -> ClassificationOutput {
        // Get the model's predictions
        let output = self.forward(tokens);
        
        // Calculate the loss (how far off our predictions are)
        let loss = output.cross_entropy_loss(targets);
        
        // Calculate the accuracy (percentage of correct predictions)
        let accuracy = output.accuracy(targets);
        
        // Return both loss and accuracy
        ClassificationOutput::new(loss, accuracy)
    }
}
