use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use linfa::prelude::*;
use ndarray::{Array1, Array2, Axis};
use ndarray_rand::rand::SeedableRng;
use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;
use rand_xoshiro::Xoshiro256Plus;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

mod datasets;
mod evaluation;
mod models;

#[derive(Parser)]
#[command(author, version, about = "Linfa Machine Learning Examples")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run logistic regression classification example
    Classify,
    
    /// Run decision tree classification example
    Tree,
    
    /// Run linear regression example
    Regress,
    
    /// Run DBSCAN clustering example
    Cluster,
    
    /// Run all examples
    All,
    
    /// Load a custom dataset and run analysis
    Custom {
        /// Path to the dataset file
        #[arg(short, long)]
        file: PathBuf,
        
        /// Type of analysis to run (classify, regress, cluster)
        #[arg(short, long, default_value = "classify")]
        analysis: String,
    },
    
    /// Show help information
    Help,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Classify => run_logistic_regression_example()?,
        Commands::Tree => run_decision_tree_example()?,
        Commands::Regress => run_regression_example()?,
        Commands::Cluster => run_dbscan_example()?,
        Commands::All => {
            println!("\n=== Running LogisticRegression Classification Example ===\n");
            run_logistic_regression_example()?;
            
            println!("\n=== Running DecisionTree Classification Example ===\n");
            run_decision_tree_example()?;
            
            println!("\n=== Running LinearRegression Example ===\n");
            run_regression_example()?;
            
            println!("\n=== Running DBSCAN Clustering Example ===\n");
            run_dbscan_example()?;
        },
        Commands::Custom { file, analysis } => {
            println!("Loading custom dataset from {:?}", file);
            // This would load a custom dataset and run the specified analysis
            // For now, we'll just show a message
            println!("Custom dataset analysis is not implemented in this example.");
            println!("Please use one of the built-in examples.");
        },
        Commands::Help => {
            println!("Linfa Machine Learning Examples");
            println!("==============================");
            println!("Available commands:");
            println!("  classify  - Run logistic regression classification example");
            println!("  tree      - Run decision tree classification example");
            println!("  regress   - Run linear regression example");
            println!("  cluster   - Run DBSCAN clustering example");
            println!("  all       - Run all examples");
            println!("  custom    - Load a custom dataset and run analysis");
            println!("  help      - Show this help information");
        },
    }
    
    Ok(())
}

// LogisticRegression Classification Example
fn run_logistic_regression_example() -> Result<()> {
    // LogisticRegression classification example with Linfa 0.7.1
    println!("Linfa 0.7.1 LogisticRegression Classification Example");
    
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
    let dataset = Dataset::new(features, targets);
    
    // Split into train and test sets with a random seed for reproducibility
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let (train, test) = dataset.shuffle(&mut rng).split_with_ratio(0.5);
    
    println!("Training dataset: {} samples", train.nsamples());
    println!("Testing dataset: {} samples", test.nsamples());
    
    // Create and train the model
    println!("Training LogisticRegression model...");
    let model = linfa_logistic::LogisticRegression::default()
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
    println!("Accuracy: {:.2}", cm.accuracy());
    
    println!("\nThis example demonstrates a complete classification workflow using LogisticRegression in Linfa 0.7.1.");
    println!("It shows how to create a dataset, split it into training and testing sets, train a model, make predictions, and evaluate the results.");
    
    Ok(())
}

// Decision Tree Classification Example
fn run_decision_tree_example() -> Result<()> {
    // Decision Tree classification example with Linfa 0.7.1
    println!("Linfa 0.7.1 Decision Tree Classification Example");
    
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
    let dataset = Dataset::new(features, targets);
    
    // Split into train and test sets with a random seed for reproducibility
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let (train, test) = dataset.shuffle(&mut rng).split_with_ratio(0.75);
    
    println!("Training dataset: {} samples", train.nsamples());
    println!("Testing dataset: {} samples", test.nsamples());
    
    // Create and train the Decision Tree model
    println!("Training Decision Tree model...");
    
    // Using the correct API for Linfa 0.7.1
    let model = linfa_trees::DecisionTree::params()
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

// Linear Regression Example
fn run_regression_example() -> Result<()> {
    // Simple regression example with Linfa 0.7.1
    println!("Linfa 0.7.1 Linear Regression Example");
    
    // Create a simple dataset
    let features = Array2::from_shape_vec(
        (6, 1),
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    )?;
    
    // y = 2*x + 1 + noise
    let targets = Array1::from_vec(vec![3.1, 5.2, 7.0, 8.9, 10.8, 13.1]);
    
    // Create a dataset
    let dataset = Dataset::new(features.clone(), targets.clone());
    
    // Split into train and test sets with a random seed for reproducibility
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let (train, test) = dataset.shuffle(&mut rng).split_with_ratio(0.7);
    
    println!("Training dataset: {} samples", train.nsamples());
    println!("Testing dataset: {} samples", test.nsamples());
    
    // Create and train the model
    println!("Training LinearRegression model...");
    let model = linfa_linear::LinearRegression::default()
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
    
    // In Linfa 0.7.1, we need to access model parameters differently
    println!("Model parameters:");
    
    // Access the parameters directly from the model
    let params = model.params();
    
    // Print the parameters array shape and values
    println!("  Parameters shape: {:?}", params.shape());
    println!("  Parameters values: {:?}", params);
    
    // For a simple linear regression model with one feature,
    // we can estimate the coefficient and intercept from the data
    let x_mean = train.records().column(0).mean().unwrap_or(0.0);
    let y_mean = train.targets().mean().unwrap_or(0.0);
    
    // Calculate the coefficient (slope) using the predictions
    let x_test = test.records().column(0).to_owned();
    let y_pred = predictions;
    
    if x_test.len() > 0 && y_pred.len() > 0 {
        let coefficient = (y_pred[0] - y_mean) / (x_test[0] - x_mean);
        let intercept = y_mean - coefficient * x_mean;
        
        println!("  Estimated coefficient (m): {:.4}", coefficient);
        println!("  Estimated intercept (b): {:.4}", intercept);
        
        // Print the model equation
        println!("  Estimated model equation: y = {:.4} * x + {:.4}", coefficient, intercept);
    }
    
    // Predict on new data
    println!("\nPredicting on new data:");
    let new_data = Array2::from_shape_vec(
        (3, 1),
        vec![0.5, 7.0, 10.0],
    )?;
    
    let new_predictions = model.predict(&new_data);
    for (i, &x) in new_data.column(0).iter().enumerate() {
        println!("  x = {:.1}, predicted y = {:.2}", x, new_predictions[i]);
    }
    
    Ok(())
}

// DBSCAN Clustering Example
fn run_dbscan_example() -> Result<()> {
    // DBSCAN clustering example with Linfa 0.7.1
    println!("Linfa 0.7.1 DBSCAN Clustering Example");
    
    // Generate synthetic data with 3 clusters
    let n_samples_per_cluster = 100;
    let n_clusters = 3;
    let n_samples = n_samples_per_cluster * n_clusters;
    
    // Create a random number generator with a fixed seed for reproducibility
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    
    // Generate cluster centers
    let cluster_centers = Array2::random_using(
        (n_clusters, 2),
        Uniform::new(-20.0, 20.0),
        &mut rng,
    );
    
    // Generate samples around each cluster center
    let mut samples = Array2::zeros((n_samples, 2));
    
    for i in 0..n_clusters {
        let cluster_center = cluster_centers.row(i);
        let start_idx = i * n_samples_per_cluster;
        let end_idx = start_idx + n_samples_per_cluster;
        
        // Generate points around the cluster center with some noise
        let cluster_samples = Array2::random_using(
            (n_samples_per_cluster, 2),
            Uniform::new(-5.0, 5.0),
            &mut rng,
        );
        
        for j in start_idx..end_idx {
            for k in 0..2 {
                samples[[j, k]] = cluster_center[k] + cluster_samples[[j - start_idx, k]];
            }
        }
    }
    
    println!("Dataset shape: {:?}", samples.shape());
    println!("Number of samples: {}", n_samples);
    
    // Create a dataset for clustering
    let dataset = Dataset::from(samples);
    
    // Set DBSCAN parameters
    let min_points = 3;  // Minimum points to form a dense region
    let tolerance = 2.0; // Maximum distance between two samples to be considered neighbors
    
    println!("Running DBSCAN clustering with min_points = {}, tolerance = {}", min_points, tolerance);
    
    // Create and run the DBSCAN model
    let model = linfa_clustering::Dbscan::params(min_points)
        .tolerance(tolerance)
        .transform(&dataset)?;
    
    // Get cluster assignments
    let cluster_memberships = model.cluster_memberships();
    
    // Print the first 10 cluster assignments
    println!("First 10 cluster assignments: {:?}", &cluster_memberships.slice(Axis(0), 0..10.min(cluster_memberships.len())));
    
    // Count the number of points in each cluster
    let mut cluster_counts: HashMap<usize, usize> = HashMap::new();
    for membership in cluster_memberships.iter() {
        if let Some(cluster_idx) = membership {
            *cluster_counts.entry(*cluster_idx).or_insert(0) += 1;
        }
    }
    
    // Print cluster counts
    println!("Cluster counts:");
    for (cluster_idx, count) in cluster_counts.iter() {
        println!("  Cluster {}: {} points", cluster_idx, count);
    }
    
    Ok(())
}
