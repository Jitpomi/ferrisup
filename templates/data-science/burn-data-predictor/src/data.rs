// Data Handling for Numerical Prediction
// This file handles loading and processing numerical data

use burn::data::dataset::{Dataset, InMemDataset};
use burn::data::dataloader::batcher::Batcher;
use burn::tensor::{backend::Backend, Data, Tensor};
use std::path::Path;
use std::fs;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use rand::{Rng, thread_rng};
use serde::{Serialize, Deserialize};

// Import our configuration parameters
use crate::config::{
    FEATURE_COLUMNS, TARGET_COLUMN, NORMALIZE_FEATURES, NORMALIZE_TARGET,
    TEST_SPLIT_RATIO, USE_AUGMENTATION, NOISE_LEVEL, FEATURE_DROPOUT
};

// Data Item - represents a single batch of numerical data
#[derive(Clone, Debug)]
pub struct DataItem<B: Backend> {
    // Batch of features - shape [batch_size, num_features]
    pub features: Tensor<B, 2>,
    // Batch of targets - shape [batch_size, num_targets]
    pub targets: Tensor<B, 2>,
}

// Data Batcher - converts raw numerical data into batches for the model
pub struct DataBatcher<B: Backend> {
    batch_size: usize,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> DataBatcher<B> {
    // Create a new batcher with the specified batch size
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

// Raw Data item - represents a single example with features and target
#[derive(Clone, Debug)]
pub struct RawDataItem {
    // Feature values
    pub features: Vec<f32>,
    // Target value(s)
    pub targets: Vec<f32>,
}

// Implement the Batcher trait for our DataBatcher
impl<B: Backend> Batcher<RawDataItem, DataItem<B>> for DataBatcher<B> {
    // Convert a batch of raw items into a processed DataItem
    fn batch(&self, items: Vec<RawDataItem>) -> DataItem<B> {
        // Number of items in this batch
        let batch_size = items.len();
        
        // Get the number of features and targets
        let num_features = if !items.is_empty() { items[0].features.len() } else { 0 };
        let num_targets = if !items.is_empty() { items[0].targets.len() } else { 0 };
        
        // Create tensors to hold the batch data
        let mut features_data = Data::new(
            vec![0.0; batch_size * num_features],
            [batch_size, num_features],
        );
        
        let mut targets_data = Data::new(
            vec![0.0; batch_size * num_targets],
            [batch_size, num_targets],
        );
        
        // Process each item in the batch
        for (i, item) in items.iter().enumerate() {
            // Copy the feature values to the batch tensor
            for (j, &value) in item.features.iter().enumerate() {
                features_data.value_mut()[i * num_features + j] = value;
            }
            
            // Copy the target values to the batch tensor
            for (j, &value) in item.targets.iter().enumerate() {
                targets_data.value_mut()[i * num_targets + j] = value;
            }
        }
        
        // Create tensors from the data
        let features = Tensor::<B, 2>::from_data(features_data);
        let targets = Tensor::<B, 2>::from_data(targets_data);
        
        // Return the processed batch
        DataItem { features, targets }
    }
}

// Statistics for normalization
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataStats {
    // Feature statistics
    pub feature_means: Vec<f32>,
    pub feature_stds: Vec<f32>,
    // Target statistics
    pub target_means: Vec<f32>,
    pub target_stds: Vec<f32>,
    // Column names
    pub feature_columns: Vec<String>,
    pub target_columns: Vec<String>,
}

impl DataStats {
    // Create new empty statistics
    pub fn new(num_features: usize, num_targets: usize) -> Self {
        Self {
            feature_means: vec![0.0; num_features],
            feature_stds: vec![1.0; num_features],
            target_means: vec![0.0; num_targets],
            target_stds: vec![1.0; num_targets],
            feature_columns: FEATURE_COLUMNS.iter().map(|&s| s.to_string()).collect(),
            target_columns: vec![TARGET_COLUMN.to_string()],
        }
    }
    
    // Normalize features
    pub fn normalize_features(&self, features: &mut [f32]) {
        if NORMALIZE_FEATURES {
            for i in 0..features.len() {
                features[i] = (features[i] - self.feature_means[i]) / self.feature_stds[i];
            }
        }
    }
    
    // Normalize targets
    pub fn normalize_targets(&self, targets: &mut [f32]) {
        if NORMALIZE_TARGET {
            for i in 0..targets.len() {
                targets[i] = (targets[i] - self.target_means[i]) / self.target_stds[i];
            }
        }
    }
    
    // Denormalize targets (for predictions)
    pub fn denormalize_targets(&self, targets: &mut [f32]) {
        if NORMALIZE_TARGET {
            for i in 0..targets.len() {
                targets[i] = targets[i] * self.target_stds[i] + self.target_means[i];
            }
        }
    }
    
    // Save statistics to a file
    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    // Load statistics from a file
    pub fn load(path: &str) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let stats = serde_json::from_str(&json)?;
        Ok(stats)
    }
}

// Dataset structure to hold our numerical data
pub struct NumericalDataset {
    // List of examples (features and targets)
    items: Vec<RawDataItem>,
    // Statistics for normalization
    stats: DataStats,
}

impl NumericalDataset {
    // Create a new dataset with the given items and statistics
    pub fn new(items: Vec<RawDataItem>, stats: DataStats) -> Self {
        Self { items, stats }
    }
    
    // Get the statistics
    pub fn stats(&self) -> &DataStats {
        &self.stats
    }
    
    // Get the number of features
    pub fn num_features(&self) -> usize {
        if !self.items.is_empty() {
            self.items[0].features.len()
        } else {
            0
        }
    }
    
    // Get the number of targets
    pub fn num_targets(&self) -> usize {
        if !self.items.is_empty() {
            self.items[0].targets.len()
        } else {
            0
        }
    }
    
    // Split the dataset into training and testing sets
    pub fn split_train_test(&self, test_ratio: f32) -> (InMemDataset<RawDataItem>, InMemDataset<RawDataItem>) {
        let test_size = (self.items.len() as f32 * test_ratio).round() as usize;
        let test_size = test_size.min(self.items.len());
        
        let train_items = self.items[test_size..].to_vec();
        let test_items = self.items[0..test_size].to_vec();
        
        (
            InMemDataset::new(train_items),
            InMemDataset::new(test_items),
        )
    }
}

// Implement the Dataset trait for our NumericalDataset
impl Dataset<RawDataItem> for NumericalDataset {
    // Get the number of examples in the dataset
    fn len(&self) -> usize {
        self.items.len()
    }
    
    // Get a specific example by index
    fn get(&self, index: usize) -> Option<RawDataItem> {
        self.items.get(index).cloned()
    }
}

// Load a numerical dataset from a CSV file
pub fn load_csv_dataset(file_path: &str) -> Result<NumericalDataset> {
    // CUSTOMIZE HERE: Modify how CSV data is loaded
    
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    
    // Get headers
    let headers = csv_reader.headers()?.clone();
    
    // Find column indices
    let mut feature_indices = Vec::new();
    for &feature_col in FEATURE_COLUMNS.iter() {
        if let Some(idx) = headers.iter().position(|h| h == feature_col) {
            feature_indices.push(idx);
        } else {
            return Err(anyhow!("Feature column '{}' not found in CSV", feature_col));
        }
    }
    
    let target_index = headers.iter().position(|h| h == TARGET_COLUMN)
        .ok_or_else(|| anyhow!("Target column '{}' not found in CSV", TARGET_COLUMN))?;
    
    // Read all data
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    for result in csv_reader.records() {
        let record = result?;
        
        // Extract features
        let mut features = Vec::new();
        for &idx in &feature_indices {
            let value = record.get(idx)
                .ok_or_else(|| anyhow!("Missing feature value"))?
                .parse::<f32>()?;
            features.push(value);
        }
        
        // Extract target
        let target = record.get(target_index)
            .ok_or_else(|| anyhow!("Missing target value"))?
            .parse::<f32>()?;
        
        features_data.push(features);
        targets_data.push(vec![target]);
    }
    
    // Calculate statistics
    let num_features = if !features_data.is_empty() { features_data[0].len() } else { 0 };
    let num_targets = 1; // Single target for regression
    
    let mut stats = DataStats::new(num_features, num_targets);
    
    // Calculate means
    if !features_data.is_empty() {
        for j in 0..num_features {
            let sum: f32 = features_data.iter().map(|f| f[j]).sum();
            stats.feature_means[j] = sum / features_data.len() as f32;
        }
        
        let sum: f32 = targets_data.iter().map(|t| t[0]).sum();
        stats.target_means[0] = sum / targets_data.len() as f32;
    }
    
    // Calculate standard deviations
    if !features_data.is_empty() {
        for j in 0..num_features {
            let variance: f32 = features_data.iter()
                .map(|f| (f[j] - stats.feature_means[j]).powi(2))
                .sum::<f32>() / features_data.len() as f32;
            stats.feature_stds[j] = variance.sqrt().max(1e-6); // Avoid division by zero
        }
        
        let variance: f32 = targets_data.iter()
            .map(|t| (t[0] - stats.target_means[0]).powi(2))
            .sum::<f32>() / targets_data.len() as f32;
        stats.target_stds[0] = variance.sqrt().max(1e-6); // Avoid division by zero
    }
    
    // Create raw data items with normalized values
    let mut items = Vec::new();
    for (features, targets) in features_data.iter().zip(targets_data.iter()) {
        let mut normalized_features = features.clone();
        let mut normalized_targets = targets.clone();
        
        stats.normalize_features(&mut normalized_features);
        stats.normalize_targets(&mut normalized_targets);
        
        items.push(RawDataItem {
            features: normalized_features,
            targets: normalized_targets,
        });
    }
    
    // Create and return the dataset
    Ok(NumericalDataset::new(items, stats))
}

// Apply data augmentation to a data item
fn apply_augmentation(item: &RawDataItem) -> RawDataItem {
    // CUSTOMIZE HERE: Add or modify augmentation techniques
    
    if !USE_AUGMENTATION {
        return item.clone();
    }
    
    let mut rng = thread_rng();
    let mut features = item.features.clone();
    let targets = item.targets.clone();
    
    // Add Gaussian noise to features
    if rng.gen_bool(0.5) {
        for feature in &mut features {
            *feature += rng.gen_range(-NOISE_LEVEL..NOISE_LEVEL);
        }
    }
    
    // Randomly zero out some features (feature dropout)
    if rng.gen_bool(0.3) {
        for feature in &mut features {
            if rng.gen_bool(FEATURE_DROPOUT as f64) {
                *feature = 0.0;
            }
        }
    }
    
    RawDataItem { features, targets }
}

// Create a sample dataset with synthetic data for testing
pub fn create_sample_dataset() -> NumericalDataset {
    // CUSTOMIZE HERE: Modify the synthetic data generation
    
    let num_samples = 1000;
    let num_features = 5;
    let num_targets = 1;
    
    let mut rng = thread_rng();
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    // Generate synthetic data
    // This is a simple linear model with some noise
    for _ in 0..num_samples {
        let mut features = Vec::new();
        for _ in 0..num_features {
            features.push(rng.gen_range(-10.0..10.0));
        }
        
        // Target is a linear combination of features plus noise
        let target = features.iter().enumerate()
            .map(|(i, &f)| f * (i as f32 + 1.0))
            .sum::<f32>() + rng.gen_range(-5.0..5.0);
        
        features_data.push(features);
        targets_data.push(vec![target]);
    }
    
    // Calculate statistics
    let mut stats = DataStats::new(num_features, num_targets);
    
    // Calculate means
    for j in 0..num_features {
        let sum: f32 = features_data.iter().map(|f| f[j]).sum();
        stats.feature_means[j] = sum / num_samples as f32;
    }
    
    let sum: f32 = targets_data.iter().map(|t| t[0]).sum();
    stats.target_means[0] = sum / num_samples as f32;
    
    // Calculate standard deviations
    for j in 0..num_features {
        let variance: f32 = features_data.iter()
            .map(|f| (f[j] - stats.feature_means[j]).powi(2))
            .sum::<f32>() / num_samples as f32;
        stats.feature_stds[j] = variance.sqrt().max(1e-6);
    }
    
    let variance: f32 = targets_data.iter()
        .map(|t| (t[0] - stats.target_means[0]).powi(2))
        .sum::<f32>() / num_samples as f32;
    stats.target_stds[0] = variance.sqrt().max(1e-6);
    
    // Create raw data items with normalized values
    let mut items = Vec::new();
    for (features, targets) in features_data.iter().zip(targets_data.iter()) {
        let mut normalized_features = features.clone();
        let mut normalized_targets = targets.clone();
        
        stats.normalize_features(&mut normalized_features);
        stats.normalize_targets(&mut normalized_targets);
        
        items.push(RawDataItem {
            features: normalized_features,
            targets: normalized_targets,
        });
    }
    
    // Create and return the dataset
    NumericalDataset::new(items, stats)
}

// Generate a simple housing price dataset
pub fn create_housing_dataset() -> NumericalDataset {
    // CUSTOMIZE HERE: Modify the housing data generation
    
    let num_samples = 1000;
    let num_features = 5;
    let num_targets = 1;
    
    let mut rng = thread_rng();
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    // Feature names (for reference)
    // FEATURE_COLUMNS = ["sqft", "bedrooms", "bathrooms", "age", "location_score"]
    
    // Generate synthetic housing data
    for _ in 0..num_samples {
        // Square footage (800 to 4000)
        let sqft = rng.gen_range(800.0..4000.0);
        
        // Bedrooms (1 to 6)
        let bedrooms = rng.gen_range(1.0..6.0).round();
        
        // Bathrooms (1 to 4)
        let bathrooms = rng.gen_range(1.0..4.0).round() * 0.5;
        
        // Age of house (0 to 100 years)
        let age = rng.gen_range(0.0..100.0);
        
        // Location score (1 to 10)
        let location = rng.gen_range(1.0..10.0);
        
        // Features
        let features = vec![sqft, bedrooms, bathrooms, age, location];
        
        // Target price (based on features with some randomness)
        let base_price = 50000.0;
        let sqft_factor = sqft * 100.0;
        let bedroom_factor = bedrooms * 15000.0;
        let bathroom_factor = bathrooms * 25000.0;
        let age_factor = -age * 500.0;
        let location_factor = location * 30000.0;
        
        let price = base_price + sqft_factor + bedroom_factor + bathroom_factor + 
                   age_factor + location_factor + rng.gen_range(-50000.0..50000.0);
        
        features_data.push(features);
        targets_data.push(vec![price]);
    }
    
    // Calculate statistics
    let mut stats = DataStats::new(num_features, num_targets);
    
    // Set feature column names
    stats.feature_columns = vec![
        "sqft".to_string(),
        "bedrooms".to_string(),
        "bathrooms".to_string(),
        "age".to_string(),
        "location_score".to_string(),
    ];
    
    stats.target_columns = vec!["price".to_string()];
    
    // Calculate means
    for j in 0..num_features {
        let sum: f32 = features_data.iter().map(|f| f[j]).sum();
        stats.feature_means[j] = sum / num_samples as f32;
    }
    
    let sum: f32 = targets_data.iter().map(|t| t[0]).sum();
    stats.target_means[0] = sum / num_samples as f32;
    
    // Calculate standard deviations
    for j in 0..num_features {
        let variance: f32 = features_data.iter()
            .map(|f| (f[j] - stats.feature_means[j]).powi(2))
            .sum::<f32>() / num_samples as f32;
        stats.feature_stds[j] = variance.sqrt().max(1e-6);
    }
    
    let variance: f32 = targets_data.iter()
        .map(|t| (t[0] - stats.target_means[0]).powi(2))
        .sum::<f32>() / num_samples as f32;
    stats.target_stds[0] = variance.sqrt().max(1e-6);
    
    // Create raw data items with normalized values
    let mut items = Vec::new();
    for (features, targets) in features_data.iter().zip(targets_data.iter()) {
        let mut normalized_features = features.clone();
        let mut normalized_targets = targets.clone();
        
        stats.normalize_features(&mut normalized_features);
        stats.normalize_targets(&mut normalized_targets);
        
        items.push(RawDataItem {
            features: normalized_features,
            targets: normalized_targets,
        });
    }
    
    // Create and return the dataset
    NumericalDataset::new(items, stats)
}
