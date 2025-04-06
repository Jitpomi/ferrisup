// Image Classifier CLI Application
// This program provides a command-line interface for training and using image classification models

use burn::data::dataset::Dataset;
use burn::module::Module;
use burn::tensor::backend::Backend;
use burn::backend::ndarray::{NdArray, NdArrayDevice};
use burn::backend::Autodiff;
use burn::optim::Adam;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

// Import from our library
use {{project_name}}::{
    model::ImageClassifierModel,
    data::{ImageBatcher, ImageDataset, load_image_dataset},
    config::{BATCH_SIZE, LEARNING_RATE, EPOCHS, IMAGE_SIZE, DEFAULT_DATA_DIR, DEFAULT_MODEL_FILE},
    training::{TrainingStepHandler, ValidationStepHandler}
};

// Command line interface for our application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Train a new image classification model
    Train {
        /// Directory containing training images (organized in class subdirectories)
        #[arg(short, long, default_value = DEFAULT_DATA_DIR)]
        data_dir: String,
        
        /// Number of training epochs
        #[arg(short, long, default_value_t = EPOCHS)]
        epochs: usize,
        
        /// Output file to save the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        output: String,
        
        /// Batch size for training
        #[arg(short, long, default_value_t = BATCH_SIZE)]
        batch_size: usize,
    },
    
    /// Evaluate a trained model on a test dataset
    Evaluate {
        /// Path to the trained model file
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model: String,
        
        /// Directory containing test images (organized in class subdirectories)
        #[arg(short, long, default_value = DEFAULT_DATA_DIR)]
        data_dir: String,
    },
    
    /// Predict the class of a single image
    Predict {
        /// Path to the trained model file
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model: String,
        
        /// Path to the image file to classify
        #[arg(short, long)]
        image: String,
    },
}

// Main function - entry point of our program
fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Execute the appropriate command
    match cli.command {
        Commands::Train { data_dir, epochs, output, batch_size } => {
            train(data_dir, epochs, output, batch_size)
        },
        Commands::Evaluate { model, data_dir } => {
            evaluate(model, data_dir)
        },
        Commands::Predict { model, image } => {
            predict(model, image)
        },
    }
}

// Training function - teaches our model to classify images
fn train(data_dir: String, epochs: usize, output: String, batch_size: usize) -> Result<()> {
    println!("🔍 Loading dataset from {}", data_dir);
    let dataset = load_image_dataset(&data_dir, IMAGE_SIZE)?;
    
    println!("📊 Dataset loaded with {} images in {} classes", dataset.len(), dataset.num_classes());
    println!("   Classes: {:?}", dataset.class_names());
    
    // Split dataset into training and validation sets (80%/20%)
    let (train_dataset, valid_dataset) = dataset.split_by_ratio([0.8, 0.2]);
    
    println!("🧩 Split dataset: {} training images, {} validation images", 
             train_dataset.len(), valid_dataset.len());
    
    // Initialize device and model
    let device = NdArrayDevice::Cpu;
    type Backend = Autodiff<NdArray>;
    
    // Create model configuration based on our dataset
    let config = ImageClassifierModel::<Backend>::config()
        .with_num_classes(dataset.num_classes());
    
    // Initialize model
    let model = ImageClassifierModel::new(&config, &device);
    
    // Initialize optimizer
    let optimizer = Adam::new(LEARNING_RATE);
    
    // Create data batchers
    let train_batcher = ImageBatcher::<Backend>::new(batch_size);
    let valid_batcher = ImageBatcher::<Backend>::new(batch_size);
    
    // Create step handlers
    let train_step = TrainingStepHandler::new(model.clone(), optimizer);
    let valid_step = ValidationStepHandler::new(model.clone());
    
    // Create progress bar
    let progress_bar = ProgressBar::new((epochs * train_dataset.len() / batch_size) as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    
    // Training loop
    println!("🚀 Starting training for {} epochs", epochs);
    
    for epoch in 0..epochs {
        // Training phase
        let mut train_loss = 0.0;
        let mut train_accuracy = 0.0;
        let mut train_batches = 0;
        
        for batch_idx in 0..(train_dataset.len() / batch_size) {
            // Get batch of items
            let items = (0..batch_size)
                .map(|i| train_dataset.get(batch_idx * batch_size + i).unwrap())
                .collect();
            
            // Process batch
            let batch = train_batcher.batch(items);
            let output = train_step.step(&batch);
            
            // Update metrics
            train_loss += output.loss;
            train_accuracy += output.output.accuracy;
            train_batches += 1;
            
            // Update progress bar
            progress_bar.inc(1);
            progress_bar.set_message(format!("Epoch {}/{}", epoch + 1, epochs));
        }
        
        // Validation phase
        let mut valid_accuracy = 0.0;
        let mut valid_batches = 0;
        
        for batch_idx in 0..(valid_dataset.len() / batch_size) {
            // Get batch of items
            let items = (0..batch_size)
                .map(|i| valid_dataset.get(batch_idx * batch_size + i).unwrap())
                .collect();
            
            // Process batch
            let batch = valid_batcher.batch(items);
            let output = valid_step.step(&batch);
            
            // Update metrics
            valid_accuracy += output.accuracy;
            valid_batches += 1;
        }
        
        // Calculate epoch metrics
        let epoch_train_loss = train_loss / train_batches as f32;
        let epoch_train_accuracy = train_accuracy / train_batches as f32;
        let epoch_valid_accuracy = valid_accuracy / valid_batches as f32;
        
        println!("📈 Epoch {}/{}: Loss = {:.4}, Train Acc = {:.2}%, Valid Acc = {:.2}%", 
                 epoch + 1, epochs, 
                 epoch_train_loss, 
                 epoch_train_accuracy * 100.0, 
                 epoch_valid_accuracy * 100.0);
    }
    
    progress_bar.finish_with_message("Training complete!");
    
    // Save the model
    println!("💾 Saving model to {}", output);
    
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(&output).parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Save the model
    model.save_file(output, &device)?;
    
    println!("✅ Model saved successfully!");
    
    Ok(())
}

// Evaluation function - tests how well our model classifies images
fn evaluate(model_path: String, data_dir: String) -> Result<()> {
    println!("🔍 Loading dataset from {}", data_dir);
    let dataset = load_image_dataset(&data_dir, IMAGE_SIZE)?;
    
    println!("📊 Dataset loaded with {} images in {} classes", dataset.len(), dataset.num_classes());
    
    // Initialize device and model
    let device = NdArrayDevice::Cpu;
    type Backend = Autodiff<NdArray>;
    
    // Load the model
    println!("📂 Loading model from {}", model_path);
    let model = ImageClassifierModel::<Backend>::load_file(model_path, &device)?;
    
    // Create data batcher
    let batcher = ImageBatcher::<Backend>::new(1); // Evaluate one image at a time
    
    // Create validation step handler
    let mut valid_step = ValidationStepHandler::new(model);
    
    // Evaluation loop
    println!("🧪 Evaluating model on {} images", dataset.len());
    
    let mut correct = 0;
    let mut total = 0;
    let mut class_correct = vec![0; dataset.num_classes()];
    let mut class_total = vec![0; dataset.num_classes()];
    
    // Create progress bar
    let progress_bar = ProgressBar::new(dataset.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    
    for i in 0..dataset.len() {
        // Get item
        let item = dataset.get(i).unwrap();
        let true_class = item.target as usize;
        
        // Process item
        let batch = batcher.batch(vec![item]);
        let output = valid_step.step(&batch);
        
        // Get predicted class
        let pred_class = model.get_class_index(&batch.images);
        
        // Update metrics
        if pred_class == true_class {
            correct += 1;
            class_correct[true_class] += 1;
        }
        
        total += 1;
        class_total[true_class] += 1;
        
        // Update progress bar
        progress_bar.inc(1);
    }
    
    progress_bar.finish();
    
    // Calculate overall accuracy
    let accuracy = (correct as f32) / (total as f32) * 100.0;
    println!("📊 Overall Accuracy: {}/{} ({:.2}%)", correct, total, accuracy);
    
    // Print per-class accuracy
    println!("📊 Per-class Accuracy:");
    for (i, class_name) in dataset.class_names().iter().enumerate() {
        if class_total[i] > 0 {
            let class_acc = (class_correct[i] as f32) / (class_total[i] as f32) * 100.0;
            println!("   {}: {}/{} ({:.2}%)", class_name, class_correct[i], class_total[i], class_acc);
        }
    }
    
    Ok(())
}

// Prediction function - classifies a single image
fn predict(model_path: String, image_path: String) -> Result<()> {
    // Initialize device and model
    let device = NdArrayDevice::Cpu;
    type Backend = Autodiff<NdArray>;
    
    // Load the model
    println!("📂 Loading model from {}", model_path);
    let model = ImageClassifierModel::<Backend>::load_file(model_path, &device)?;
    
    // Load the image
    println!("🖼️ Loading image from {}", image_path);
    
    use {{project_name}}::data::load_and_process_image;
    let img = load_and_process_image(Path::new(&image_path))?;
    
    // Convert image to tensor
    use {{project_name}}::data::{RawImageItem, image_to_tensor};
    let tensor_data = image_to_tensor(&img);
    
    // Create a batch with just this image
    let batcher = ImageBatcher::<Backend>::new(1);
    let batch = batcher.batch(vec![RawImageItem {
        image: tensor_data,
        target: 0, // Dummy target, not used for prediction
    }]);
    
    // Get prediction
    let class_index = model.get_class_index(&batch.images);
    let class_probs = model.get_class_probabilities(&batch.images);
    
    // Display results
    println!("🔮 Prediction Results:");
    println!("   Predicted class: {}", class_index);
    
    // Display top probabilities
    println!("   Class probabilities:");
    let mut probs: Vec<(usize, f32)> = class_probs.iter().enumerate().map(|(i, &p)| (i, p)).collect();
    probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    for (i, (class_idx, prob)) in probs.iter().take(3).enumerate() {
        println!("   {}. Class {}: {:.2}%", i + 1, class_idx, prob * 100.0);
    }
    
    Ok(())
}

// Backend selection module for ndarray
#[cfg(feature = "ndarray")]
mod ndarray_backend {
    use burn::backend::{Autodiff, ndarray::{NdArray, NdArrayDevice}};
    use super::*;

    pub fn run() {
        let device = NdArrayDevice::Cpu;
        // Run with ndarray backend
    }
}

// Backend selection module for torch CPU
#[cfg(feature = "tch-cpu")]
mod tch_cpu {
    use burn::backend::{Autodiff, libtorch::{LibTorch, LibTorchDevice}};
    use super::*;

    pub fn run() {
        let device = LibTorchDevice::Cpu;
        // Run with torch CPU backend
    }
}

// Backend selection module for torch GPU
#[cfg(feature = "tch-gpu")]
mod tch_gpu {
    use burn::backend::{Autodiff, libtorch::{LibTorch, LibTorchDevice}};
    use super::*;

    pub fn run() {
        #[cfg(not(target_os = "macos"))]
        let device = LibTorchDevice::Cuda(0);
        #[cfg(target_os = "macos")]
        let device = LibTorchDevice::Mps;
        // Run with torch GPU backend
    }
}

// Backend selection module for wgpu
#[cfg(feature = "wgpu")]
mod wgpu {
    use burn::backend::{Autodiff, wgpu::{Wgpu, WgpuDevice}};
    use super::*;

    pub fn run() {
        let device = WgpuDevice::default();
        // Run with wgpu backend
    }
}
