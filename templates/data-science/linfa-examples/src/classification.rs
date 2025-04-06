use anyhow::Result;
use linfa::prelude::*;
use linfa_logistic::LogisticRegression;
use ndarray::{Array1, Array2, Ix1};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use std::fs::File;
use std::path::Path;
use csv;

pub fn run_logistic_regression_example() -> Result<()> {
    // LogisticRegression classification example with Linfa 0.7.1
    println!("Linfa 0.7.1 LogisticRegression Classification Example");
    
    // Check if CSV file exists
    let csv_path = Path::new("data/sample_classification.csv");
    let dataset = if csv_path.exists() {
        println!("Loading data from CSV file: {}", csv_path.display());
        load_csv_dataset(csv_path)?
    } else {
        println!("CSV file not found, using synthetic data");
        create_synthetic_dataset()?
    };
    
    // Split into train and test sets with a random seed for reproducibility
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let (train, test) = dataset.shuffle(&mut rng).split_with_ratio(0.5);
    
    println!("Training dataset: {} samples", train.nsamples());
    println!("Testing dataset: {} samples", test.nsamples());
    
    // Create and train the model
    println!("Training LogisticRegression model...");
    let model = LogisticRegression::default()
        .max_iterations(100)
        .fit(&train)?;
    
    // Make predictions
    println!("Making predictions...");
    let predictions = model.predict(test.records());
    println!("Predictions: {:?}", predictions);
    
    // Calculate accuracy
    let cm = predictions.confusion_matrix(&test)?;
    println!("Confusion Matrix:");
    println!("{:?}", cm);
    
    let accuracy = cm.accuracy();
    println!("Accuracy: {:.2}", accuracy);
    
    println!("\nThis example demonstrates a complete classification workflow using LogisticRegression in Linfa 0.7.1.");
    println!("It shows how to create a dataset, split it into training and testing sets, train a model, make predictions, and evaluate the results.");
    
    Ok(())
}

// Function to create a synthetic dataset
fn create_synthetic_dataset() -> Result<Dataset<f64, usize, Ix1>> {
    // Create a simple dataset for binary classification
    let features = Array2::from_shape_vec(
        (6, 2),
        vec![
            1.0, 2.0,  // Class 0
            1.0, 3.0,  // Class 0
            2.0, 2.0,  // Class 0
            3.0, 1.0,  // Class 1
            3.0, 3.0,  // Class 1
            4.0, 2.0,  // Class 1
        ],
    )?;
    
    let targets = Array1::from_vec(vec![0, 0, 0, 1, 1, 1]);
    
    // Create a dataset
    Ok(Dataset::new(features, targets))
}

// Function to load dataset from CSV
fn load_csv_dataset(path: &Path) -> Result<Dataset<f64, usize, Ix1>> {
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    for result in reader.records() {
        let record = result?;
        
        // Extract features (all columns except the last one)
        for i in 0..(record.len() - 1) {
            let value = record[i].parse::<f64>()?;
            features_data.push(value);
        }
        
        // Extract target (last column)
        let target = record[record.len() - 1].parse::<usize>()?;
        targets_data.push(target);
    }
    
    // Calculate number of samples and features
    let num_samples = targets_data.len();
    let num_features = if num_samples > 0 { features_data.len() / num_samples } else { 0 };
    
    // Create feature array
    let features = Array2::from_shape_vec((num_samples, num_features), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}
