// Custom Image Classifier Application
// This program trains a neural network to classify your own images
// into different categories (like cats vs. dogs, or flower types)

use burn::data::dataset::Dataset;
use burn::module::Module;
use burn::tensor::backend::Backend;
use burn::train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep};
use burn::config::Config;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

// Import our model and data handling code
mod model;
mod data;

use model::{ImageClassifierConfig, ImageClassifierModel};
use data::{ImageBatcher, ImageItem, ImageDataset};

// Command line interface for our application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Available commands: train, evaluate, or predict
#[derive(Subcommand)]
enum Commands {
    // Train a new model
    Train {
        // Directory containing training images organized in category folders
        #[arg(long)]
        data_dir: String,
        
        // Number of training cycles
        #[arg(short, long, default_value_t = 20)]
        epochs: usize,
        
        // Where to save the trained model
        #[arg(short, long, default_value = "model.json")]
        output: String,
        
        // How many images to process at once
        #[arg(short, long, default_value_t = 16)]
        batch_size: usize,
        
        // Size to resize images to (square)
        #[arg(long, default_value_t = 224)]
        image_size: usize,
    },
    
    // Evaluate an existing model
    Evaluate {
        // Path to the saved model file
        #[arg(short, long)]
        model: String,
        
        // Directory containing test images organized in category folders
        #[arg(long)]
        data_dir: String,
        
        // Size to resize images to (square)
        #[arg(long, default_value_t = 224)]
        image_size: usize,
    },
    
    // Predict the category of a single image
    Predict {
        // Path to the saved model file
        #[arg(short, long)]
        model: String,
        
        // Path to the image to classify
        #[arg(short, long)]
        image: String,
        
        // Size to resize image to (square)
        #[arg(long, default_value_t = 224)]
        image_size: usize,
    },
}

// Main function - entry point of our program
fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Choose which command to run
    match cli.command {
        Commands::Train { data_dir, epochs, output, batch_size, image_size } => {
            // Run the training process
            train(data_dir, epochs, output, batch_size, image_size)?;
        }
        Commands::Evaluate { model, data_dir, image_size } => {
            // Run the evaluation process
            evaluate(model, data_dir, image_size)?;
        }
        Commands::Predict { model, image, image_size } => {
            // Run the prediction process
            predict(model, image, image_size)?;
        }
    }
    
    Ok(())
}

// Training function - teaches our model to classify images
fn train(data_dir: String, epochs: usize, output: String, batch_size: usize, image_size: usize) -> Result<()> {
    // We'll use the CPU for computations
    // You can change this to GPU if available
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading images from {}...", data_dir);
    
    // Load the image dataset
    let dataset = data::load_image_dataset(&data_dir, image_size)?;
    
    // Get the number of categories (classes)
    let num_classes = dataset.num_classes();
    let class_names = dataset.class_names();
    
    println!("Found {} classes: {:?}", num_classes, class_names);
    println!("Found {} images", dataset.len());
    
    // Split into training and validation sets (80% train, 20% validation)
    let (train_data, valid_data) = dataset.split_by_ratio([0.8, 0.2]);
    
    println!("Training set: {} images", train_data.len());
    println!("Validation set: {} images", valid_data.len());
    
    // Create data batchers (group images into batches)
    let train_batcher = ImageBatcher::<B>::new(batch_size);
    let valid_batcher = ImageBatcher::<B>::new(batch_size);
    
    // Create data loaders
    let train_loader = train_data.into_loader(train_batcher, batch_size, true, None);
    let valid_loader = valid_data.into_loader(valid_batcher, batch_size, false, None);
    
    println!("Creating model with {} output classes...", num_classes);
    
    // Create a new model with configuration for our specific task
    let config = ImageClassifierConfig::new(num_classes);
    let mut model = ImageClassifierModel::<B>::new(&config);
    
    // Create an optimizer (Adam) to adjust model weights during training
    let learning_rate = 1e-3;
    let optimizer = burn::optim::Adam::new(learning_rate);
    
    // Create a training step handler
    let mut train_step = TrainingStepHandler::new(model.clone(), optimizer);
    
    // Create a validation step handler
    let mut valid_step = ValidationStepHandler::new(model.clone());
    
    println!("Starting training for {} epochs...", epochs);
    
    // Training loop
    for epoch in 1..=epochs {
        println!("Epoch {}/{}:", epoch, epochs);
        
        // Training phase
        let mut train_metrics = ClassificationOutput::new();
        
        // Create a progress bar for training
        let train_progress = ProgressBar::new(train_loader.len() as u64);
        train_progress.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap());
        train_progress.set_message("Training");
        
        for batch in train_loader.iter() {
            // Perform one training step
            let output = train_step.step(&batch);
            train_metrics.extend(&output);
            train_progress.inc(1);
        }
        
        train_progress.finish_with_message(format!(
            "Train Loss: {:.4}, Train Accuracy: {:.4}",
            train_metrics.loss(),
            train_metrics.accuracy()
        ));
        
        // Validation phase
        let mut valid_metrics = ClassificationOutput::new();
        
        // Create a progress bar for validation
        let valid_progress = ProgressBar::new(valid_loader.len() as u64);
        valid_progress.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.green/blue} {pos}/{len} {msg}")
            .unwrap());
        valid_progress.set_message("Validating");
        
        for batch in valid_loader.iter() {
            // Perform one validation step
            let output = valid_step.step(&batch);
            valid_metrics.extend(&output);
            valid_progress.inc(1);
        }
        
        valid_progress.finish_with_message(format!(
            "Valid Loss: {:.4}, Valid Accuracy: {:.4}",
            valid_metrics.loss(),
            valid_metrics.accuracy()
        ));
        
        // Update the model with the best weights
        model = train_step.model.clone();
    }
    
    // Save the trained model
    println!("Saving model to {}...", output);
    let artifact = model.into_artifact();
    artifact.save(output).expect("Failed to save model");
    
    // Save class names alongside the model
    let class_names_path = Path::new(&output).with_extension("classes.json");
    let class_names_json = serde_json::to_string(&class_names)?;
    std::fs::write(class_names_path, class_names_json)?;
    
    println!("Training complete!");
    
    Ok(())
}

// Evaluation function - tests how well our model classifies images
fn evaluate(model_path: String, data_dir: String, image_size: usize) -> Result<()> {
    // We'll use the CPU for computations
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading model from {}...", model_path);
    
    // Load the saved model
    let artifact = burn::artifact::Artifact::load(&model_path)
        .expect("Failed to load model");
    let model = ImageClassifierModel::<B>::from_artifact(&artifact);
    
    // Load class names
    let class_names_path = Path::new(&model_path).with_extension("classes.json");
    let class_names: Vec<String> = if class_names_path.exists() {
        let class_names_json = std::fs::read_to_string(class_names_path)?;
        serde_json::from_str(&class_names_json)?
    } else {
        // If class names file doesn't exist, try to infer from data directory
        let dataset = data::load_image_dataset(&data_dir, image_size)?;
        dataset.class_names()
    };
    
    println!("Classes: {:?}", class_names);
    
    println!("Loading test images from {}...", data_dir);
    
    // Load the test dataset
    let dataset = data::load_image_dataset(&data_dir, image_size)?;
    
    println!("Found {} test images", dataset.len());
    
    // Create a data batcher
    let batcher = ImageBatcher::<B>::new(16);
    
    // Create a data loader
    let loader = dataset.into_loader(batcher, 16, false, None);
    
    // Create a validation step handler
    let mut valid_step = ValidationStepHandler::new(model);
    
    // Evaluation phase
    let mut metrics = ClassificationOutput::new();
    
    // Create a progress bar
    let progress = ProgressBar::new(loader.len() as u64);
    progress.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.green/blue} {pos}/{len} {msg}")
        .unwrap());
    progress.set_message("Evaluating");
    
    for batch in loader.iter() {
        // Perform one validation step
        let output = valid_step.step(&batch);
        metrics.extend(&output);
        progress.inc(1);
    }
    
    progress.finish();
    
    // Print evaluation results
    println!(
        "Test Loss: {:.4}, Test Accuracy: {:.4}",
        metrics.loss(),
        metrics.accuracy()
    );
    
    Ok(())
}

// Prediction function - classifies a single image
fn predict(model_path: String, image_path: String, image_size: usize) -> Result<()> {
    // We'll use the CPU for computations
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading model from {}...", model_path);
    
    // Load the saved model
    let artifact = burn::artifact::Artifact::load(&model_path)
        .expect("Failed to load model");
    let model = ImageClassifierModel::<B>::from_artifact(&artifact);
    
    // Load class names
    let class_names_path = Path::new(&model_path).with_extension("classes.json");
    let class_names: Vec<String> = if class_names_path.exists() {
        let class_names_json = std::fs::read_to_string(class_names_path)?;
        serde_json::from_str(&class_names_json)?
    } else {
        // If class names file doesn't exist, use generic class names
        (0..model.num_classes()).map(|i| format!("Class {}", i)).collect()
    };
    
    println!("Classes: {:?}", class_names);
    
    println!("Loading image from {}...", image_path);
    
    // Load and preprocess the image
    let image_tensor = data::load_single_image(&image_path, image_size)?;
    
    // Forward pass - get prediction from the model
    let output = model.forward(image_tensor);
    
    // Get the predicted class
    let prediction = output.argmax(1).into_scalar();
    
    // Get the confidence scores
    let confidences = output.softmax(1).into_data().value().to_vec();
    
    // Print the prediction
    println!("Prediction: {} ({})", class_names[prediction], prediction);
    println!("Confidence: {:.2}%", confidences[prediction] * 100.0);
    
    // Print top 3 predictions if we have more than 3 classes
    if class_names.len() > 3 {
        println!("Top predictions:");
        
        // Create a vector of (class_index, confidence) pairs
        let mut class_confidences: Vec<(usize, f32)> = confidences
            .iter()
            .enumerate()
            .map(|(i, &conf)| (i, conf))
            .collect();
        
        // Sort by confidence (descending)
        class_confidences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Print top 3
        for (i, (class_idx, conf)) in class_confidences.iter().take(3).enumerate() {
            println!("  {}. {} - {:.2}%", i + 1, class_names[*class_idx], conf * 100.0);
        }
    }
    
    Ok(())
}

// Training step handler - manages one step of training
struct TrainingStepHandler<B: Backend> {
    model: ImageClassifierModel<B>,
    optimizer: burn::optim::Adam<B>,
}

impl<B: Backend> TrainingStepHandler<B> {
    fn new(model: ImageClassifierModel<B>, optimizer: burn::optim::Adam<B>) -> Self {
        Self { model, optimizer }
    }
}

// Implement the TrainStep trait for our training handler
impl<B: Backend> TrainStep<ImageItem<B>, ClassificationOutput> for TrainingStepHandler<B> {
    fn step(&mut self, batch: &ImageItem<B>) -> TrainOutput<ClassificationOutput> {
        // Forward pass - get predictions from the model
        let output = self.model.forward_classification(batch.images.clone(), batch.targets.clone());
        
        // Backward pass - calculate gradients
        let grads = output.loss.backward();
        
        // Update model weights using the optimizer
        self.model = self.optimizer.step(&self.model, &grads);
        
        // Return training metrics
        TrainOutput::new(self.model.clone(), output)
    }
}

// Validation step handler - manages one step of validation
struct ValidationStepHandler<B: Backend> {
    model: ImageClassifierModel<B>,
}

impl<B: Backend> ValidationStepHandler<B> {
    fn new(model: ImageClassifierModel<B>) -> Self {
        Self { model }
    }
}

// Implement the ValidStep trait for our validation handler
impl<B: Backend> ValidStep<ImageItem<B>, ClassificationOutput> for ValidationStepHandler<B> {
    fn step(&mut self, batch: &ImageItem<B>) -> ClassificationOutput {
        // Forward pass - get predictions from the model
        self.model.forward_classification(batch.images.clone(), batch.targets.clone())
    }
}
