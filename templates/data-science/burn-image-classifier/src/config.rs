// Configuration file for the Image Classifier
// CUSTOMIZE HERE: Edit these parameters to adapt the model to your needs

use burn::config::Config;
use serde::{Deserialize, Serialize};

// Dataset parameters
pub const IMAGE_SIZE: usize = 32;         // Size images will be resized to (square)
pub const NUM_CHANNELS: usize = 3;        // 3 for RGB, 1 for grayscale
pub const NUM_CLASSES: usize = 10;        // Number of categories to classify

// Class names (modify these to match your categories)
#[allow(dead_code)]
pub const CLASS_NAMES: [&str; NUM_CLASSES] = [
    "airplane", "automobile", "bird", "cat", "deer", 
    "dog", "frog", "horse", "ship", "truck"
];

// Model architecture
pub const CONV_FILTERS: [usize; 3] = [32, 64, 128];  // Number of filters in each conv layer
pub const FC_LAYERS: [usize; 2] = [512, 128];        // Size of fully connected layers
pub const DROPOUT_RATE: f32 = 0.5;                   // Dropout rate (0.0 to 1.0)

// Training parameters
pub const BATCH_SIZE: usize = 64;         // Number of images per batch
pub const LEARNING_RATE: f32 = 0.001;     // Learning rate for optimizer
pub const EPOCHS: usize = 10;             // Number of training cycles

// Data augmentation options (set to false to disable)
pub const USE_AUGMENTATION: bool = true;  // Whether to use data augmentation
#[allow(dead_code)]
pub const RANDOM_FLIP: bool = true;       // Randomly flip images horizontally
#[allow(dead_code)]
pub const RANDOM_CROP: bool = true;       // Randomly crop and resize images
#[allow(dead_code)]
pub const RANDOM_BRIGHTNESS: bool = true; // Randomly adjust brightness

// Paths and filenames
pub const DEFAULT_DATA_DIR: &str = "data";          // Default directory for training data
pub const DEFAULT_MODEL_FILE: &str = "model.json"; // Default filename for saved model
#[allow(dead_code)]
pub const HISTORY_FILE: &str = "training_history.json";   // File to save training history

// Device configuration
#[allow(dead_code)]
pub const USE_GPU: bool = false;          // Whether to use GPU acceleration if available

// Model configuration with Burn's Config trait
#[derive(Config, Debug)]
pub struct ImageClassifierConfig {
    pub num_classes: usize,
    pub conv_filters: Vec<usize>,
    pub fc_layers: Vec<usize>,
    pub dropout_rate: f32,
}

impl Default for ImageClassifierConfig {
    fn default() -> Self {
        Self {
            num_classes: NUM_CLASSES,
            conv_filters: CONV_FILTERS.to_vec(),
            fc_layers: FC_LAYERS.to_vec(),
            dropout_rate: DROPOUT_RATE,
        }
    }
}

// Training configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub batch_size: usize,
    pub learning_rate: f32,
    pub epochs: usize,
    pub use_augmentation: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            batch_size: BATCH_SIZE,
            learning_rate: LEARNING_RATE,
            epochs: EPOCHS,
            use_augmentation: USE_AUGMENTATION,
        }
    }
}
