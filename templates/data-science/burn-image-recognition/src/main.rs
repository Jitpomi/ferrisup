use clap::{Parser, Subcommand};
use std::path::PathBuf;
use image;

mod model;
mod data;
mod training;

use data::{mnist_dataloader, normalize_mnist_pixel, MnistBatch};
use model::Model;
use training::{train, evaluate};
use burn_autodiff::Autodiff;
use burn::tensor::Tensor;
use burn::prelude::Backend;
use std::sync::Arc;
use burn::record::{Recorder, CompactRecorder};
use burn::module::Module;
use burn::data::dataloader::DataLoader;

// For training
// (Burn autodiff backend wrapper)
type TrainBackend = Autodiff<burn_ndarray::NdArray>;
type InferenceBackend = burn_ndarray::NdArray;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Train {
        #[arg(short, long, default_value = "10")]
        num_epochs: usize,
        #[arg(short, long, default_value = "64")]
        batch_size: usize,
        #[arg(short, long, default_value = "0.001")]
        learning_rate: f64,
        #[arg(short, long, default_value = "./model.json")]
        model_path: PathBuf,
    },
    Evaluate {
        #[arg(short, long)]
        model_path: PathBuf,
        #[arg(short, long, default_value = "64")]
        batch_size: usize,
    },
    Predict {
        #[arg(short, long)]
        model_path: PathBuf,
        #[arg(short, long)]
        image_path: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Train { num_epochs, batch_size, learning_rate, model_path } => {
            println!("🚀 Training MNIST digit recognition model");
            // Check for MNIST data presence
            if !std::path::Path::new("./data/mnist/train-images-idx3-ubyte").exists() {
                eprintln!("❌ MNIST data not found. Please run ./download_mnist.sh before training.");
                std::process::exit(1);
            }
            let device = <TrainBackend as Backend>::Device::default();
            train::<TrainBackend>(
                &device,
                num_epochs,
                batch_size,
                learning_rate,
                model_path.to_string_lossy().to_string(),
            );
            println!("✅ Training completed successfully!");
        },
        Commands::Evaluate { model_path, batch_size } => {
            println!("🔍 Evaluating MNIST digit recognition model");
            // Check for model file presence
            if !model_path.exists() {
                eprintln!("❌ Model file not found: {}", model_path.display());
                std::process::exit(1);
            }
            let device = <InferenceBackend as Backend>::Device::default();
            let mut model = Model::<InferenceBackend>::new(&device);
            let record = CompactRecorder::new().load(model_path, &device)?;
            model = model.load_record(record);
            let test_loader: Arc<dyn DataLoader<MnistBatch<InferenceBackend>>> = mnist_dataloader::<InferenceBackend>(false, device.clone(), batch_size, None, 2);
            let (loss, accuracy) = evaluate::<InferenceBackend>(&model, test_loader.as_ref());
            println!("📊 Test accuracy: {:.2}%", accuracy * 100.0);
            println!("📉 Test loss: {:.4}", loss);
        },
        Commands::Predict { model_path, image_path } => {
            println!("🔮 Predicting digit from image");
            // Check for model file presence
            if !model_path.exists() {
                eprintln!("❌ Model file not found: {}", model_path.display());
                std::process::exit(1);
            }
            if !image_path.exists() {
                eprintln!("❌ Image file not found: {}", image_path.display());
                std::process::exit(1);
            }
            let device = <InferenceBackend as Backend>::Device::default();
            let mut model = Model::<InferenceBackend>::new(&device);
            let record = CompactRecorder::new().load(model_path, &device)?;
            model = model.load_record(record);
            let image = image::open(image_path)?.to_luma8();
            let image = if image.dimensions() != (28, 28) {
                image::imageops::resize(&image, 28, 28, image::imageops::FilterType::Nearest)
            } else {
                image
            };
            let image_data: Vec<f32> = image.pixels().map(|p| normalize_mnist_pixel(p[0])).collect();
            let input = Tensor::<InferenceBackend, 3>::from_floats(image_data.as_slice(), &device);
            let output = model.forward(input);
            let pred_data = output.argmax(1).to_data();
            let pred_slice = pred_data.as_slice().unwrap_or(&[0]);
            let pred = pred_slice[0];
            println!("Predicted digit: {}", pred);
        }
    }
    Ok(())
}
