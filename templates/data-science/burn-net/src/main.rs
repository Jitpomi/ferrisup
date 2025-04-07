mod data;
mod model;

use burn::{
    config::Config,
    data::{dataloader::DataLoaderBuilder, dataset::vision::MnistDataset},
    module::Module,
    optim::AdamConfig,
    prelude::*,
    record::{CompactRecorder, Recorder, NoStdTrainingRecorder},
    tensor::backend::AutodiffBackend,
    train::{
        metric::{AccuracyMetric, LossMetric},
        LearnerBuilder,
    },
};
use clap::{Parser, Subcommand};
use data::{MnistBatch, MnistBatcher};
use model::Model;
use std::path::PathBuf;

/// A deep learning application using the Burn framework
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
        /// Number of epochs for training
        #[arg(short, long, default_value_t = 10)]
        epochs: usize,
        
        /// Path to save the model
        #[arg(short, long, default_value = "model.json")]
        output: PathBuf,
        
        /// Batch size for training
        #[arg(short, long, default_value_t = 32)]
        batch_size: usize,
    },
    
    /// Evaluate a trained model
    Evaluate {
        /// Path to the model file
        #[arg(short, long)]
        model: PathBuf,
    },
}

#[derive(Config)]
struct TrainingConfig {
    #[config(default = 10)]
    pub num_epochs: usize,

    #[config(default = 32)]
    pub batch_size: usize,

    #[config(default = 4)]
    pub num_workers: usize,

    #[config(default = 3e-4)]
    pub learning_rate: f64,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Train { epochs, output, batch_size } => {
            println!("Training a neural network model for {} epochs", epochs);
            
            // Create a type alias for the backend
            type Backend = burn::backend::ndarray::NdArray<f32>;
            
            // Initialize the device
            let device = Default::default();
            
            // Create the training config
            let config = TrainingConfig {
                num_epochs: *epochs,
                batch_size: *batch_size,
                ..Default::default()
            };
            
            // Create the model
            let model = Model::<Backend>::new(&device);
            
            // Create the optimizer
            let optimizer = AdamConfig::new().with_learning_rate(config.learning_rate).init();
            
            // Create the dataloaders
            let batcher_train = MnistBatcher::<Backend>::new(device.clone());
            let batcher_valid = MnistBatcher::<Backend>::new(device.clone());
            
            // Load the MNIST dataset
            let dataset_train = MnistDataset::train();
            let dataset_valid = MnistDataset::test();
            
            // Create the dataloaders
            let dataloader_train = DataLoaderBuilder::new(batcher_train)
                .batch_size(config.batch_size)
                .shuffle(true)
                .num_workers(config.num_workers)
                .build(dataset_train);
            
            let dataloader_valid = DataLoaderBuilder::new(batcher_valid)
                .batch_size(config.batch_size)
                .shuffle(false)
                .num_workers(config.num_workers)
                .build(dataset_valid);
            
            // Create the learner
            let mut learner = LearnerBuilder::new(output.to_str().unwrap())
                .metric_train(AccuracyMetric::new())
                .metric_valid(AccuracyMetric::new())
                .metric_train(LossMetric::new())
                .metric_valid(LossMetric::new())
                .with_epochs(config.num_epochs)
                .build(model, optimizer);
            
            // Train the model
            println!("Starting training...");
            let model_trained = learner.fit(dataloader_train, dataloader_valid);
            
            // Save the model
            println!("Saving model to {:?}", output);
            let recorder = NoStdTrainingRecorder::new();
            recorder.record(model_trained).save(output.to_str().unwrap()).expect("Failed to save model");
            
            println!("✅ Training completed successfully!");
        },
        Commands::Evaluate { model } => {
            println!("Evaluating model from {:?}", model);
            
            // Create a type alias for the backend
            type Backend = burn::backend::ndarray::NdArray<f32>;
            
            // Initialize the device
            let device = Default::default();
            
            // Load the model
            let recorder = NoStdTrainingRecorder::new();
            let model: Model<Backend> = recorder.load(model.to_str().unwrap()).expect("Failed to load model");
            
            // Create the batcher
            let batcher = MnistBatcher::<Backend>::new(device.clone());
            
            // Load the MNIST dataset
            let dataset = MnistDataset::test();
            
            // Create the dataloader
            let dataloader = DataLoaderBuilder::new(batcher)
                .batch_size(32)
                .shuffle(false)
                .num_workers(4)
                .build(dataset);
            
            // Evaluate the model
            let mut total = 0;
            let mut correct = 0;
            
            for batch in dataloader {
                let output = model.forward(batch.images);
                let predictions = output.argmax(1);
                let targets = batch.targets;
                
                for i in 0..predictions.dims()[0] {
                    let pred = predictions.get(i).into_scalar() as usize;
                    let target = targets.get(i).into_scalar() as usize;
                    
                    if pred == target {
                        correct += 1;
                    }
                    total += 1;
                }
            }
            
            let accuracy = (correct as f32) / (total as f32) * 100.0;
            println!("Test accuracy: {:.2}%", accuracy);
        },
    }
}
