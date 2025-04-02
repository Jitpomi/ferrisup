mod model;
mod dataset;
mod train;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use burn::tensor::backend::Backend;
use burn_ndarray::NdArray;

// Define the backend type
type MyBackend = NdArray<f32>;

/// A deep learning application using Burn for neural networks
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Train a neural network model
    Train {
        /// Number of epochs
        #[arg(short, long, default_value_t = 10)]
        epochs: usize,
        
        /// Batch size
        #[arg(short, long, default_value_t = 32)]
        batch_size: usize,
        
        /// Learning rate
        #[arg(short, long, default_value_t = 0.001)]
        learning_rate: f64,
        
        /// Path to save the model
        #[arg(short, long, default_value = "model.burn")]
        output: PathBuf,
        
        /// Use MNIST dataset
        #[arg(short, long, default_value_t = true)]
        mnist: bool,
    },
    
    /// Evaluate a trained model
    Evaluate {
        /// Path to the model file
        #[arg(short, long)]
        model: PathBuf,
        
        /// Path to the test image (if not specified, uses MNIST test set)
        #[arg(short, long)]
        image: Option<PathBuf>,
    },
    
    /// Make a prediction with a trained model
    Predict {
        /// Path to the model file
        #[arg(short, long)]
        model: PathBuf,
        
        /// Path to the image to predict
        #[arg(short, long)]
        image: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Train {
            epochs,
            batch_size,
            learning_rate,
            output,
            mnist,
        } => {
            println!("🧠 Training neural network model");
            println!("Epochs: {}", epochs);
            println!("Batch size: {}", batch_size);
            println!("Learning rate: {}", learning_rate);
            
            if *mnist {
                println!("Dataset: MNIST");
                train::train_mnist::<MyBackend>(
                    *epochs,
                    *batch_size,
                    *learning_rate as f32,
                    output,
                )?;
            } else {
                println!("Custom dataset not implemented yet. Using MNIST instead.");
                train::train_mnist::<MyBackend>(
                    *epochs,
                    *batch_size,
                    *learning_rate as f32,
                    output,
                )?;
            }
        },
        
        Commands::Evaluate { model, image } => {
            println!("📊 Evaluating model: {}", model.display());
            
            if let Some(img_path) = image {
                println!("Evaluating on single image: {}", img_path.display());
                train::evaluate_single::<MyBackend>(model, img_path)?;
            } else {
                println!("Evaluating on MNIST test set");
                train::evaluate_mnist::<MyBackend>(model)?;
            }
        },
        
        Commands::Predict { model, image } => {
            println!("🔮 Making prediction with model: {}", model.display());
            println!("Image: {}", image.display());
            
            let prediction = train::predict::<MyBackend>(model, image)?;
            println!("Prediction: {}", prediction);
        },
    }

    Ok(())
}
