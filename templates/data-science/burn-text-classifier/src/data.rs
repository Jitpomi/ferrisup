// Data Handling for Text Classification
// This file handles loading and processing text data for our classifier

use burn::data::dataset::{Dataset, InMemDataset};
use burn::data::dataloader::batcher::Batcher;
use burn::tensor::{backend::Backend, Data, Tensor};
use std::path::Path;
use std::fs::File;
use csv::Reader;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use crate::tokenizer::Tokenizer;

// Text Item - represents a single batch of text data
#[derive(Clone, Debug)]
pub struct TextItem<B: Backend> {
    // Batch of token sequences - shape [batch_size, seq_len]
    pub tokens: Tensor<B, 2, usize>,
    // Batch of labels - shape [batch_size]
    pub targets: Tensor<B, 1, usize>,
}

// Text Batcher - converts raw data into batches for the model
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
    // Tokenized text (sequence of token IDs)
    pub tokens: Vec<usize>,
    // Label (class index)
    pub label: usize,
}

// Implement the Batcher trait for our TextBatcher
impl<B: Backend> Batcher<RawTextItem, TextItem<B>> for TextBatcher<B> {
    // Convert a batch of raw items into a processed TextItem
    fn batch(&self, items: Vec<RawTextItem>) -> TextItem<B> {
        // Number of items in this batch
        let batch_size = items.len();
        
        // Find the maximum sequence length in this batch
        let max_seq_len = items.iter()
            .map(|item| item.tokens.len())
            .max()
            .unwrap_or(1);
        
        // Create tensors to hold the batch data
        let mut tokens_data = Data::new(
            vec![0; batch_size * max_seq_len],
            [batch_size, max_seq_len],
        );
        let mut targets_data = Data::new(vec![0; batch_size], [batch_size]);
        
        // Process each item in the batch
        for (i, item) in items.iter().enumerate() {
            // Set the target (label)
            targets_data.value_mut()[i] = item.label;
            
            // Copy the token IDs (padding with zeros if needed)
            for (j, &token) in item.tokens.iter().enumerate() {
                if j < max_seq_len {
                    tokens_data.value_mut()[i * max_seq_len + j] = token;
                }
            }
        }
        
        // Create tensors from the data
        let tokens = Tensor::<B, 2, usize>::from_data(tokens_data);
        let targets = Tensor::<B, 1, usize>::from_data(targets_data);
        
        // Return the processed batch
        TextItem { tokens, targets }
    }
}

// Text Dataset - holds our collection of texts and their labels
pub struct TextDataset {
    // List of text items
    items: Vec<RawTextItem>,
    // Mapping from class index to class name
    class_map: HashMap<usize, String>,
    // Mapping from class name to class index
    class_index_map: HashMap<String, usize>,
}

impl TextDataset {
    // Create a new dataset with the given items and class maps
    pub fn new(
        items: Vec<RawTextItem>,
        class_map: HashMap<usize, String>,
        class_index_map: HashMap<String, usize>,
    ) -> Self {
        Self {
            items,
            class_map,
            class_index_map,
        }
    }
    
    // Get the number of classes in the dataset
    pub fn num_classes(&self) -> usize {
        self.class_map.len()
    }
    
    // Get the list of class names
    pub fn class_names(&self) -> Vec<String> {
        let mut names = vec!["".to_string(); self.class_map.len()];
        for (idx, name) in &self.class_map {
            names[*idx] = name.clone();
        }
        names
    }
    
    // Get the class index for a given class name
    pub fn class_index(&self, class_name: &str) -> Option<usize> {
        self.class_index_map.get(class_name).copied()
    }
}

// Implement the Dataset trait for our TextDataset
impl Dataset<RawTextItem> for TextDataset {
    // Get the number of items in the dataset
    fn len(&self) -> usize {
        self.items.len()
    }
    
    // Get a specific item by index
    fn get(&self, index: usize) -> Option<RawTextItem> {
        self.items.get(index).cloned()
    }
}

// Load a text dataset from a CSV file
pub fn load_text_dataset(
    data_path: &str,
    text_col: &str,
    label_col: &str,
    tokenizer: &Tokenizer,
) -> Result<TextDataset> {
    // Open the CSV file
    let file = File::open(data_path)?;
    let mut rdr = Reader::from_reader(file);
    
    // Get the headers
    let headers = rdr.headers()?;
    
    // Find the indices of the text and label columns
    let text_idx = headers.iter().position(|h| h == text_col)
        .ok_or_else(|| anyhow!("Text column '{}' not found in CSV", text_col))?;
    
    let label_idx = headers.iter().position(|h| h == label_col)
        .ok_or_else(|| anyhow!("Label column '{}' not found in CSV", label_col))?;
    
    // Vector to store our examples
    let mut items = Vec::new();
    
    // Maps for class names and indices
    let mut class_map = HashMap::new();
    let mut class_index_map = HashMap::new();
    let mut next_class_idx = 0;
    
    // Process each row in the CSV
    for result in rdr.records() {
        let record = result?;
        
        // Skip empty rows
        if record.len() == 0 {
            continue;
        }
        
        // Get the text and label
        let text = record.get(text_idx)
            .ok_or_else(|| anyhow!("Text column index out of bounds"))?;
        
        let label_str = record.get(label_idx)
            .ok_or_else(|| anyhow!("Label column index out of bounds"))?;
        
        // Get or assign a class index for this label
        let label = if let Some(&idx) = class_index_map.get(label_str) {
            idx
        } else {
            let idx = next_class_idx;
            class_map.insert(idx, label_str.to_string());
            class_index_map.insert(label_str.to_string(), idx);
            next_class_idx += 1;
            idx
        };
        
        // Tokenize the text
        let tokens = tokenizer.tokenize(text);
        
        // Add this example to our dataset
        items.push(RawTextItem { tokens, label });
    }
    
    // If no data was loaded, return an error
    if items.is_empty() {
        return Err(anyhow!("No data loaded from CSV file"));
    }
    
    println!("Loaded {} examples with {} classes", items.len(), class_map.len());
    
    // Create and return the dataset
    Ok(TextDataset::new(items, class_map, class_index_map))
}

// Generate a synthetic dataset for testing purposes
pub fn generate_synthetic_dataset(num_samples: usize, num_classes: usize) -> TextDataset {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Vector to store our examples
    let mut items = Vec::with_capacity(num_samples);
    
    // Maps for class names and indices
    let mut class_map = HashMap::new();
    let mut class_index_map = HashMap::new();
    
    // Create class names
    for i in 0..num_classes {
        let class_name = format!("class_{}", i);
        class_map.insert(i, class_name.clone());
        class_index_map.insert(class_name, i);
    }
    
    // Generate random examples
    for _ in 0..num_samples {
        // Generate a random sequence of tokens (1-20 tokens)
        let seq_len = rng.gen_range(1..=20);
        let tokens: Vec<usize> = (0..seq_len)
            .map(|_| rng.gen_range(1..1000))
            .collect();
        
        // Generate a random class
        let label = rng.gen_range(0..num_classes);
        
        // Add this example to our dataset
        items.push(RawTextItem { tokens, label });
    }
    
    // Create and return the dataset
    TextDataset::new(items, class_map, class_index_map)
}
