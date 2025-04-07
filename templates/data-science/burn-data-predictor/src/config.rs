// Configuration file for the Data Predictor
// CUSTOMIZE HERE: Edit these parameters to adapt the model to your needs

// Data processing parameters
pub const FEATURE_COLUMNS: [&str; 5] = ["feature1", "feature2", "feature3", "feature4", "feature5"];
pub const TARGET_COLUMN: &str = "target";
pub const NORMALIZE_FEATURES: bool = true;     // Whether to normalize input features
pub const NORMALIZE_TARGET: bool = true;       // Whether to normalize target values
pub const TEST_SPLIT_RATIO: f32 = 0.2;         // Portion of data to use for testing

// Model architecture
pub const HIDDEN_LAYERS: [usize; 2] = [64, 32]; // Size of hidden layers
pub const ACTIVATION: &str = "relu";           // Activation function: "relu", "tanh", "sigmoid"
pub const DROPOUT_RATE: f32 = 0.1;             // Dropout rate (0.0 to 1.0)

// Training parameters
pub const BATCH_SIZE: usize = 32;              // Number of samples per batch
pub const LEARNING_RATE: f32 = 0.001;          // Learning rate for optimizer
pub const EPOCHS: usize = 100;                 // Number of training cycles
pub const EARLY_STOPPING_PATIENCE: usize = 10; // Stop training if no improvement

// Data augmentation options (set to false to disable)
pub const USE_AUGMENTATION: bool = false;      // Whether to use data augmentation
pub const NOISE_LEVEL: f32 = 0.05;             // Level of Gaussian noise to add
pub const FEATURE_DROPOUT: f32 = 0.1;          // Randomly zero out features

// Paths and filenames
pub const DEFAULT_DATA_FILE: &str = "data/sample.csv"; // Default data file
pub const DEFAULT_MODEL_FILE: &str = "model.json";     // Default model save file
pub const DEFAULT_STATS_FILE: &str = "stats.json";     // Statistics for normalization

// Advanced options
pub const OPTIMIZER: &str = "adam";            // "adam", "sgd", or "rmsprop"
pub const WEIGHT_DECAY: f32 = 0.0001;          // L2 regularization strength
pub const CLIP_GRADIENT: f32 = 1.0;            // Gradient clipping threshold
