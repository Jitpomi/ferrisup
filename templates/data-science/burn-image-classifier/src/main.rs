// Image Classifier Application
// This program trains a neural network to classify images into categories

use burn::data::dataset::Dataset;
use burn::module::Module;
use burn::tensor::backend::Backend;
use burn::train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep};
use burn::optim::Adam;
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

// Import our modules
mod model;
mod data;
mod config;

use model::{ImageClassifierConfig, ImageClassifierModel};
use data::{ImageBatcher, ImageItem, ImageDataset, load_image_dataset};
use config::{BATCH_SIZE, LEARNING_RATE, EPOCHS, IMAGE_SIZE, DEFAULT_DATA_DIR, DEFAULT_MODEL_FILE};

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
        #[arg(long, default_value = DEFAULT_DATA_DIR)]
        data_dir: String,
        
        // Number of training cycles
        #[arg(short, long, default_value_t = EPOCHS)]
        epochs: usize,
        
        // Where to save the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        output: String,
        
        // How many images to process at once
        #[arg(short, long, default_value_t = BATCH_SIZE)]
        batch_size: usize,
    },
    
    // Evaluate an existing model
    Evaluate {
        // Path to the saved model file
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model: String,
        
        // Directory containing test images organized in category folders
        #[arg(long, default_value = DEFAULT_DATA_DIR)]
        data_dir: String,
    },
    
    // Predict the category of a single image
    Predict {
        // Path to the saved model file
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model: String,
        
        // Path to the image to classify
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
        Commands::Train { data_dir, epochs, output, batch_size } => {
            // Run the training process
            train(data_dir, epochs, output, batch_size)?;
        }
        Commands::Evaluate { model, data_dir } => {
            // Run the evaluation process
            evaluate(model, data_dir)?;
        }
        Commands::Predict { model, image } => {
            // Run the prediction process
            predict(model, image)?;
        }
    }
    
    Ok(())
}

// Training function - teaches our model to classify images
fn train(data_dir: String, epochs: usize, output: String, batch_size: usize) -> Result<()> {
    // CUSTOMIZE HERE: Choose your backend (CPU, CUDA, etc.)
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading images from {}...", data_dir);
    
    // Load the image dataset
    let dataset = load_image_dataset(&data_dir, IMAGE_SIZE)?;
    
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
    let model = ImageClassifierModel::<B>::new(&config);
    
    // Create an optimizer (Adam) to adjust model weights during training
    let optimizer = Adam::new(LEARNING_RATE);
    
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
        
        println!("");
    }
    
    // Save the trained model
    println!("Saving model to {}...", output);
    let model_artifact = valid_step.model.into_artifact();
    model_artifact.save(output)?;
    
    println!("Training complete!");
    
    Ok(())
}

// Evaluation function - tests how well our model classifies images
fn evaluate(model_path: String, data_dir: String) -> Result<()> {
    // CUSTOMIZE HERE: Choose your backend (CPU, CUDA, etc.)
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading model from {}...", model_path);
    
    // Load the trained model
    let artifact = burn::artifact::load(model_path)?;
    let model = ImageClassifierModel::<B>::from_artifact(&artifact);
    
    println!("Loading images from {}...", data_dir);
    
    // Load the test dataset
    let dataset = load_image_dataset(&data_dir, IMAGE_SIZE)?;
    let class_names = dataset.class_names();
    
    println!("Found {} images in {} classes", dataset.len(), class_names.len());
    
    // Create a data batcher
    let batcher = ImageBatcher::<B>::new(BATCH_SIZE);
    
    // Create a data loader
    let loader = dataset.into_loader(batcher, BATCH_SIZE, false, None);
    
    // Create a validation step handler
    let mut valid_step = ValidationStepHandler::new(model);
    
    // Evaluation metrics
    let mut metrics = ClassificationOutput::new();
    
    // Create a progress bar
    let progress = ProgressBar::new(loader.len() as u64);
    progress.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
        .unwrap());
    progress.set_message("Evaluating");
    
    // Process each batch
    for batch in loader.iter() {
        // Perform one validation step
        let output = valid_step.step(&batch);
        metrics.extend(&output);
        progress.inc(1);
    }
    
    progress.finish();
    
    // Print the evaluation results
    println!("\nEvaluation Results:");
    println!("Loss: {:.4}", metrics.loss());
    println!("Accuracy: {:.4}", metrics.accuracy());
    
    Ok(())
}

// Prediction function - classifies a single image
fn predict(model_path: String, image_path: String) -> Result<()> {
    // CUSTOMIZE HERE: Choose your backend (CPU, CUDA, etc.)
    type B = burn::backend::ndarray::NdArray;
    
    println!("Loading model from {}...", model_path);
    
    // Load the trained model
    let artifact = burn::artifact::load(model_path)?;
    let model = ImageClassifierModel::<B>::from_artifact(&artifact);
    
    println!("Loading image from {}...", image_path);
    
    // Load and process the image
    let img = image::open(&image_path)?;
    let img = img.resize_exact(
        IMAGE_SIZE as u32,
        IMAGE_SIZE as u32,
        image::imageops::FilterType::Triangle
    );
    
    // Convert image to tensor data
    let image_tensor = data::image_to_tensor(&img);
    
    // Create a batch with a single image
    let mut images_data = burn::tensor::Data::new(
        image_tensor,
        [1, 3, IMAGE_SIZE, IMAGE_SIZE],
    );
    
    // Create tensor from the data
    let images = burn::tensor::Tensor::<B, 4>::from_data(images_data);
    
    // Run the model to get predictions
    let output = model.forward(images);
    
    // Get the predicted class
    let predictions = output.to_data();
    let mut max_score = 0.0;
    let mut predicted_class = 0;
    
    // Find the class with the highest score
    for (i, &score) in predictions.value().iter().enumerate() {
        if score > max_score {
            max_score = score;
            predicted_class = i;
        }
    }
    
    // Get the class name
    let class_name = if predicted_class < config::CLASS_NAMES.len() {
        config::CLASS_NAMES[predicted_class]
    } else {
        "Unknown"
    };
    
    // Print the prediction results
    println!("\nPrediction Results:");
    println!("Predicted class: {} ({})", class_name, predicted_class);
    println!("Confidence: {:.2}%", max_score * 100.0);
    
    Ok(())
}

// Training step handler - manages one step of training
struct TrainingStepHandler<B: Backend> {
    model: ImageClassifierModel<B>,
    optimizer: Adam<B>,
}

impl<B: Backend> TrainingStepHandler<B> {
    fn new(model: ImageClassifierModel<B>, optimizer: Adam<B>) -> Self {
        Self { model, optimizer }
    }
}

// Implement the TrainStep trait for our training handler
impl<B: Backend> TrainStep<ImageItem<B>, ClassificationOutput> for TrainingStepHandler<B> {
    fn step(&mut self, batch: &ImageItem<B>) -> TrainOutput<ClassificationOutput> {
        // Run a forward pass through the model
        let output = self.model.forward(batch.images.clone());
        
        // Calculate the loss (how wrong the predictions are)
        let loss = output.cross_entropy_with_logits(&batch.labels);
        
        // Calculate the gradients
        let grads = loss.backward();
        
        // Update the model parameters
        self.optimizer.update(&mut self.model, &grads);
        
        // Calculate accuracy
        let accuracy = output.accuracy(&batch.labels);
        
        // Return the metrics
        TrainOutput::new(
            loss.into_scalar(),
            ClassificationOutput::new_with_accuracy(loss.into_scalar(), accuracy.into_scalar()),
        )
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
        // Run a forward pass through the model
        let output = self.model.forward(batch.images.clone());
        
        // Calculate the loss (how wrong the predictions are)
        let loss = output.cross_entropy_with_logits(&batch.labels);
        
        // Calculate accuracy
        let accuracy = output.accuracy(&batch.labels);
        
        // Return the metrics
        ClassificationOutput::new_with_accuracy(loss.into_scalar(), accuracy.into_scalar())
    }
}
