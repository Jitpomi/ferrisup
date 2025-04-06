// Configuration file for the Text Sentiment Analyzer
// CUSTOMIZE HERE: Edit these parameters to adapt the model to your needs

// Text processing parameters
pub const MAX_SEQUENCE_LENGTH: usize = 100;    // Maximum number of tokens per text
pub const VOCAB_SIZE: usize = 10000;           // Size of vocabulary
pub const EMBEDDING_DIM: usize = 100;          // Embedding dimension
pub const PADDING_TOKEN: usize = 0;            // Token used for padding

// Model architecture
pub const LSTM_UNITS: usize = 128;             // Number of LSTM units
pub const FC_LAYERS: [usize; 2] = [128, 64];   // Size of fully connected layers
pub const DROPOUT_RATE: f32 = 0.3;             // Dropout rate (0.0 to 1.0)

// Output configuration
pub const NUM_CLASSES: usize = 3;              // Number of sentiment classes
pub const CLASS_NAMES: [&str; NUM_CLASSES] = ["Negative", "Neutral", "Positive"];

// Training parameters
pub const BATCH_SIZE: usize = 32;              // Number of texts per batch
pub const LEARNING_RATE: f32 = 0.001;          // Learning rate for optimizer
pub const EPOCHS: usize = 10;                  // Number of training cycles

// Data augmentation options (set to false to disable)
pub const USE_AUGMENTATION: bool = true;       // Whether to use data augmentation
pub const RANDOM_DELETION: bool = true;        // Randomly delete words
pub const RANDOM_SWAP: bool = true;            // Randomly swap words
pub const RANDOM_SYNONYM: bool = false;        // Replace with synonyms (requires wordnet)

// Paths and filenames
pub const DEFAULT_DATA_DIR: &str = "data/sample"; // Default data directory
pub const DEFAULT_MODEL_FILE: &str = "model.json"; // Default model save file
pub const DEFAULT_VOCAB_FILE: &str = "vocab.json"; // Vocabulary file

// Advanced options
pub const USE_PRETRAINED_EMBEDDINGS: bool = false; // Use pre-trained word embeddings
pub const EARLY_STOPPING_PATIENCE: usize = 3;      // Stop training if no improvement
pub const TOKENIZATION_METHOD: &str = "word";      // "word" or "subword"
