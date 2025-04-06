// Text Sentiment Analyzer Library
// This library provides functionality for training and using text sentiment analysis models

// Re-export modules for easier access
pub mod model;
pub mod data;
pub mod config;
pub mod training;

// Re-export key types and functions
pub use model::{TextSentimentConfig, TextSentimentModel};
pub use data::{TextBatcher, TextItem, TextDataset, load_text_dataset};
pub use config::{BATCH_SIZE, LEARNING_RATE, EPOCHS, VOCAB_SIZE, EMBEDDING_DIM, HIDDEN_SIZE, 
                 DEFAULT_DATA_FILE, DEFAULT_MODEL_FILE};
