use anyhow::Result;
use linfa::prelude::*;
use linfa_trees::DecisionTree;
use ndarray::{Array1, Array2, Ix1};
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use std::fs::File;
use std::path::Path;
use csv;

pub fn run_decision_tree_example() -> Result<()> {
    // Decision Tree classification example with Linfa 0.7.1
    println!("Linfa 0.7.1 Decision Tree Classification Example");
    
    // Check if CSV file exists
    let csv_path = Path::new("data/sample_classification.csv");
    let dataset = if csv_path.exists() {
        println!("Loading data from CSV file: {}", csv_path.display());
        load_csv_dataset(csv_path)?
    } else {
        println!("CSV file not found, using synthetic data");
        create_synthetic_dataset()?
    };
    
    println!("Dataset shape: {:?}", dataset.records().shape());
    println!("Number of samples: {}", dataset.nsamples());
    
    // Split into train and test sets with a random seed for reproducibility
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let (train, test) = dataset.shuffle(&mut rng).split_with_ratio(0.75);
    
    println!("Training dataset: {} samples", train.nsamples());
    println!("Testing dataset: {} samples", test.nsamples());
    
    // Create and train the Decision Tree model
    println!("Training Decision Tree model...");
    
    // Based on the memory about the correct API for Linfa 0.7.1
    let model = DecisionTree::params()
        .max_depth(Some(3))
        .fit(&train)?;
    
    // Make predictions
    println!("Making predictions...");
    let predictions = model.predict(test.records());
    
    // Print predictions vs actual values
    println!("Predictions vs Actual:");
    for (i, &pred) in predictions.iter().enumerate() {
        let actual = test.targets().get(i).unwrap();
        println!("  Predicted: {}, Actual: {}", pred, actual);
    }
    
    // Calculate and print confusion matrix and accuracy
    let cm = predictions.confusion_matrix(test.targets())?;
    println!("Confusion Matrix:\n");
    println!("{:?}", cm);
    println!("Accuracy: {:.2}", cm.accuracy());
    
    println!("\nThis example demonstrates a complete Decision Tree classification workflow using Linfa 0.7.1.");
    println!("It shows how to create a dataset, split it into training and testing sets, train a Decision Tree model,");
    println!("make predictions, and evaluate the results.");
    
    Ok(())
}

// Function to create a synthetic dataset
fn create_synthetic_dataset() -> Result<Dataset<f64, usize, Ix1>> {
    // Create a simple dataset for binary classification
    let features = Array2::from_shape_vec(
        (8, 2),
        vec![
            1.0, 2.0,  // Class 0
            1.5, 2.5,  // Class 0
            2.0, 3.0,  // Class 0
            1.0, 3.0,  // Class 0
            4.0, 1.0,  // Class 1
            4.5, 1.5,  // Class 1
            5.0, 1.0,  // Class 1
            5.5, 0.5,  // Class 1
        ],
    )?;
    
    let targets = Array1::from_vec(vec![0, 0, 0, 0, 1, 1, 1, 1]);
    
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
