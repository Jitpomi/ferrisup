use anyhow::Result;
use linfa::prelude::*;
use linfa_logistic::LogisticRegression;
use ndarray::{Array1, Array2, Ix1};
use rand::SeedableRng;
use std::path::Path;
use std::fs::File;

use crate::data_utils;

pub fn run_logistic_regression_example() -> Result<()> {
    println!("Linfa 0.7.1 Logistic Regression Example");
    
    // Check for data files in different formats
    let csv_path = Path::new("data/sample_classification.csv");
    let json_path = Path::new("data/sample_classification.json");
    
    let dataset = if csv_path.exists() {
        println!("Loading data from CSV file: {}", csv_path.display());
        load_classification_dataset(csv_path)?
    } else if json_path.exists() {
        println!("Loading data from JSON file: {}", json_path.display());
        load_classification_dataset(json_path)?
    } else {
        println!("No data files found, using synthetic data");
        create_synthetic_classification_dataset()?
    };
    
    println!("Dataset shape: [{}, {}]", dataset.nsamples(), dataset.nfeatures());
    println!("Number of samples: {}", dataset.nsamples());
    
    // Split dataset into training and testing sets
    let (train, test) = dataset.split_with_ratio(0.8);
    
    println!("Training set size: {}", train.nsamples());
    println!("Testing set size: {}", test.nsamples());
    
    // Train a logistic regression model
    let model = LogisticRegression::default()
        .max_iterations(100)
        .fit(&train)?;
    
    println!("Model trained successfully");
    
    // Make predictions on the test set
    let predictions = model.predict(&test);
    
    // Calculate accuracy
    let cm = confusion_matrix(&predictions, test.targets())?;
    let accuracy = cm.accuracy();
    
    println!("Confusion Matrix:\n{}", cm);
    println!("Accuracy: {:.2}%", accuracy * 100.0);
    
    Ok(())
}

// Function to create a synthetic classification dataset
fn create_synthetic_classification_dataset() -> Result<Dataset<f64, usize, Ix1>> {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let num_classes = 2;
    let samples_per_class = 50;
    let num_samples = num_classes * samples_per_class;
    
    let mut features_data = Vec::with_capacity(num_samples * 2);
    let mut targets_data = Vec::with_capacity(num_samples);
    
    // Create two classes of points
    for class in 0..num_classes {
        let x_center = if class == 0 { 1.0 } else { 5.0 };
        let y_center = if class == 0 { 1.0 } else { 5.0 };
        
        for _ in 0..samples_per_class {
            // Add some noise
            let x = x_center + rng.gen_range(-1.0..1.0);
            let y = y_center + rng.gen_range(-1.0..1.0);
            
            features_data.push(x);
            features_data.push(y);
            targets_data.push(class);
        }
    }
    
    // Create feature array
    let features = Array2::from_shape_vec((num_samples, 2), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}

// Function to load classification dataset from file (auto-detect format)
fn load_classification_dataset(path: &Path) -> Result<Dataset<f64, usize, Ix1>> {
    let format = data_utils::detect_file_format(path)?;
    
    match format {
        "csv" => load_csv_classification_dataset(path),
        "json" => load_json_classification_dataset(path),
        _ => Err(anyhow::anyhow!("Unsupported file format: {}", format)),
    }
}

// Function to load classification dataset from CSV
fn load_csv_classification_dataset(path: &Path) -> Result<Dataset<f64, usize, Ix1>> {
    use csv;
    
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    for result in reader.records() {
        let record = result?;
        
        if record.len() >= 3 {
            // First two columns are features
            let x = record[0].parse::<f64>()?;
            let y = record[1].parse::<f64>()?;
            
            // Last column is target
            let target = record[2].parse::<usize>()?;
            
            features_data.push(x);
            features_data.push(y);
            targets_data.push(target);
        }
    }
    
    // Calculate number of samples
    let num_samples = targets_data.len();
    
    // Create feature array (each sample has 2 features)
    let features = Array2::from_shape_vec((num_samples, 2), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}

// Function to load classification dataset from JSON
fn load_json_classification_dataset(path: &Path) -> Result<Dataset<f64, usize, Ix1>> {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct ClassificationPoint {
        x: f64,
        y: f64,
        target: usize,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    struct ClassificationDataSet {
        data: Vec<ClassificationPoint>,
    }
    
    let file = File::open(path)?;
    let dataset: ClassificationDataSet = serde_json::from_reader(file)?;
    
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    for point in dataset.data {
        features_data.push(point.x);
        features_data.push(point.y);
        targets_data.push(point.target);
    }
    
    // Calculate number of samples
    let num_samples = targets_data.len();
    
    // Create feature array (each sample has 2 features)
    let features = Array2::from_shape_vec((num_samples, 2), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}

// Helper function to create a confusion matrix
fn confusion_matrix(predictions: &Array1<usize>, targets: &Array1<usize>) -> Result<ConfusionMatrix<usize>> {
    // Find unique classes
    let mut classes = Vec::new();
    
    for &target in targets.iter() {
        if !classes.contains(&target) {
            classes.push(target);
        }
    }
    
    for &pred in predictions.iter() {
        if !classes.contains(&pred) {
            classes.push(pred);
        }
    }
    
    // Sort classes for consistent output
    classes.sort();
    
    // Create confusion matrix
    let mut cm = ConfusionMatrix::new(classes)?;
    
    // Fill confusion matrix
    for (pred, actual) in predictions.iter().zip(targets.iter()) {
        cm.increment(*pred, *actual)?;
    }
    
    Ok(cm)
}

// Simple confusion matrix implementation
struct ConfusionMatrix<T> {
    classes: Vec<T>,
    matrix: Array2<usize>,
}

impl<T: std::cmp::PartialEq + std::fmt::Display + Copy + std::fmt::Debug> ConfusionMatrix<T> {
    fn new(classes: Vec<T>) -> Result<Self> {
        let n = classes.len();
        let matrix = Array2::zeros((n, n));
        
        Ok(ConfusionMatrix { classes, matrix })
    }
    
    fn increment(&mut self, predicted: T, actual: T) -> Result<()> {
        let pred_idx = self.classes.iter().position(|&c| c == predicted)
            .ok_or_else(|| anyhow::anyhow!("Unknown class: {:?}", predicted))?;
        
        let actual_idx = self.classes.iter().position(|&c| c == actual)
            .ok_or_else(|| anyhow::anyhow!("Unknown class: {:?}", actual))?;
        
        self.matrix[[pred_idx, actual_idx]] += 1;
        
        Ok(())
    }
    
    fn accuracy(&self) -> f64 {
        let total = self.matrix.sum();
        if total == 0 {
            return 0.0;
        }
        
        let mut correct = 0;
        for i in 0..self.classes.len() {
            correct += self.matrix[[i, i]];
        }
        
        correct as f64 / total as f64
    }
}

impl<T: std::fmt::Display + std::fmt::Debug> std::fmt::Display for ConfusionMatrix<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Header
        write!(f, "{:<10} | ", "classes")?;
        for class in &self.classes {
            write!(f, "{:<10} | ", class)?;
        }
        writeln!(f)?;
        
        // Rows
        for (i, class) in self.classes.iter().enumerate() {
            write!(f, "{:<10} | ", class)?;
            
            for j in 0..self.classes.len() {
                write!(f, "{:<10} | ", self.matrix[[i, j]])?;
            }
            
            writeln!(f)?;
        }
        
        Ok(())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ConfusionMatrix<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ConfusionMatrix")
    }
}
