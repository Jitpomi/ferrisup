use anyhow::Result;
use burn::backend::Autodiff;
use burn::tensor::{Tensor, TensorData};
use burn_ndarray::NdArray;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Import our modules
use {{ project_name }}::{
    model::Model,
    training::{train, evaluate, create_dataloaders},
};

// Define the backend type
type Backend = Autodiff<NdArray<f32>>;

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
        #[arg(short, long, default_value = "10")]
        epochs: usize,
        
        #[arg(short, long, default_value = "64")]
        batch_size: usize,
        
        #[arg(short, long, default_value = "0.001")]
        learning_rate: f64,
        
        #[arg(short, long, default_value = "./model.json")]
        model_path: PathBuf,
    },
    
    // Evaluate an existing model
    Evaluate {
        #[arg(short, long)]
        model_path: PathBuf,
        
        #[arg(short, long, default_value = "64")]
        batch_size: usize,
    },
    
    // Predict using an existing model
    Predict {
        #[arg(short, long)]
        model_path: PathBuf,
        
        #[arg(short, long)]
        image_path: PathBuf,
    },
}

// Main function - entry point of our program
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Create a device for computation
    let device = Default::default();
    
    // Handle the different commands
    match cli.command {
        Commands::Train { epochs, batch_size, learning_rate, model_path } => {
            println!("🚀 Training a new MNIST digit recognition model");
            println!("📊 Epochs: {}", epochs);
            println!("📦 Batch size: {}", batch_size);
            println!("📈 Learning rate: {}", learning_rate);
            
            // Train the model
            let _trained_model = train::<Backend>(
                &device,
                epochs,
                batch_size,
                learning_rate,
                model_path.to_string_lossy().to_string(),
            );
            
            println!("✅ Training completed successfully!");
        },
        
        Commands::Evaluate { model_path, batch_size } => {
            println!("🔍 Evaluating MNIST digit recognition model");
            
            // Load the model
            let model = Model::<Backend>::load(model_path)?;
            
            // Create test dataloader (extract from tuple)
            let (_train_loader, mut test_loader) = create_dataloaders::<Backend>(&device, batch_size);
            
            // Evaluate the model
            let (accuracy, loss) = evaluate(&model, &mut test_loader);
            
            println!("📊 Test accuracy: {:.2}%", accuracy * 100.0);
            println!("📉 Test loss: {:.4}", loss);
        },
        
        Commands::Predict { model_path, image_path } => {
            println!("🔮 Predicting digit from image");
            
            // Load the model
            let model = Model::<Backend>::load(model_path)?;
            
            // Load and preprocess the image
            let image = image::open(image_path)?
                .to_luma8();
            
            // Resize to 28x28 if needed
            let image = if image.dimensions() != (28, 28) {
                image::imageops::resize(
                    &image,
                    28,
                    28,
                    image::imageops::FilterType::Nearest,
                )
            } else {
                image
            };
            
            // Convert to tensor
            let image_data: Vec<f32> = image
                .pixels()
                .map(|p| (p[0] as f32 / 255.0 - 0.1307) / 0.3081)
                .collect();
            
            // Create a 4D tensor [batch_size=1, channels=1, height=28, width=28]
            let tensor = Tensor::<Backend, 4>::from_data(
                TensorData::new(image_data, [1, 1, 28, 28]),
                &device,
            );
            
            // Make prediction
            let output = model.forward(tensor);
            
            // Get the predicted digit
            let prediction = output
                .argmax(1)
                .squeeze::<2>(1)
                .into_scalar();
            
            println!("✅ Predicted digit: {}", prediction);
        },
    }
    
    Ok(())
}
