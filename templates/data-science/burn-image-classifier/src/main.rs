// Main application for the image classifier
// This file contains the command-line interface and main logic

mod config;
mod data;
mod error;
mod model;
mod visualization;

use burn::data::dataloader::{DataLoaderBuilder};
use burn::tensor::backend::Backend;
use burn::tensor::{Tensor, TensorData};
use burn_ndarray::{NdArray, NdArrayDevice};
use clap::{Parser, Subcommand};

use crate::model::ImageClassifierModel;
use crate::data::{load_image_dataset, ImageBatcher, image_to_tensor};
use crate::error::{Result, ImageClassifierError};
use crate::config::{
    BATCH_SIZE, EPOCHS, DEFAULT_DATA_DIR, DEFAULT_MODEL_FILE, 
    IMAGE_SIZE, NUM_CHANNELS, ImageClassifierConfig
};
use crate::visualization::{
    plot_training_history, plot_predictions, Accuracy
};

use std::path::Path;

/// Image Classifier CLI
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Train a new model
    Train {
        /// Path to the data directory
        #[arg(short, long, default_value = DEFAULT_DATA_DIR)]
        data_dir: String,
        
        /// Number of epochs to train for
        #[arg(short, long, default_value_t = EPOCHS)]
        epochs: usize,
        
        /// Output directory for the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        output: String,
        
        /// Batch size for training
        #[arg(short, long, default_value_t = BATCH_SIZE)]
        batch_size: usize,
    },
    
    /// Evaluate a trained model
    Evaluate {
        /// Path to the data directory
        #[arg(short, long, default_value = DEFAULT_DATA_DIR)]
        data_dir: String,
        
        /// Path to the model file
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_path: String,
        
        /// Batch size for evaluation
        #[arg(short, long, default_value_t = BATCH_SIZE)]
        batch_size: usize,
    },
    
    /// Predict class for a single image
    Predict {
        /// Path to the image file
        #[arg(short, long)]
        image_path: String,
        
        /// Path to the model file
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Train { data_dir, epochs, output, batch_size } => {
            train(data_dir.clone(), *epochs, output.clone(), *batch_size)
        },
        Commands::Evaluate { data_dir, model_path, batch_size } => {
            evaluate(model_path.clone(), data_dir.clone(), *batch_size)
        },
        Commands::Predict { image_path, model_path } => {
            predict(model_path.clone(), image_path.clone())
        },
    }
}

fn train(data_dir: String, epochs: usize, output: String, batch_size: usize) -> Result<()> {
    // Load dataset
    println!("Loading dataset from {}", data_dir);
    let dataset = load_image_dataset(&data_dir, IMAGE_SIZE)?;
    
    println!("Found {} images with {} classes", dataset.len(), dataset.num_classes());
    
    // Split into train and validation sets
    let (train_dataset, valid_dataset) = dataset.split(0.8);
    
    println!("Training set: {} images", train_dataset.len());
    println!("Validation set: {} images", valid_dataset.len());
    
    // Create device
    let device = NdArrayDevice::default();
    
    // Create model
    let num_classes = dataset.num_classes();
    let config = ImageClassifierConfig {
        num_classes,
        conv_filters: vec![32, 64],
        fc_layers: vec![128],
        dropout_rate: 0.5,
    };
    
    let model = ImageClassifierModel::<NdArray>::new(&config, &device);
    
    // Create batcher
    let batcher = ImageBatcher::<NdArray>::new(device.clone());
    
    // Create data loaders
    let train_loader = DataLoaderBuilder::new(batcher.clone())
        .batch_size(batch_size)
        .shuffle(42) // Use a fixed seed for reproducibility
        .build(train_dataset);
        
    let valid_loader = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .build(valid_dataset);
    
    // Create metrics
    let mut accuracy_metric = Accuracy::<NdArray>::new();
    let mut train_losses = Vec::<f64>::new();
    let mut valid_losses = Vec::<f64>::new();
    let mut train_accuracies = Vec::<f64>::new();
    let mut valid_accuracies = Vec::<f64>::new();
    
    // Training loop
    for epoch in 0..epochs {
        println!("Epoch {}/{}", epoch + 1, epochs);
        
        // Training phase
        let mut train_loss = 0.0f32;
        let mut train_batches = 0;
        
        accuracy_metric.reset();
        
        for (images, targets) in train_loader.iter() {
            // Forward pass
            let output = model.forward(images.clone());
            
            // Compute loss
            let loss = cross_entropy_with_logits_loss(output.clone(), targets.clone());
            
            // Update metrics
            train_loss += loss.mean().into_scalar();
            train_batches += 1;
            
            // Calculate accuracy
            let predictions = output.argmax(1);
            let target_classes = targets.argmax(1);
            
            let pred_vec: Vec<f32> = predictions.into_data().into_vec().unwrap_or_default();
            let target_vec: Vec<f32> = target_classes.into_data().into_vec().unwrap_or_default();
            
            for i in 0..pred_vec.len() {
                if pred_vec[i] as usize == target_vec[i] as usize {
                    accuracy_metric.add_correct();
                } else {
                    accuracy_metric.add_incorrect();
                }
            }
        }
        
        let train_accuracy = accuracy_metric.compute();
        train_loss /= train_batches as f32;
        
        train_losses.push(train_loss as f64);
        train_accuracies.push(train_accuracy as f64);
        
        println!("Train Loss: {:.4}, Train Accuracy: {:.4}", train_loss, train_accuracy);
        
        // Validation phase
        let mut valid_loss = 0.0f32;
        let mut valid_batches = 0;
        
        accuracy_metric.reset();
        
        for (images, targets) in valid_loader.iter() {
            // Forward pass
            let output = model.forward(images.clone());
            
            // Compute loss
            let loss = cross_entropy_with_logits_loss(output.clone(), targets.clone());
            
            // Update metrics
            valid_loss += loss.mean().into_scalar();
            valid_batches += 1;
            
            // Calculate accuracy
            let predictions = output.argmax(1);
            let target_classes = targets.argmax(1);
            
            let pred_vec: Vec<f32> = predictions.into_data().into_vec().unwrap_or_default();
            let target_vec: Vec<f32> = target_classes.into_data().into_vec().unwrap_or_default();
            
            for i in 0..pred_vec.len() {
                if pred_vec[i] as usize == target_vec[i] as usize {
                    accuracy_metric.add_correct();
                } else {
                    accuracy_metric.add_incorrect();
                }
            }
        }
        
        let valid_accuracy = accuracy_metric.compute();
        valid_loss /= valid_batches as f32;
        
        valid_losses.push(valid_loss as f64);
        valid_accuracies.push(valid_accuracy as f64);
        
        println!("Valid Loss: {:.4}, Valid Accuracy: {:.4}", valid_loss, valid_accuracy);
    }
    
    // Save model
    let output_dir = Path::new(&output);
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }
    
    let model_path = output_dir.join("model.json");
    ImageClassifierModel::<NdArray>::save_file(&model, model_path.to_str().unwrap())?;
    
    // Plot training history
    let history_path = output_dir.join("training_history.png");
    plot_training_history(
        &train_losses,
        &valid_losses,
        &train_accuracies,
        &valid_accuracies,
        history_path.to_str().unwrap(),
    )?;
    
    println!("Training completed. Model saved to {}", output);
    
    Ok(())
}

fn evaluate(model_path: String, data_dir: String, batch_size: usize) -> Result<()> {
    // Load dataset
    println!("Loading dataset from {}", data_dir);
    let dataset = load_image_dataset(&data_dir, IMAGE_SIZE)?;
    
    println!("Found {} images with {} classes", dataset.len(), dataset.num_classes());
    
    // Create device
    let device = NdArrayDevice::default();
    
    // Load model
    let model = ImageClassifierModel::<NdArray>::load(&model_path, &device)?;
    
    // Create batcher
    let batcher = ImageBatcher::<NdArray>::new(device.clone());
    
    // Store num_classes before moving dataset
    let num_classes = dataset.num_classes();
    
    // Create data loader
    let data_loader = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .build(dataset);
    
    // Create metrics
    let mut accuracy_metric = Accuracy::<NdArray>::new();
    let mut confusion_matrix = vec![vec![0; num_classes]; num_classes];
    
    // Evaluation loop
    for (images, targets) in data_loader.iter() {
        // Forward pass
        let output = model.forward(images.clone());
        
        // Get predicted classes (use argmax instead of softmax)
        let predictions = output.argmax(1);
        let target_classes = targets.argmax(1);
        
        let pred_vec: Vec<f32> = predictions.into_data().into_vec().unwrap_or_default();
        let target_vec: Vec<f32> = target_classes.into_data().into_vec().unwrap_or_default();
        
        // Update metrics
        for i in 0..pred_vec.len() {
            let pred_idx = pred_vec[i] as usize;
            let target_idx = target_vec[i] as usize;
            
            // Update confusion matrix
            confusion_matrix[target_idx][pred_idx] += 1;
            
            // Update accuracy
            if pred_idx == target_idx {
                accuracy_metric.add_correct();
            } else {
                accuracy_metric.add_incorrect();
            }
        }
    }
    
    // Calculate accuracy
    let accuracy = accuracy_metric.compute();
    
    println!("Evaluation Results:");
    println!("Accuracy: {:.4}", accuracy);
    
    // Print confusion matrix
    println!("Confusion Matrix:");
    println!("{:^10}", "");
    print!("{:^10}", "Pred →");
    for i in 0..num_classes {
        print!("{:^5}", i);
    }
    println!();
    
    for (i, row) in confusion_matrix.iter().enumerate() {
        print!("{:^10}", format!("True {}", i));
        for &count in row {
            print!("{:^5}", count);
        }
        println!();
    }
    
    Ok(())
}

fn predict(model_path: String, image_path: String) -> Result<()> {
    // Load image
    println!("Loading image from {}", image_path);
    let img = image::open(&image_path).map_err(|e| ImageClassifierError::ImageError(e))?;
    
    // Convert image to tensor
    let tensor = image_to_tensor(&img)?;
    
    // Create device
    let device = NdArrayDevice::default();
    
    // Load model
    let model = ImageClassifierModel::<NdArray>::load(&model_path, &device)?;
    
    // Create tensor from image
    let tensor_data = TensorData::new(tensor, [1, NUM_CHANNELS, IMAGE_SIZE, IMAGE_SIZE]);
    let input = Tensor::<NdArray, 4>::from_data(tensor_data, &device);
    
    // Forward pass
    let output = model.forward(input);
    
    // Get top predictions (use argmax instead of softmax)
    let (indices, values) = top_k(output, 5);
    
    // Print predictions
    println!("Predictions:");
    for (i, (class_idx, prob)) in indices.iter().zip(values.iter()).enumerate() {
        println!("{}. Class {}: {:.2}%", i + 1, class_idx, prob * 100.0);
    }
    
    // Plot predictions
    let output_path = "prediction.png";
    plot_predictions(&img, &indices, &values, output_path)?;
    
    println!("Prediction visualization saved to {}", output_path);
    
    Ok(())
}

/// Calculate the top-k values and indices from a tensor
fn top_k<B: Backend>(tensor: Tensor<B, 2>, k: usize) -> (Vec<usize>, Vec<f32>) {
    let data = tensor.into_data();
    let values = data.into_vec().unwrap_or_default();
    
    // Create (index, value) pairs
    let mut pairs: Vec<(usize, f32)> = Vec::new();
    for i in 0..values.len() {
        pairs.push((i, values[i]));
    }
    
    // Sort by value in descending order
    pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // Take top k
    let top_k_pairs = pairs.into_iter().take(k).collect::<Vec<_>>();
    
    // Split into separate vectors
    let indices = top_k_pairs.iter().map(|&(i, _)| i).collect();
    let values = top_k_pairs.iter().map(|&(_, v)| v).collect();
    
    (indices, values)
}

/// Calculate cross-entropy loss with logits
fn cross_entropy_with_logits_loss<B: Backend>(output: Tensor<B, 2>, targets: Tensor<B, 2>) -> Tensor<B, 1> {
    // Apply log softmax (manually calculate instead of using softmax().log())
    let max_vals = output.clone().max_dim(1);
    let max_vals = max_vals.unsqueeze::<2>();
    let shifted = output.clone() - max_vals.clone();
    let exp_shifted = shifted.clone().exp();
    let sum_exp = exp_shifted.sum_dim(1);
    let sum_exp = sum_exp.unsqueeze::<2>();
    let log_softmax = shifted - sum_exp.log();
    
    // Calculate negative log likelihood loss for each sample
    let loss = -(log_softmax * targets);
    
    // Sum along the class dimension to get per-sample loss
    // Ensure we have a 1D tensor by flattening the result
    let batch_size = output.shape().dims::<2>()[0];
    loss.sum_dim(1).reshape([batch_size])
}
