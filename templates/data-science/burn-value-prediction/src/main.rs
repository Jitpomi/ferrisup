// Value Prediction Application
// This program trains a neural network to predict numeric values
// such as house prices, stock values, or temperatures

use burn::data::dataset::Dataset;
use burn::module::Module;
use burn::tensor::backend::Backend;
use burn::train::{RegressionOutput, TrainOutput, TrainStep, ValidStep};
use burn::config::Config;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Import our model and data handling code
mod model;
mod data;

use model::{RegressionConfig, RegressionModel};
use data::{RegressionBatcher, RegressionItem};

// Command line interface for our application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// Available commands: train or predict
#[derive(Subcommand)]
enum Commands {
    // Train a new model
    Train {
        // Number of training cycles
        #[arg(short, long, default_value_t = 100)]
        epochs: usize,
        
        // Path to the CSV data file
        #[arg(short, long, default_value = "data/housing.csv")]
        data: String,
        
        // Where to save the trained model
        #[arg(short, long, default_value = "model.json")]
        output: String,
        
        // How many examples to process at once
        #[arg(short, long, default_value_t = 32)]
        batch_size: usize,
    },
    
    // Make predictions with an existing model
    Predict {
        // Path to the saved model file
        #[arg(short, long)]
        model: String,
        
        // Path to the input data file
        #[arg(short, long)]
        input: String,
    },
}

// Main function - entry point of our program
fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Choose which command to run
    match cli.command {
        Commands::Train { epochs, data, output, batch_size } => {
            // Run the training process
            train(epochs, data, output, batch_size)?;
        }
        Commands::Predict { model, input } => {
            // Run the prediction process
            predict(model, input)?;
        }
    }
    
    Ok(())
}

// Training function - teaches our model to predict values
fn train(epochs: usize, data_path: String, output: String, batch_size: usize) -> anyhow::Result<()> {
    // We'll use the CPU for computations
    // You can change this to GPU if available
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading dataset from {}...", data_path);
    
    // Load the dataset from CSV
    let dataset = data::load_regression_dataset(&data_path)?;
    
    // Split into training and validation sets (80% train, 20% validation)
    let (train_data, valid_data) = dataset.split_by_ratio([0.8, 0.2]);
    
    // Create data batchers (group examples into batches)
    let train_batcher = RegressionBatcher::<B>::new(batch_size);
    let valid_batcher = RegressionBatcher::<B>::new(batch_size);
    
    // Create data loaders
    let train_loader = train_data.into_loader(train_batcher, batch_size, true, None);
    let valid_loader = valid_data.into_loader(valid_batcher, batch_size, false, None);
    
    println!("Creating model...");
    
    // Create a new model with default configuration
    let config = RegressionConfig::new(dataset.num_features());
    let mut model = RegressionModel::<B>::new(&config);
    
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
        let mut train_metrics = RegressionOutput::new();
        
        for batch in train_loader.iter() {
            // Perform one training step
            let output = train_step.step(&batch);
            train_metrics.extend(&output);
        }
        
        // Validation phase
        let mut valid_metrics = RegressionOutput::new();
        
        for batch in valid_loader.iter() {
            // Perform one validation step
            let output = valid_step.step(&batch);
            valid_metrics.extend(&output);
        }
        
        // Update the model with the best weights
        model = train_step.model.clone();
        
        // Print training progress (only every 10 epochs to reduce output)
        if epoch % 10 == 0 || epoch == 1 || epoch == epochs {
            println!(
                "Epoch: {}/{}, Train MSE: {:.4}, Valid MSE: {:.4}",
                epoch,
                epochs,
                train_metrics.mse(),
                valid_metrics.mse()
            );
        }
    }
    
    // Save the trained model
    println!("Saving model to {}...", output);
    let artifact = model.into_artifact();
    artifact.save(output).expect("Failed to save model");
    
    println!("Training complete!");
    
    Ok(())
}

// Prediction function - uses the trained model to make predictions
fn predict(model_path: String, input_path: String) -> anyhow::Result<()> {
    // We'll use the CPU for computations
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading model from {}...", model_path);
    
    // Load the saved model
    let artifact = burn::artifact::Artifact::load(model_path)
        .expect("Failed to load model");
    let model = RegressionModel::<B>::from_artifact(&artifact);
    
    println!("Loading test data from {}...", input_path);
    
    // Load the test dataset
    let dataset = data::load_regression_dataset(&input_path)?;
    
    // Create a data batcher
    let batcher = RegressionBatcher::<B>::new(1);
    
    // Create a data loader (batch size 1 for individual predictions)
    let loader = dataset.into_loader(batcher, 1, false, None);
    
    println!("Making predictions...");
    println!("Input Features | Predicted Value");
    println!("---------------|----------------");
    
    // Make predictions for each example
    for batch in loader.iter() {
        // Forward pass - get prediction from the model
        let prediction = model.forward(batch.features.clone());
        
        // Get the predicted value (a single number)
        let pred_value = prediction.into_scalar();
        
        // Print the features and prediction
        // In a real application, you might want to save these to a file
        println!("{:?} | {:.4}", batch.features.to_data().value(), pred_value);
    }
    
    println!("Prediction complete!");
    
    Ok(())
}

// Training step handler - manages one step of training
struct TrainingStepHandler<B: Backend> {
    model: RegressionModel<B>,
    optimizer: burn::optim::Adam<B>,
}

impl<B: Backend> TrainingStepHandler<B> {
    fn new(model: RegressionModel<B>, optimizer: burn::optim::Adam<B>) -> Self {
        Self { model, optimizer }
    }
}

// Implement the TrainStep trait for our training handler
impl<B: Backend> TrainStep<RegressionItem<B>, RegressionOutput> for TrainingStepHandler<B> {
    fn step(&mut self, batch: &RegressionItem<B>) -> TrainOutput<RegressionOutput> {
        // Forward pass - get predictions from the model
        let output = self.model.forward_regression(batch.features.clone(), batch.targets.clone());
        
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
    model: RegressionModel<B>,
}

impl<B: Backend> ValidationStepHandler<B> {
    fn new(model: RegressionModel<B>) -> Self {
        Self { model }
    }
}

// Implement the ValidStep trait for our validation handler
impl<B: Backend> ValidStep<RegressionItem<B>, RegressionOutput> for ValidationStepHandler<B> {
    fn step(&mut self, batch: &RegressionItem<B>) -> RegressionOutput {
        // Forward pass - get predictions from the model
        self.model.forward_regression(batch.features.clone(), batch.targets.clone())
    }
}
