// Image Recognition Application
// This program trains a neural network to recognize handwritten digits
// using the MNIST dataset

use burn::data::dataset::Dataset;
use burn::module::Module;
use burn::tensor::backend::Backend;
use burn::train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep};
use burn::config::Config;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Import our model and data handling code
mod model;
mod data;

use model::{MnistConfig, MnistModel};
use data::{MnistBatcher, MnistItem};

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
        // Number of training cycles
        #[arg(short, long, default_value_t = 10)]
        epochs: usize,
        
        // Where to save the trained model
        #[arg(short, long, default_value = "model.json")]
        output: String,
        
        // How many images to process at once
        #[arg(short, long, default_value_t = 32)]
        batch_size: usize,
    },
    
    // Evaluate an existing model
    Evaluate {
        // Path to the saved model file
        #[arg(short, long)]
        model: String,
    },
}

// Main function - entry point of our program
fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Choose which command to run
    match cli.command {
        Commands::Train { epochs, output, batch_size } => {
            // Run the training process
            train(epochs, output, batch_size);
        }
        Commands::Evaluate { model } => {
            // Run the evaluation process
            evaluate(model);
        }
    }
}

// Training function - teaches our model to recognize digits
fn train(epochs: usize, output: String, batch_size: usize) {
    // We'll use the CPU for computations
    // You can change this to GPU if available
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading MNIST dataset...");
    
    // Load the MNIST dataset
    let dataset = data::load_mnist_dataset();
    
    // Split into training and validation sets
    let (train_data, valid_data) = dataset.split_by_ratio([0.8, 0.2]);
    
    // Create data batchers (group images into batches)
    let train_batcher = MnistBatcher::<B>::new(batch_size);
    let valid_batcher = MnistBatcher::<B>::new(batch_size);
    
    // Create data loaders
    let train_loader = train_data.into_loader(train_batcher, batch_size, true, None);
    let valid_loader = valid_data.into_loader(valid_batcher, batch_size, false, None);
    
    println!("Creating model...");
    
    // Create a new model with default configuration
    let config = MnistConfig::new();
    let mut model = MnistModel::<B>::new(&config);
    
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
        // Training phase
        let mut train_metrics = ClassificationOutput::new();
        
        for batch in train_loader.iter() {
            // Perform one training step
            let output = train_step.step(&batch);
            train_metrics.extend(&output);
        }
        
        // Validation phase
        let mut valid_metrics = ClassificationOutput::new();
        
        for batch in valid_loader.iter() {
            // Perform one validation step
            let output = valid_step.step(&batch);
            valid_metrics.extend(&output);
        }
        
        // Update the model with the best weights
        model = train_step.model.clone();
        
        // Print training progress
        println!(
            "Epoch: {}/{}, Train Loss: {:.4}, Train Accuracy: {:.4}, Valid Loss: {:.4}, Valid Accuracy: {:.4}",
            epoch,
            epochs,
            train_metrics.loss(),
            train_metrics.accuracy(),
            valid_metrics.loss(),
            valid_metrics.accuracy()
        );
    }
    
    // Save the trained model
    println!("Saving model to {}...", output);
    let artifact = model.into_artifact();
    artifact.save(output).expect("Failed to save model");
    
    println!("Training complete!");
}

// Evaluation function - tests how well our model recognizes digits
fn evaluate(model_path: String) {
    // We'll use the CPU for computations
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading model from {}...", model_path);
    
    // Load the saved model
    let artifact = burn::artifact::Artifact::load(model_path)
        .expect("Failed to load model");
    let model = MnistModel::<B>::from_artifact(&artifact);
    
    println!("Loading MNIST test dataset...");
    
    // Load the MNIST test dataset
    let dataset = data::load_mnist_test_dataset();
    
    // Create a data batcher
    let batcher = MnistBatcher::<B>::new(32);
    
    // Create a data loader
    let loader = dataset.into_loader(batcher, 32, false, None);
    
    // Create a validation step handler
    let mut valid_step = ValidationStepHandler::new(model);
    
    // Evaluation phase
    let mut metrics = ClassificationOutput::new();
    
    for batch in loader.iter() {
        // Perform one validation step
        let output = valid_step.step(&batch);
        metrics.extend(&output);
    }
    
    // Print evaluation results
    println!(
        "Test Loss: {:.4}, Test Accuracy: {:.4}",
        metrics.loss(),
        metrics.accuracy()
    );
}

// Training step handler - manages one step of training
struct TrainingStepHandler<B: Backend> {
    model: MnistModel<B>,
    optimizer: burn::optim::Adam<B>,
}

impl<B: Backend> TrainingStepHandler<B> {
    fn new(model: MnistModel<B>, optimizer: burn::optim::Adam<B>) -> Self {
        Self { model, optimizer }
    }
}

// Implement the TrainStep trait for our training handler
impl<B: Backend> TrainStep<MnistItem<B>, ClassificationOutput> for TrainingStepHandler<B> {
    fn step(&mut self, batch: &MnistItem<B>) -> TrainOutput<ClassificationOutput> {
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
    model: MnistModel<B>,
}

impl<B: Backend> ValidationStepHandler<B> {
    fn new(model: MnistModel<B>) -> Self {
        Self { model }
    }
}

// Implement the ValidStep trait for our validation handler
impl<B: Backend> ValidStep<MnistItem<B>, ClassificationOutput> for ValidationStepHandler<B> {
    fn step(&mut self, batch: &MnistItem<B>) -> ClassificationOutput {
        // Forward pass - get predictions from the model
        self.model.forward_classification(batch.images.clone(), batch.targets.clone())
    }
}
