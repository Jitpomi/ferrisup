// Data Handling for Text Sentiment Analysis
// This file handles loading and processing text data

use burn::data::dataset::{Dataset, InMemDataset};
use burn::data::dataloader::batcher::Batcher;
use burn::tensor::{backend::Backend, Data, Tensor};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufRead, BufReader};
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use rand::{Rng, thread_rng, seq::SliceRandom};
use serde::{Serialize, Deserialize};

// Import our configuration parameters
use crate::config::{
    MAX_SEQUENCE_LENGTH, VOCAB_SIZE, PADDING_TOKEN, NUM_CLASSES, CLASS_NAMES,
    USE_AUGMENTATION, RANDOM_DELETION, RANDOM_SWAP, RANDOM_SYNONYM
};

// Text Item - represents a single batch of text data
#[derive(Clone, Debug)]
pub struct TextItem<B: Backend> {
    // Batch of token sequences - shape [batch_size, seq_length]
    pub tokens: Tensor<B, 2>,
    // Batch of labels - shape [batch_size, num_classes]
    pub labels: Tensor<B, 2>,
}

// Text Batcher - converts raw text data into batches for the model
pub struct TextBatcher<B: Backend> {
    batch_size: usize,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> TextBatcher<B> {
    // Create a new batcher with the specified batch size
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

// Raw Text item - represents a single example with text and label
#[derive(Clone, Debug)]
pub struct RawTextItem {
    // Original text
    pub text: String,
    // Tokenized text (as token IDs)
    pub tokens: Vec<usize>,
    // Class label (integer)
    pub label: usize,
}

// Implement the Batcher trait for our TextBatcher
impl<B: Backend> Batcher<RawTextItem, TextItem<B>> for TextBatcher<B> {
    // Convert a batch of raw items into a processed TextItem
    fn batch(&self, items: Vec<RawTextItem>) -> TextItem<B> {
        // Number of items in this batch
        let batch_size = items.len();
        
        // Create tensors to hold the batch data
        let mut tokens_data = Data::new(
            vec![PADDING_TOKEN; batch_size * MAX_SEQUENCE_LENGTH],
            [batch_size, MAX_SEQUENCE_LENGTH],
        );
        
        let mut labels_data = Data::new(
            vec![0.0; batch_size * NUM_CLASSES],
            [batch_size, NUM_CLASSES],
        );
        
        // Process each item in the batch
        for (i, item) in items.iter().enumerate() {
            // Copy the token IDs to the batch tensor
            for (j, &token) in item.tokens.iter().enumerate() {
                if j < MAX_SEQUENCE_LENGTH {
                    tokens_data.value_mut()[i * MAX_SEQUENCE_LENGTH + j] = token;
                } else {
                    break; // Truncate if longer than MAX_SEQUENCE_LENGTH
                }
            }
            
            // Set the one-hot encoded label
            let label_offset = i * NUM_CLASSES;
            for j in 0..NUM_CLASSES {
                labels_data.value_mut()[label_offset + j] = if j == item.label { 1.0 } else { 0.0 };
            }
        }
        
        // Create tensors from the data
        let tokens = Tensor::<B, 2>::from_data(tokens_data);
        let labels = Tensor::<B, 2>::from_data(labels_data);
        
        // Return the processed batch
        TextItem { tokens, labels }
    }
}

// Vocabulary for tokenization
#[derive(Serialize, Deserialize, Clone)]
pub struct Vocabulary {
    // Mapping from token to ID
    token_to_id: HashMap<String, usize>,
    // Mapping from ID to token
    id_to_token: HashMap<usize, String>,
    // Special tokens
    pub pad_token: String,
    pub unk_token: String,
}

impl Vocabulary {
    // Create a new empty vocabulary
    pub fn new() -> Self {
        let mut token_to_id = HashMap::new();
        let mut id_to_token = HashMap::new();
        
        // Add special tokens
        let pad_token = "<PAD>".to_string();
        let unk_token = "<UNK>".to_string();
        
        token_to_id.insert(pad_token.clone(), PADDING_TOKEN);
        id_to_token.insert(PADDING_TOKEN, pad_token.clone());
        
        token_to_id.insert(unk_token.clone(), 1);
        id_to_token.insert(1, unk_token.clone());
        
        Self {
            token_to_id,
            id_to_token,
            pad_token,
            unk_token,
        }
    }
    
    // Build vocabulary from a list of texts
    pub fn build_from_texts(texts: &[String], max_size: usize) -> Self {
        let mut vocab = Self::new();
        let mut token_counts: HashMap<String, usize> = HashMap::new();
        
        // Count token frequencies
        for text in texts {
            for token in tokenize(text) {
                *token_counts.entry(token).or_insert(0) += 1;
            }
        }
        
        // Sort tokens by frequency
        let mut token_counts: Vec<(String, usize)> = token_counts.into_iter().collect();
        token_counts.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Add most frequent tokens to vocabulary (up to max_size)
        let start_id = vocab.token_to_id.len(); // Start after special tokens
        for (i, (token, _)) in token_counts.iter().enumerate().take(max_size - start_id) {
            let id = i + start_id;
            vocab.token_to_id.insert(token.clone(), id);
            vocab.id_to_token.insert(id, token.clone());
        }
        
        vocab
    }
    
    // Convert a token to its ID
    pub fn token_to_id(&self, token: &str) -> usize {
        *self.token_to_id.get(token).unwrap_or_else(|| self.token_to_id.get(&self.unk_token).unwrap())
    }
    
    // Convert an ID to its token
    pub fn id_to_token(&self, id: usize) -> String {
        self.id_to_token.get(&id).cloned().unwrap_or_else(|| self.unk_token.clone())
    }
    
    // Get the size of the vocabulary
    pub fn len(&self) -> usize {
        self.token_to_id.len()
    }
    
    // Save vocabulary to a file
    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    // Load vocabulary from a file
    pub fn load(path: &str) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let vocab = serde_json::from_str(&json)?;
        Ok(vocab)
    }
}

// Dataset structure to hold our text data
pub struct TextDataset {
    // List of examples (texts and labels)
    items: Vec<RawTextItem>,
    // Vocabulary for tokenization
    vocabulary: Vocabulary,
    // Class names
    class_names: Vec<String>,
}

impl TextDataset {
    // Create a new dataset with the given items, vocabulary, and class names
    pub fn new(items: Vec<RawTextItem>, vocabulary: Vocabulary, class_names: Vec<String>) -> Self {
        Self { items, vocabulary, class_names }
    }
    
    // Get the vocabulary
    pub fn vocabulary(&self) -> &Vocabulary {
        &self.vocabulary
    }
    
    // Get the number of classes
    pub fn num_classes(&self) -> usize {
        self.class_names.len()
    }
    
    // Get the class names
    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }
    
    // Split the dataset into training and validation sets
    pub fn split_by_ratio(&self, ratios: [f32; 2]) -> (InMemDataset<RawTextItem>, InMemDataset<RawTextItem>) {
        let total: f32 = ratios.iter().sum();
        let ratio_a = ratios[0] / total;
        
        let n_a = (self.items.len() as f32 * ratio_a).round() as usize;
        let n_a = n_a.min(self.items.len());
        
        let items_a = self.items[0..n_a].to_vec();
        let items_b = self.items[n_a..].to_vec();
        
        (
            InMemDataset::new(items_a),
            InMemDataset::new(items_b),
        )
    }
}

// Implement the Dataset trait for our TextDataset
impl Dataset<RawTextItem> for TextDataset {
    // Get the number of examples in the dataset
    fn len(&self) -> usize {
        self.items.len()
    }
    
    // Get a specific example by index
    fn get(&self, index: usize) -> Option<RawTextItem> {
        self.items.get(index).cloned()
    }
}

// Load a text dataset from a directory
// The directory should have subdirectories for each class
pub fn load_text_dataset(data_dir: &str) -> Result<TextDataset> {
    // CUSTOMIZE HERE: Modify how texts are loaded and organized
    
    let data_path = Path::new(data_dir);
    if !data_path.exists() {
        return Err(anyhow!("Data directory does not exist: {}", data_dir));
    }
    
    let mut texts = Vec::new();
    let mut labels = Vec::new();
    let mut class_names = Vec::new();
    
    // Read class directories (each subdirectory is a class)
    let class_dirs = fs::read_dir(data_path)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
        })
        .collect::<Vec<_>>();
    
    if class_dirs.is_empty() {
        return Err(anyhow!("No class directories found in {}", data_dir));
    }
    
    // Sort class directories to ensure consistent class indices
    let mut class_dirs = class_dirs;
    class_dirs.sort_by_key(|dir| dir.file_name());
    
    // Process each class directory
    for (class_idx, class_dir) in class_dirs.iter().enumerate() {
        let class_name = class_dir.file_name().to_string_lossy().to_string();
        class_names.push(class_name.clone());
        
        // Read all text files in this class directory
        let text_files = fs::read_dir(class_dir.path())?
            .filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                let extension = path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");
                
                // Only include text files
                matches!(extension.to_lowercase().as_str(), "txt" | "text")
            })
            .collect::<Vec<_>>();
        
        // Add each text file to our dataset
        for text_file in text_files {
            let file = fs::File::open(text_file.path())?;
            let reader = BufReader::new(file);
            
            // Read the entire file as a single text
            let text = reader.lines()
                .filter_map(Result::ok)
                .collect::<Vec<String>>()
                .join(" ");
            
            texts.push(text);
            labels.push(class_idx);
        }
    }
    
    // If no class names were found in the directory, use the default ones
    if class_names.is_empty() {
        class_names = CLASS_NAMES.iter().map(|&s| s.to_string()).collect();
    }
    
    // If no texts were loaded, return an error
    if texts.is_empty() {
        return Err(anyhow!("No text files found in {}", data_dir));
    }
    
    // Build vocabulary from the texts
    let vocabulary = Vocabulary::build_from_texts(&texts, VOCAB_SIZE);
    
    // Create raw text items
    let mut items = Vec::new();
    for (text, label) in texts.iter().zip(labels.iter()) {
        // Apply data augmentation if enabled
        let processed_text = if USE_AUGMENTATION {
            apply_augmentation(text)
        } else {
            text.clone()
        };
        
        // Tokenize the text
        let tokens = tokenize_and_convert(&processed_text, &vocabulary);
        
        items.push(RawTextItem {
            text: processed_text,
            tokens,
            label: *label,
        });
    }
    
    // Create and return the dataset
    Ok(TextDataset::new(items, vocabulary, class_names))
}

// Load a text dataset from a CSV file
pub fn load_text_dataset_from_csv(file_path: &str, text_column: &str, label_column: &str) -> Result<TextDataset> {
    // CUSTOMIZE HERE: Modify how CSV data is loaded
    
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    
    let mut texts = Vec::new();
    let mut labels = Vec::new();
    let mut class_set = HashSet::new();
    
    // Read the CSV file
    for result in csv_reader.records() {
        let record = result?;
        
        // Get the text and label from the specified columns
        let text = record.get(0).ok_or_else(|| anyhow!("Text column not found"))?.to_string();
        let label_str = record.get(1).ok_or_else(|| anyhow!("Label column not found"))?.to_string();
        
        texts.push(text);
        class_set.insert(label_str.clone());
        labels.push(label_str);
    }
    
    // Create a mapping from label string to index
    let mut class_names: Vec<String> = class_set.into_iter().collect();
    class_names.sort(); // Sort for consistency
    
    let label_to_index: HashMap<String, usize> = class_names.iter()
        .enumerate()
        .map(|(i, label)| (label.clone(), i))
        .collect();
    
    // Convert label strings to indices
    let label_indices: Vec<usize> = labels.iter()
        .map(|label| *label_to_index.get(label).unwrap())
        .collect();
    
    // Build vocabulary from the texts
    let vocabulary = Vocabulary::build_from_texts(&texts, VOCAB_SIZE);
    
    // Create raw text items
    let mut items = Vec::new();
    for (text, label) in texts.iter().zip(label_indices.iter()) {
        // Apply data augmentation if enabled
        let processed_text = if USE_AUGMENTATION {
            apply_augmentation(text)
        } else {
            text.clone()
        };
        
        // Tokenize the text
        let tokens = tokenize_and_convert(&processed_text, &vocabulary);
        
        items.push(RawTextItem {
            text: processed_text,
            tokens,
            label: *label,
        });
    }
    
    // If no class names were found, use the default ones
    let class_names = if class_names.is_empty() {
        CLASS_NAMES.iter().map(|&s| s.to_string()).collect()
    } else {
        class_names
    };
    
    // Create and return the dataset
    Ok(TextDataset::new(items, vocabulary, class_names))
}

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

// Tokenize text and convert to token IDs
fn tokenize_and_convert(text: &str, vocabulary: &Vocabulary) -> Vec<usize> {
    tokenize(text).iter()
        .map(|token| vocabulary.token_to_id(token))
        .collect()
}

// Apply data augmentation to a text
fn apply_augmentation(text: &str) -> String {
    // CUSTOMIZE HERE: Add or modify augmentation techniques
    
    let mut rng = thread_rng();
    let tokens = tokenize(text);
    
    if tokens.is_empty() {
        return text.clone();
    }
    
    let mut result = tokens.clone();
    
    // Random word deletion
    if RANDOM_DELETION && rng.gen_bool(0.3) {
        let delete_count = (result.len() as f32 * 0.1).max(1.0) as usize;
        for _ in 0..delete_count {
            if result.len() > 1 { // Keep at least one token
                let idx = rng.gen_range(0..result.len());
                result.remove(idx);
            }
        }
    }
    
    // Random word swap
    if RANDOM_SWAP && rng.gen_bool(0.3) {
        let swap_count = (result.len() as f32 * 0.1).max(1.0) as usize;
        for _ in 0..swap_count {
            if result.len() > 1 {
                let idx1 = rng.gen_range(0..result.len());
                let idx2 = rng.gen_range(0..result.len());
                result.swap(idx1, idx2);
            }
        }
    }
    
    // Random synonym replacement (simplified version)
    if RANDOM_SYNONYM && rng.gen_bool(0.3) {
        // This is a simplified version that just replaces with similar words
        // In a real implementation, you would use a thesaurus or WordNet
        let synonyms: HashMap<&str, Vec<&str>> = [
            ("good", vec!["great", "excellent", "fine", "nice"]),
            ("bad", vec!["poor", "terrible", "awful", "horrible"]),
            ("happy", vec!["glad", "joyful", "pleased", "delighted"]),
            ("sad", vec!["unhappy", "depressed", "gloomy", "miserable"]),
        ].iter().cloned().collect();
        
        for i in 0..result.len() {
            if rng.gen_bool(0.1) {
                if let Some(syn_list) = synonyms.get(result[i].as_str()) {
                    if !syn_list.is_empty() {
                        let syn = syn_list.choose(&mut rng).unwrap();
                        result[i] = syn.to_string();
                    }
                }
            }
        }
    }
    
    // Join tokens back into a string
    result.join(" ")
}

// Create a sample dataset with synthetic texts for testing
pub fn create_sample_dataset() -> TextDataset {
    // CUSTOMIZE HERE: Modify the synthetic data generation
    
    // Sample texts for each sentiment class
    let samples = [
        // Negative samples
        vec![
            "This product is terrible and I regret buying it.",
            "Worst experience ever, would not recommend.",
            "I'm very disappointed with the quality.",
            "The customer service was awful and unhelpful.",
            "This is a complete waste of money.",
        ],
        // Neutral samples
        vec![
            "The product works as expected, nothing special.",
            "It's okay for the price, but could be better.",
            "Delivery was on time, product is standard.",
            "I have mixed feelings about this purchase.",
            "It's neither great nor terrible, just average.",
        ],
        // Positive samples
        vec![
            "I love this product, it exceeded my expectations!",
            "Great quality and excellent customer service.",
            "This is exactly what I needed, very satisfied.",
            "Highly recommend, works perfectly for me.",
            "Best purchase I've made this year, very happy.",
        ],
    ];
    
    let mut texts = Vec::new();
    let mut labels = Vec::new();
    
    // Add all sample texts to our dataset
    for (label, class_samples) in samples.iter().enumerate() {
        for &text in class_samples {
            texts.push(text.to_string());
            labels.push(label);
        }
    }
    
    // Build vocabulary from the texts
    let vocabulary = Vocabulary::build_from_texts(&texts, VOCAB_SIZE);
    
    // Create raw text items
    let mut items = Vec::new();
    for (text, label) in texts.iter().zip(labels.iter()) {
        // Tokenize the text
        let tokens = tokenize_and_convert(text, &vocabulary);
        
        items.push(RawTextItem {
            text: text.clone(),
            tokens,
            label: *label,
        });
    }
    
    // Create class names
    let class_names = CLASS_NAMES.iter().map(|&s| s.to_string()).collect();
    
    // Create and return the dataset
    TextDataset::new(items, vocabulary, class_names)
}
