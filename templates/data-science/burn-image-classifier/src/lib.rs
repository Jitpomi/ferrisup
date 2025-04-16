// Public interface for the image classifier library
// This file exports the main components of the library

pub mod config;
pub mod data;
pub mod error;
pub mod model;
pub mod visualization;
#[cfg(test)]
mod tests;

// Re-export main components
pub use crate::config::{
    BATCH_SIZE, EPOCHS, DEFAULT_DATA_DIR, DEFAULT_MODEL_FILE, 
    IMAGE_SIZE, NUM_CHANNELS, NUM_CLASSES, ImageClassifierConfig
};
pub use crate::data::{ImageDataset, ImageBatcher, image_to_tensor, generate_synthetic_dataset, load_image_dataset};
pub use crate::error::{ImageClassifierError, Result};
pub use crate::model::ImageClassifierModel;
pub use crate::visualization::{plot_training_history, plot_predictions, plot_confusion_matrix, Accuracy};
