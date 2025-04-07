// Data Handling for Value Prediction
// This file handles loading and processing CSV data

use burn::data::dataset::{Dataset, InMemDataset};
use burn::data::dataloader::batcher::Batcher;
use burn::tensor::{backend::Backend, Data, Tensor};
use std::path::Path;
use std::fs::File;
use csv::Reader;
use anyhow::Result;

// Regression Item - represents a single batch of regression data
#[derive(Clone, Debug)]
pub struct RegressionItem<B: Backend> {
    // Batch of input features - shape [batch_size, num_features]
    pub features: Tensor<B, 2>,
    // Batch of target values - shape [batch_size, 1]
    pub targets: Tensor<B, 2>,
}

// Regression Batcher - converts raw data into batches for the model
pub struct RegressionBatcher<B: Backend> {
    batch_size: usize,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> RegressionBatcher<B> {
    // Create a new batcher with the specified batch size
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

// Raw Regression item - represents a single example with features and target
#[derive(Clone, Debug)]
pub struct RawRegressionItem {
    // Input features (e.g., house size, number of bedrooms)
    pub features: Vec<f32>,
    // Target value to predict (e.g., house price)
    pub target: f32,
}

// Implement the Batcher trait for our RegressionBatcher
impl<B: Backend> Batcher<RawRegressionItem, RegressionItem<B>> for RegressionBatcher<B> {
    // Convert a batch of raw items into a processed RegressionItem
    fn batch(&self, items: Vec<RawRegressionItem>) -> RegressionItem<B> {
        // Number of items in this batch
        let batch_size = items.len();
        // Number of features per item
        let num_features = items[0].features.len();
        
        // Create tensors to hold the batch data
        let mut features_data = Data::new(
            vec![0.0; batch_size * num_features],
            [batch_size, num_features],
        );
        let mut targets_data = Data::new(
            vec![0.0; batch_size * 1],
            [batch_size, 1],
        );
        
        // Process each item in the batch
        for (i, item) in items.iter().enumerate() {
            // Set the target value
            targets_data.value_mut()[i] = item.target;
            
            // Copy the feature values
            for (j, &feature) in item.features.iter().enumerate() {
                features_data.value_mut()[i * num_features + j] = feature;
            }
        }
        
        // Create tensors from the data
        let features = Tensor::<B, 2>::from_data(features_data);
        let targets = Tensor::<B, 2>::from_data(targets_data);
        
        // Return the processed batch
        RegressionItem { features, targets }
    }
}

// Dataset structure to hold our regression data
pub struct RegressionDataset {
    // List of examples (features and targets)
    items: Vec<RawRegressionItem>,
    // Number of features per example
    num_features: usize,
}

impl RegressionDataset {
    // Create a new dataset with the given items and number of features
    pub fn new(items: Vec<RawRegressionItem>, num_features: usize) -> Self {
        Self { items, num_features }
    }
    
    // Get the number of features in this dataset
    pub fn num_features(&self) -> usize {
        self.num_features
    }
}

// Implement the Dataset trait for our RegressionDataset
impl Dataset<RawRegressionItem> for RegressionDataset {
    // Get the number of examples in the dataset
    fn len(&self) -> usize {
        self.items.len()
    }
    
    // Get a specific example by index
    fn get(&self, index: usize) -> Option<RawRegressionItem> {
        self.items.get(index).cloned()
    }
}

// Load a regression dataset from a CSV file
pub fn load_regression_dataset(path: &str) -> Result<RegressionDataset> {
    // Open the CSV file
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    
    // Vector to store our examples
    let mut items = Vec::new();
    // Number of features (will be determined from the first row)
    let mut num_features = 0;
    
    // Process each row in the CSV
    for result in rdr.records() {
        let record = result?;
        
        // Skip empty rows
        if record.len() == 0 {
            continue;
        }
        
        // The last column is the target value, all others are features
        let target_index = record.len() - 1;
        
        // Set the number of features based on the first row
        if num_features == 0 {
            num_features = target_index;
        }
        
        // Parse the target value
        let target = record[target_index].parse::<f32>()
            .map_err(|e| anyhow::anyhow!("Failed to parse target value: {}", e))?;
        
        // Parse the feature values
        let mut features = Vec::with_capacity(num_features);
        for i in 0..num_features {
            let feature = record[i].parse::<f32>()
                .map_err(|e| anyhow::anyhow!("Failed to parse feature {}: {}", i, e))?;
            features.push(feature);
        }
        
        // Add this example to our dataset
        items.push(RawRegressionItem { features, target });
    }
    
    // If no data was loaded, return an error
    if items.is_empty() {
        return Err(anyhow::anyhow!("No data loaded from CSV file"));
    }
    
    // Create and return the dataset
    Ok(RegressionDataset::new(items, num_features))
}

// Generate a synthetic dataset for testing purposes
pub fn generate_synthetic_dataset(num_samples: usize, num_features: usize) -> RegressionDataset {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Vector to store our examples
    let mut items = Vec::with_capacity(num_samples);
    
    // Generate random examples
    for _ in 0..num_samples {
        // Generate random feature values
        let features: Vec<f32> = (0..num_features)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();
        
        // Generate a target value based on a simple linear combination of features
        // plus some random noise
        let target = features.iter().enumerate()
            .map(|(i, &x)| (i as f32 + 1.0) * x) // Simple linear function
            .sum::<f32>()
            + rng.gen_range(-0.1..0.1); // Add some noise
        
        // Add this example to our dataset
        items.push(RawRegressionItem { features, target });
    }
    
    // Create and return the dataset
    RegressionDataset::new(items, num_features)
}

// Create a sample CSV file with synthetic data for testing
pub fn create_sample_csv(path: &str, num_samples: usize, num_features: usize) -> Result<()> {
    use std::io::Write;
    
    // Generate synthetic data
    let dataset = generate_synthetic_dataset(num_samples, num_features);
    
    // Create the CSV file
    let mut file = File::create(path)?;
    
    // Write header row
    let mut header = String::new();
    for i in 0..num_features {
        header.push_str(&format!("feature_{},", i));
    }
    header.push_str("target\n");
    file.write_all(header.as_bytes())?;
    
    // Write data rows
    for item in dataset.items {
        let mut row = String::new();
        for feature in item.features {
            row.push_str(&format!("{},", feature));
        }
        row.push_str(&format!("{}\n", item.target));
        file.write_all(row.as_bytes())?;
    }
    
    Ok(())
}
