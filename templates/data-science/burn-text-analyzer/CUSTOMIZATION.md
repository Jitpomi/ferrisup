# Customizing the Text Sentiment Analyzer

This guide explains how to customize the Text Sentiment Analyzer template for your specific needs. All major customization points are clearly marked with `// CUSTOMIZE HERE` comments in the code.

## Quick Customization

For most use cases, you only need to modify the `config.rs` file, which centralizes all tweakable parameters.

## 1. Adjusting Text Processing

In `config.rs`, you can modify:

```rust
// Text processing parameters
pub const MAX_SEQUENCE_LENGTH: usize = 100;    // Maximum number of tokens per text
pub const VOCAB_SIZE: usize = 10000;           // Size of vocabulary
pub const EMBEDDING_DIM: usize = 100;          // Embedding dimension
pub const PADDING_TOKEN: usize = 0;            // Token used for padding
```

- **Increase `MAX_SEQUENCE_LENGTH`** for longer texts
- **Increase `VOCAB_SIZE`** for more diverse vocabulary
- **Increase `EMBEDDING_DIM`** for more expressive word representations

## 2. Changing the Model Architecture

In `config.rs`, you can adjust:

```rust
// Model architecture
pub const LSTM_UNITS: usize = 128;             // Number of LSTM units
pub const FC_LAYERS: [usize; 2] = [128, 64];   // Size of fully connected layers
pub const DROPOUT_RATE: f32 = 0.3;             // Dropout rate (0.0 to 1.0)
```

- **Increase `LSTM_UNITS`** for more complex pattern recognition
- **Modify `FC_LAYERS`** to add more layers or change their size
- **Adjust `DROPOUT_RATE`** to control overfitting (higher = more regularization)

## 3. Changing Output Classes

By default, the model classifies text into three sentiment classes (Negative, Neutral, Positive). To change this:

1. Update the class configuration in `config.rs`:

```rust
// Output configuration
pub const NUM_CLASSES: usize = 3;              // Number of sentiment classes
pub const CLASS_NAMES: [&str; NUM_CLASSES] = ["Negative", "Neutral", "Positive"];
```

2. Organize your training data accordingly, with one directory per class.

## 4. Training Parameters

Adjust training behavior in `config.rs`:

```rust
// Training parameters
pub const BATCH_SIZE: usize = 32;              // Number of texts per batch
pub const LEARNING_RATE: f32 = 0.001;          // Learning rate for optimizer
pub const EPOCHS: usize = 10;                  // Number of training cycles
```

- **Increase `BATCH_SIZE`** for faster training (requires more memory)
- **Decrease `LEARNING_RATE`** for more stable training
- **Increase `EPOCHS`** for longer training

## 5. Data Augmentation

Control data augmentation in `config.rs`:

```rust
// Data augmentation options
pub const USE_AUGMENTATION: bool = true;       // Whether to use data augmentation
pub const RANDOM_DELETION: bool = true;        // Randomly delete words
pub const RANDOM_SWAP: bool = true;            // Randomly swap words
pub const RANDOM_SYNONYM: bool = false;        // Replace with synonyms
```

## Advanced Customization

### Custom Tokenization

To change how text is tokenized, modify the `tokenize` function in `data.rs`:

```rust
// Simple word tokenization
fn tokenize(text: &str) -> Vec<String> {
    // CUSTOMIZE HERE: Modify tokenization approach
    
    // Convert to lowercase and split by whitespace
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}
```

Options include:
- Character-level tokenization
- Subword tokenization (BPE, WordPiece)
- Using external tokenizers

### Custom Model Architecture

For more advanced model changes, modify the `TextAnalyzerModel` struct and its implementation in `model.rs`:

```rust
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
```

You can:
- Replace LSTM with GRU or Transformer layers
- Add attention mechanisms
- Implement more complex architectures

### Custom Data Loading

To support different data formats, modify the data loading functions in `data.rs`:

```rust
// Load a text dataset from a directory
pub fn load_text_dataset(data_dir: &str) -> Result<TextDataset> {
    // CUSTOMIZE HERE: Modify how texts are loaded and organized
    
    // Implementation...
}
```

You can add support for:
- Different file formats
- Different directory structures
- Online data sources

## Example: Adapting for Topic Classification

Here's how to adapt the template for topic classification instead of sentiment analysis:

1. Update `config.rs`:
```rust
pub const NUM_CLASSES: usize = 5;
pub const CLASS_NAMES: [&str; NUM_CLASSES] = ["Sports", "Politics", "Technology", "Entertainment", "Business"];
```

2. Organize your data with one directory per topic:
```
data/
  ├── sports/
  ├── politics/
  ├── technology/
  ├── entertainment/
  └── business/
```

3. Train the model:
```bash
cargo run -- train --data-dir path/to/your/data
```

## Example: Using Pre-trained Word Embeddings

To use pre-trained word embeddings (like GloVe or Word2Vec):

1. Enable pre-trained embeddings in `config.rs`:
```rust
pub const USE_PRETRAINED_EMBEDDINGS: bool = true;
```

2. Implement embedding loading in `data.rs` (look for the `// CUSTOMIZE HERE` comment in the `Vocabulary` implementation).

## Getting Help

If you need more help customizing this template, check out:
- [Burn Documentation](https://github.com/tracel-ai/burn)
- [Rust NLP Resources](https://github.com/rust-nlp)
