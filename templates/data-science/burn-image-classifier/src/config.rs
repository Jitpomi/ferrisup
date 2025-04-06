// Configuration file for the Image Classifier
// CUSTOMIZE HERE: Edit these parameters to adapt the model to your needs

// Dataset parameters
pub const IMAGE_SIZE: usize = 32;         // Size images will be resized to (square)
pub const NUM_CHANNELS: usize = 3;        // 3 for RGB, 1 for grayscale
pub const NUM_CLASSES: usize = 10;        // Number of categories to classify

// Class names (modify these to match your categories)
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
pub const RANDOM_FLIP: bool = true;       // Randomly flip images horizontally
pub const RANDOM_CROP: bool = true;       // Randomly crop and resize images
pub const RANDOM_BRIGHTNESS: bool = true; // Randomly adjust brightness

// Paths and filenames
pub const DEFAULT_DATA_DIR: &str = "data/sample"; // Default data directory
pub const DEFAULT_MODEL_FILE: &str = "model.json"; // Default model save file
pub const DEFAULT_STATS_FILE: &str = "training_stats.json"; // Training statistics

// Advanced options
pub const USE_MIXED_PRECISION: bool = false; // Use mixed precision training (if supported)
pub const EARLY_STOPPING_PATIENCE: usize = 5; // Stop training if no improvement after this many epochs
