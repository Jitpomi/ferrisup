// Image Classifier Library
// This library provides functionality for training and using image classification models

// Re-export modules for easier access
pub mod model;
pub mod data;
pub mod config;
pub mod training;

// Re-export key types and functions
pub use model::{ImageClassifierConfig, ImageClassifierModel};
pub use data::{ImageBatcher, ImageItem, ImageDataset, load_image_dataset};
pub use config::{BATCH_SIZE, LEARNING_RATE, EPOCHS, IMAGE_SIZE, DEFAULT_DATA_DIR, DEFAULT_MODEL_FILE};
pub use training::{TrainingStepHandler, ValidationStepHandler};
