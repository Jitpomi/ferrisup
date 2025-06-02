use anyhow::Result;
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use ndarray::{Array1, Array2, Ix1};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use std::fs::File;
use std::path::Path;

use crate::data_utils;

pub fn run_regression_example() -> Result<()> {
    // Simple regression example with Linfa 0.7.1
    println!("Linfa 0.7.1 Linear Regression Example");
    
    // Check for data files in different formats
    let csv_path = Path::new("data/sample_regression.csv");
    let json_path = Path::new("data/sample_regression.json");
    
    let dataset = if csv_path.exists() {
        println!("Loading data from CSV file: {}", csv_path.display());
        data_utils::load_csv_dataset(csv_path)?
    } else if json_path.exists() {
        println!("Loading data from JSON file: {}", json_path.display());
        data_utils::load_json_dataset(json_path)?
    } else {
        println!("No data files found, using synthetic data");
        data_utils::create_synthetic_regression_dataset()?
    };
    
    // Split into train and test sets with a random seed for reproducibility
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let (train, test) = dataset.shuffle(&mut rng).split_with_ratio(0.7);
    
    println!("Training dataset: {} samples", train.nsamples());
    println!("Testing dataset: {} samples", test.nsamples());
    
    // Create and train the model
    println!("Training LinearRegression model...");
    let model = LinearRegression::default()
        .fit(&train)?;
    
    // Make predictions
    println!("Making predictions...");
    let predictions = model.predict(test.records());
    
    // Print predictions vs actual values
    println!("Predictions vs Actual:");
    for (i, pred) in predictions.iter().enumerate() {
        let actual = test.targets().get(i).unwrap();
        println!("  Predicted: {:.2}, Actual: {:.2}", pred, actual);
    }
    
    // Calculate metrics
    let mse = predictions.iter()
        .zip(test.targets().iter())
        .map(|(&p, &a)| (p - a) * (p - a))
        .sum::<f64>() / predictions.len() as f64;
    
    println!("Mean Squared Error: {:.4}", mse);
    
    // Print model parameters
    println!("Model parameters:");
    println!("  Parameters shape: {:?}", model.params().shape());
    println!("  Parameters values: {:?}", model.params());
    
    // For simple linear regression, we can interpret the parameters
    if model.params().len() == 1 {
        let m = model.params()[0]; // Coefficient (slope)
        let b = model.intercept(); // Intercept
        
        println!("  Estimated coefficient (m): {:.4}", m);
        println!("  Estimated intercept (b): {:.4}", b);
        println!("  Estimated model equation: y = {:.4} * x + {:.4}", m, b);
        
        // Make predictions on new data
        println!("\nPredicting on new data:");
        let new_x_values = [0.5, 7.0, 10.0];
        
        for &x in &new_x_values {
            let y = m * x + b;
            println!("  x = {:.1}, predicted y = {:.2}", x, y);
        }
    }
    
    Ok(())
}

// Function to create a synthetic dataset
fn create_synthetic_dataset() -> Result<Dataset<f64, f64, Ix1>> {
    // Create a simple dataset
    let features = Array2::from_shape_vec(
        (6, 1),
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    )?;
    
    // y = 2*x + 1 + noise
    let targets = Array1::from_vec(vec![3.1, 5.2, 7.0, 8.9, 10.8, 13.1]);
    
    // Create a dataset
    Ok(Dataset::new(features, targets))
}

// Function to load dataset from CSV
fn load_csv_dataset(path: &Path) -> Result<Dataset<f64, f64, Ix1>> {
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    // Skip header row
    for result in reader.records() {
        let record = result?;
        
        if record.len() >= 2 {
            // First column is x (feature)
            let x = record[0].parse::<f64>()?;
            features_data.push(x);
            
            // Second column is y (target)
            let y = record[1].parse::<f64>()?;
            targets_data.push(y);
        }
    }
    
    // Calculate number of samples
    let num_samples = targets_data.len();
    
    // Create feature array (each sample has 1 feature)
    let features = Array2::from_shape_vec((num_samples, 1), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}
