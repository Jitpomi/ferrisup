// Data Predictor Library
// This library provides functionality for training and using numerical data prediction models

// Re-export modules for easier access
pub mod model;
pub mod data;
pub mod config;
pub mod training;

// Re-export key types and functions
pub use model::{DataPredictorConfig, DataPredictorModel};
pub use data::{DataBatcher, DataItem, DatasetInfo, load_dataset};
pub use config::{BATCH_SIZE, LEARNING_RATE, EPOCHS, HIDDEN_LAYERS, HIDDEN_SIZE, 
                 DEFAULT_DATA_FILE, DEFAULT_MODEL_FILE};
