// Image Recognition Application using Linfa
// This program trains a machine learning model to recognize handwritten digits
// using the MNIST dataset and the Linfa framework

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use linfa::prelude::*;
use linfa_nn::{KNearestNeighbors, KNearestNeighborsValidParams};
use ndarray::{Array2, Axis};
use ndarray_rand::rand::SeedableRng;
use rand_isaac::Isaac64Rng;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};

// Import our data handling code
mod data;

// Command line interface for our application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Available commands: train or evaluate
#[derive(Subcommand)]
enum Commands {
    // Train a new model
    Train {
        // Number of neighbors to consider in KNN algorithm
        #[arg(short, long, default_value_t = 5)]
        k_neighbors: usize,
        
        // Where to save the trained model
        #[arg(short, long, default_value = "model.json")]
        output: String,
        
        // Whether to use the actual MNIST dataset (if available)
        #[arg(long)]
        use_mnist: bool,
    },
    
    // Evaluate an existing model
    Evaluate {
        // Path to the saved model file
        #[arg(short, long)]
        model: String,
        
        // Whether to use the actual MNIST dataset (if available)
        #[arg(long)]
        use_mnist: bool,
    },
    
    // Predict using a single image
    Predict {
        // Path to the saved model file
        #[arg(short, long)]
        model: String,
        
        // Path to the image file
        #[arg(short, long)]
        image: String,
    },
}

// Main function - entry point of our program
fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Choose which command to run
    match cli.command {
        Commands::Train { k_neighbors, output, use_mnist } => {
            // Run the training process
            train(k_neighbors, output, use_mnist)?;
        }
        Commands::Evaluate { model, use_mnist } => {
            // Run the evaluation process
            evaluate(model, use_mnist)?;
        }
        Commands::Predict { model, image } => {
            // Run the prediction process
            predict(model, image)?;
        }
    }
    
    Ok(())
}

// Training function - teaches our model to recognize digits
fn train(k_neighbors: usize, output: String, use_mnist: bool) -> Result<()> {
    println!("Loading dataset...");
    
    // Load the dataset (either real MNIST or synthetic data)
    let (train_data, train_targets, test_data, test_targets) = 
        if use_mnist {
            match data::load_real_mnist() {
                Ok(dataset) => dataset,
                Err(_) => {
                    println!("Warning: Could not load real MNIST dataset. Using synthetic data instead.");
                    data::generate_synthetic_dataset(1000, 100)?
                }
            }
        } else {
            data::generate_synthetic_dataset(1000, 100)?
        };
    
    println!("Training data shape: {:?}", train_data.shape());
    println!("Training targets shape: {:?}", train_targets.shape());
    
    // Create a dataset from the training data
    let train_dataset = Dataset::new(train_data, train_targets);
    
    println!("Training K-Nearest Neighbors model with k = {}...", k_neighbors);
    
    // Create a progress bar
    let progress = ProgressBar::new(1);
    progress.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
        .unwrap());
    progress.set_message("Training model...");
    
    // Create and train a K-Nearest Neighbors model
    // KNN doesn't actually "train" in the traditional sense - it just stores the data
    let model = KNearestNeighbors::params()
        .k(k_neighbors)
        .weights(linfa_nn::Distance::Euclidean)
        .fit(&train_dataset)?;
    
    progress.finish_with_message("Training complete!");
    
    // Evaluate on the test data
    let test_dataset = Dataset::new(test_data.clone(), test_targets.clone());
    let predictions = model.predict(&test_dataset);
    
    // Calculate accuracy
    let correct_predictions = predictions
        .iter()
        .zip(test_targets.iter())
        .filter(|(pred, actual)| pred == actual)
        .count();
    
    let accuracy = correct_predictions as f64 / test_targets.len() as f64;
    println!("Test accuracy: {:.4}", accuracy);
    
    // Save the model
    println!("Saving model to {}...", output);
    model.save(&output)?;
    
    println!("Training complete!");
    
    Ok(())
}

// Evaluation function - tests how well our model recognizes digits
fn evaluate(model_path: String, use_mnist: bool) -> Result<()> {
    println!("Loading model from {}...", model_path);
    
    // Load the saved model
    let model = KNearestNeighbors::<f64, usize>::load(&model_path)?;
    
    println!("Loading test dataset...");
    
    // Load the dataset (either real MNIST or synthetic data)
    let (_, _, test_data, test_targets) = 
        if use_mnist {
            match data::load_real_mnist() {
                Ok(dataset) => dataset,
                Err(_) => {
                    println!("Warning: Could not load real MNIST dataset. Using synthetic data instead.");
                    data::generate_synthetic_dataset(100, 100)?
                }
            }
        } else {
            data::generate_synthetic_dataset(100, 100)?
        };
    
    println!("Test data shape: {:?}", test_data.shape());
    println!("Test targets shape: {:?}", test_targets.shape());
    
    // Create a dataset from the test data
    let test_dataset = Dataset::new(test_data, test_targets.clone());
    
    // Make predictions
    println!("Making predictions...");
    let predictions = model.predict(&test_dataset);
    
    // Calculate accuracy
    let correct_predictions = predictions
        .iter()
        .zip(test_targets.iter())
        .filter(|(pred, actual)| pred == actual)
        .count();
    
    let accuracy = correct_predictions as f64 / test_targets.len() as f64;
    println!("Test accuracy: {:.4}", accuracy);
    
    // Print confusion matrix
    println!("Confusion matrix:");
    let mut confusion_matrix = vec![vec![0; 10]; 10];
    for (pred, actual) in predictions.iter().zip(test_targets.iter()) {
        confusion_matrix[*actual][*pred] += 1;
    }
    
    for i in 0..10 {
        println!("{}: {:?}", i, confusion_matrix[i]);
    }
    
    Ok(())
}

// Prediction function - recognizes a single digit
fn predict(model_path: String, image_path: String) -> Result<()> {
    println!("Loading model from {}...", model_path);
    
    // Load the saved model
    let model = KNearestNeighbors::<f64, usize>::load(&model_path)?;
    
    println!("Loading image from {}...", image_path);
    
    // Load and preprocess the image
    let image_data = data::load_and_preprocess_image(&image_path)?;
    
    // Create a dataset with just this image
    let image_dataset = Dataset::new(image_data, ndarray::Array1::zeros(1));
    
    // Make a prediction
    let prediction = model.predict(&image_dataset);
    let digit = prediction[0];
    
    println!("Prediction: {}", digit);
    
    Ok(())
}
