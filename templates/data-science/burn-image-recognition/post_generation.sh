#!/bin/bash
set -e

echo "Running post-generation fixes for Burn Image Recognition project..."

# Make the download scripts executable
chmod +x download_mnist.sh
chmod +x download_sample_images.sh

# Create a placeholder model.json file if it doesn't exist
if [ ! -f model.json ]; then
    echo "Creating placeholder model.json file..."
    echo "{}" > model.json
fi

# Create data directories
mkdir -p data/mnist
mkdir -p sample_images

# Fix Cargo.toml to ensure no duplicate dependencies
if [ -f Cargo.toml ]; then
    echo "Ensuring Cargo.toml has all required dependencies..."
    
    # Create a clean version of Cargo.toml without duplicates
    TMP_FILE=$(mktemp)
    cat > "$TMP_FILE" << 'EOL'
[package]
name = "{{project_name}}"
version = "0.1.0"
edition = "2021"

[dependencies]
burn = { version = "0.17.0", features = ["ndarray", "autodiff", "train"] }
burn-tensor = { version = "0.17.0" }
burn-train = { version = "0.17.0" }
burn-autodiff = { version = "0.17.0" }
burn-ndarray = { version = "0.17.0" }
# Uncomment one of the following backends based on your hardware
# burn-wgpu = { version = "0.17.0", features = ["metal"] } # For macOS with Metal
# burn-cuda = { version = "0.17.0" } # For NVIDIA GPUs
# burn-wgpu = { version = "0.17.0", features = ["vulkan"] } # For Vulkan support
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
clap = { version = "4.4", features = ["derive"] }
indicatif = "0.17.7"
image = "0.24.7"

[lib]
path = "src/lib.rs"

[[bin]]
name = "app"
path = "src/main.rs"
EOL

    # Replace project name placeholder with actual project name
    PROJECT_NAME=$(grep "^name" Cargo.toml | head -1 | cut -d '"' -f 2 || echo "demo")
    sed -i.bak "s/{{project_name}}/$PROJECT_NAME/g" "$TMP_FILE"
    mv "$TMP_FILE" Cargo.toml
    rm -f Cargo.toml.bak
fi

# Fix the data.rs file to use the correct MnistDataset implementation
cat > src/data.rs << 'EOL'
use burn::data::dataset::Dataset;
use burn::data::dataloader::{DataLoader, DataLoaderBuilder, batcher::Batcher};
use burn::tensor::{backend::Backend, Int, Tensor, Data, Shape};
use burn::prelude::*;
use std::path::Path;
use std::sync::Arc;

/// Normalize a single MNIST pixel value (u8 or f32) to f32 with PyTorch stats.
pub fn normalize_mnist_pixel<T: Into<f32>>(pixel: T) -> f32 {
    ((pixel.into() / 255.0) - 0.1307) / 0.3081
}

#[derive(Debug, Clone)]
pub struct MnistItem {
    pub image: Vec<f32>,
    pub label: usize,
}

#[derive(Debug, Clone)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 3>,
    pub targets: Tensor<B, 1, Int>,
}

#[derive(Debug, Clone)]
pub struct MnistBatcher;

impl MnistBatcher {
    pub fn new() -> Self {
        Self
    }
}

impl<B: Backend> Batcher<B, MnistItem, MnistBatch<B>> for MnistBatcher {
    fn batch(&self, items: Vec<MnistItem>, device: &B::Device) -> MnistBatch<B> {
        let batch_size = items.len();
        
        // Create a flat vector of all pixel values
        let mut image_data = Vec::with_capacity(batch_size * 28 * 28);
        for item in &items {
            image_data.extend_from_slice(&item.image);
        }
        
        // Create the images tensor
        let images = Tensor::<B, 3>::from_data(
            Data::new(image_data, Shape::new([batch_size, 28, 28])),
            device
        );

        // Create the targets tensor
        let targets = Tensor::<B, 1, Int>::from_data(
            Data::new(items.iter().map(|item| item.label as i64).collect::<Vec<_>>(), Shape::new([batch_size])),
            device
        );

        MnistBatch { images, targets }
    }
}

pub struct MnistDataset {
    images: Vec<MnistItem>,
}

impl MnistDataset {
    pub fn new(images: Vec<MnistItem>) -> Self {
        Self { images }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let images = std::fs::read(path.join("train-images-idx3-ubyte")).unwrap();
        let labels = std::fs::read(path.join("train-labels-idx1-ubyte")).unwrap();

        // Skip the header bytes: 16 for images, 8 for labels
        let images = &images[16..];
        let labels = &labels[8..];

        let images = images
            .chunks(28 * 28)
            .zip(labels.iter())
            .map(|(chunk, &label)| {
                let values = chunk
                    .iter()
                    .map(|&b| normalize_mnist_pixel(b))
                    .collect::<Vec<_>>();
                
                MnistItem {
                    image: values,
                    label: label as usize,
                }
            })
            .collect::<Vec<_>>();

        Self { images }
    }
    
    pub fn train() -> Self {
        Self::from_path("data/mnist")
    }
    
    pub fn test() -> Self {
        Self::from_path("data/mnist")
    }
}

impl Dataset<MnistItem> for MnistDataset {
    fn get(&self, index: usize) -> Option<MnistItem> {
        self.images.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.images.len()
    }
}

/// Build a `DataLoader` for MNIST training or testing data.
pub fn mnist_dataloader<B: Backend + 'static>(
    train: bool,
    device: &B::Device,
    batch_size: usize,
    shuffle: Option<u64>,
    num_workers: usize,
) -> Arc<dyn DataLoader<MnistBatch<B>>> {
    let dataset = if train {
        MnistDataset::train()
    } else {
        MnistDataset::test()
    };
    let batcher = MnistBatcher::new();
    let mut builder = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .num_workers(num_workers)
        .set_device(device.clone());
    
    if let Some(seed) = shuffle {
        builder = builder.shuffle(seed);
    }
    
    builder.build(dataset)
}
EOL

# Fix the model.rs file to use the correct imports and module structure
cat > src/model.rs << 'EOL'
use crate::data::MnistBatch;
use burn::{
    module::Module,
    nn,
    nn::{loss::CrossEntropyLossConfig, BatchNorm, PaddingConfig2d},
    tensor::backend::Backend,
    tensor::backend::AutodiffBackend,
    tensor::Tensor,
    train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep},
    record::Record,
    prelude::*,
};

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    conv1: ConvBlock<B>,
    conv2: ConvBlock<B>,
    conv3: ConvBlock<B>,
    dropout: nn::Dropout,
    fc1: nn::Linear<B>,
    fc2: nn::Linear<B>,
    activation: nn::Gelu,
}

const NUM_CLASSES: usize = 10;

impl<B: Backend> Model<B> {
    pub fn new(device: &B::Device) -> Self {
        let conv1 = ConvBlock::new([1, 8], [3, 3], device); // out: [Batch,8,26,26]
        let conv2 = ConvBlock::new([8, 16], [3, 3], device); // out: [Batch,16,24x24]
        let conv3 = ConvBlock::new([16, 24], [3, 3], device); // out: [Batch,24,22x22]
        let hidden_size = 24 * 22 * 22;
        let fc1 = nn::LinearConfig::new(hidden_size, 32)
            .with_bias(false)
            .init(device);
        let fc2 = nn::LinearConfig::new(32, NUM_CLASSES)
            .with_bias(false)
            .init(device);

        let dropout = nn::DropoutConfig::new(0.5).init();

        Self {
            conv1,
            conv2,
            conv3,
            dropout,
            fc1,
            fc2,
            activation: nn::Gelu::new(),
        }
    }

    pub fn forward(&self, input: &Tensor<B, 3>) -> Tensor<B, 2> {
        let [batch_size, height, width] = input.dims();
        
        let x = input.clone().reshape([batch_size, 1, height, width]);
        
        let x = self.conv1.forward(&x);
        let x = self.conv2.forward(&x);
        let x = self.conv3.forward(&x);
        
        let [batch_size, channels, height, width] = x.dims();
        
        let x = x.reshape([batch_size, channels * height * width]);
        
        let x = self.dropout.forward(x);
        let x = self.fc1.forward(x);
        let x = self.activation.forward(x);
        let x = self.fc2.forward(x);
        
        x
    }

    pub fn forward_classification(&self, item: MnistBatch<B>) -> ClassificationOutput<B> {
        let targets = item.targets;
        let output = self.forward(&item.images);
        let loss = CrossEntropyLossConfig::new()
            .init(&output.device())
            .forward(output.clone(), targets.clone());

        ClassificationOutput {
            loss,
            output,
            targets,
        }
    }

    pub fn from_record(record: &impl Record, device: &B::Device) -> Self {
        Module::from_record(record, device)
    }
}

#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    conv: nn::conv::Conv2d<B>,
    norm: BatchNorm<B, 2>,
    activation: nn::Gelu,
}

impl<B: Backend> ConvBlock<B> {
    pub fn new(channels: [usize; 2], kernel_size: [usize; 2], device: &B::Device) -> Self {
        let conv = nn::conv::Conv2dConfig::new(channels, kernel_size)
            .with_padding(PaddingConfig2d::Valid)
            .init(device);
        let norm = nn::BatchNormConfig::new(channels[1]).init(device);

        Self {
            conv,
            norm,
            activation: nn::Gelu::new(),
        }
    }

    pub fn forward(&self, input: &Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(input.clone());
        let x = self.norm.forward(x);

        self.activation.forward(x)
    }
}

impl<B: AutodiffBackend> TrainStep<MnistBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, item: MnistBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(item);

        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<MnistBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, item: MnistBatch<B>) -> ClassificationOutput<B> {
        self.forward_classification(item)
    }
}
EOL

# Fix the training.rs file to use the correct imports and training loop
cat > src/training.rs << 'EOL'
use crate::data::{MnistBatch, mnist_dataloader};
use crate::model::Model;
use burn::{
    tensor::backend::Backend,
    train::{
        ClassificationOutput, LearningRate, Optimizer, OptimizerConfig, StepOutput, TrainStep, ValidStep,
    },
    record::{Recorder, CompactRecorder},
    prelude::*,
};
use std::sync::Arc;

pub fn train<B: burn::tensor::backend::AutodiffBackend>(
    device: &B::Device,
    num_epochs: usize,
    batch_size: usize,
    learning_rate: f64,
    model_path: String,
) {
    // Create the model and optimizer
    let model = Model::new(device);
    let mut optimizer = burn::optim::AdamConfig::new()
        .with_learning_rate(learning_rate)
        .with_weight_decay(1e-5)
        .init();

    // Create the training and validation data loaders
    let train_loader = mnist_dataloader::<B>(true, device, batch_size, Some(42), 2);
    let valid_loader = mnist_dataloader::<B>(false, device, batch_size, None, 2);

    // Initialize the recorder
    let mut recorder = CompactRecorder::new();

    // Training loop
    for epoch in 0..num_epochs {
        let mut train_loss = 0.0;
        let mut train_acc = 0.0;
        let mut train_batches = 0;

        // Training
        for batch in train_loader.iter() {
            let output = model.step(batch);
            let batch_loss = output.loss.clone().into_scalar();
            let batch_accuracy = accuracy(output.item);

            train_loss += batch_loss;
            train_acc += batch_accuracy;
            train_batches += 1;

            // Update the model
            optimizer.update(&mut *output.model, output.gradients);

            // Print progress
            if train_batches % 100 == 0 {
                println!(
                    "Epoch: {}/{}, Batch: {}, Loss: {:.4}, Accuracy: {:.2}%",
                    epoch + 1,
                    num_epochs,
                    train_batches,
                    train_loss / train_batches as f64,
                    train_acc * 100.0 / train_batches as f64
                );
            }
        }

        // Calculate average training metrics
        train_loss /= train_batches as f64;
        train_acc /= train_batches as f64;

        // Validation
        let (val_loss, val_acc) = evaluate::<B>(&model, valid_loader.as_ref());

        println!(
            "Epoch: {}/{}, Train Loss: {:.4}, Train Acc: {:.2}%, Val Loss: {:.4}, Val Acc: {:.2}%",
            epoch + 1,
            num_epochs,
            train_loss,
            train_acc * 100.0,
            val_loss,
            val_acc * 100.0
        );
    }

    // Save the model
    recorder.record(&model);
    recorder.save(model_path).expect("Failed to save model");
}

pub fn evaluate<B: Backend>(
    model: &Model<B>,
    loader: &dyn burn::data::dataloader::DataLoader<MnistBatch<B>>,
) -> (f64, f64) {
    let mut total_loss = 0.0;
    let mut total_acc = 0.0;
    let mut num_batches = 0;

    for batch in loader.iter() {
        let output = model.step(batch);
        let batch_loss = output.loss.into_scalar();
        let batch_accuracy = accuracy(output);

        total_loss += batch_loss;
        total_acc += batch_accuracy;
        num_batches += 1;
    }

    (total_loss / num_batches as f64, total_acc / num_batches as f64)
}

fn accuracy<B: Backend>(output: ClassificationOutput<B>) -> f64 {
    let predictions = output.output.argmax(1);
    let targets = output.targets;
    
    let pred_data = predictions.to_data();
    let target_data = targets.to_data();
    
    let pred = pred_data.as_slice::<i64>().unwrap();
    let target = target_data.as_slice::<i64>().unwrap();
    
    let correct = pred.iter().zip(target.iter()).filter(|&(a, b)| a == b).count();
    correct as f64 / pred.len() as f64
}
EOL

# Fix the main.rs file to use the correct imports and command structure
cat > src/main.rs << 'EOL'
#![recursion_limit = "256"] // Required for WGPU backends

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use image;

mod model;
mod data;
mod training;

use data::{MnistBatch, normalize_mnist_pixel};
use model::Model;
use training::{train, evaluate};
use burn::tensor::{backend::Backend, Tensor, Data, Shape};
use burn::prelude::*;
use std::sync::Arc;
use burn::record::{Recorder, CompactRecorder};
use burn::module::Module;
use burn::data::dataloader::DataLoader;

// Choose your preferred backend by uncommenting one of these sections:

// For CPU (NdArray backend)
type MyBackend = burn_ndarray::NdArray;
type MyAutodiffBackend = burn_autodiff::Autodiff<MyBackend>;
// End CPU section

// For Metal (macOS)
// type MyBackend = burn_wgpu::Wgpu<burn_wgpu::metal::Metal>;
// type MyAutodiffBackend = burn_autodiff::Autodiff<MyBackend>;
// End Metal section

// For CUDA (NVIDIA GPUs)
// type MyBackend = burn_cuda::Cuda;
// type MyAutodiffBackend = burn_autodiff::Autodiff<MyBackend>;
// End CUDA section

// For Vulkan
// type MyBackend = burn_wgpu::Wgpu<burn_wgpu::vulkan::Vulkan>;
// type MyAutodiffBackend = burn_autodiff::Autodiff<MyBackend>;
// End Vulkan section

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
    
    // Create the appropriate device based on the selected backend
    let device = <MyBackend as Backend>::Device::default();
    
    match cli.command {
        Commands::Train { num_epochs, batch_size, learning_rate, model_path } => {
            println!("🚀 Training MNIST digit recognition model");
            // Check for MNIST data presence
            if !std::path::Path::new("./data/mnist/train-images-idx3-ubyte").exists() {
                eprintln!("❌ MNIST data not found. Please run ./download_mnist.sh before training.");
                std::process::exit(1);
            }
            
            train::<MyAutodiffBackend>(
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
            
            let record = CompactRecorder::new().load(model_path.to_path_buf(), &device)?;
            let model = Model::<MyBackend>::from_record(&record, &device);
            let test_loader: Arc<dyn DataLoader<MnistBatch<MyBackend>>> = 
                data::mnist_dataloader::<MyBackend>(false, &device, batch_size, None, 2);
            
            let (loss, accuracy) = evaluate::<MyBackend>(&model, test_loader.as_ref());
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
            
            let record = CompactRecorder::new().load(model_path.to_path_buf(), &device)?;
            let model = Model::<MyBackend>::from_record(&record, &device);
            let image = image::open(image_path)?.to_luma8();
            let image = if image.dimensions() != (28, 28) {
                image::imageops::resize(&image, 28, 28, image::imageops::FilterType::Nearest)
            } else {
                image
            };
            
            let image_data: Vec<f32> = image.pixels().map(|p| normalize_mnist_pixel(p[0])).collect();
            let input = Tensor::<MyBackend, 3>::from_data(
                Data::new(image_data, Shape::new([1, 28, 28])),
                &device
            );
            
            let output = model.forward(&input);
            let pred_data = output.argmax(1).to_data();
            let pred_slice = pred_data.as_slice::<i64>().unwrap_or(&[0]);
            let pred = pred_slice[0];
            println!("Predicted digit: {}", pred);
        }
    }
    Ok(())
}
EOL

# Fix the download_mnist.sh script to properly download and extract the MNIST dataset
cat > download_mnist.sh << 'EOL'
#!/bin/bash
set -e

echo "Downloading MNIST dataset..."

# Create data directory if it doesn't exist
mkdir -p data/mnist

# Download MNIST dataset files
wget -nc -q http://yann.lecun.com/exdb/mnist/train-images-idx3-ubyte.gz -O data/mnist/train-images-idx3-ubyte.gz
wget -nc -q http://yann.lecun.com/exdb/mnist/train-labels-idx1-ubyte.gz -O data/mnist/train-labels-idx1-ubyte.gz
wget -nc -q http://yann.lecun.com/exdb/mnist/t10k-images-idx3-ubyte.gz -O data/mnist/t10k-images-idx3-ubyte.gz
wget -nc -q http://yann.lecun.com/exdb/mnist/t10k-labels-idx1-ubyte.gz -O data/mnist/t10k-labels-idx1-ubyte.gz

# Extract the files
gunzip -f data/mnist/train-images-idx3-ubyte.gz
gunzip -f data/mnist/train-labels-idx1-ubyte.gz
gunzip -f data/mnist/t10k-images-idx3-ubyte.gz
gunzip -f data/mnist/t10k-labels-idx1-ubyte.gz

# Create a placeholder model.json file if it doesn't exist
if [ ! -f model.json ]; then
    echo "{}" > model.json
fi

echo "✅ MNIST dataset downloaded and extracted successfully!"
echo "You can now run 'cargo run -- train' to train the model."
EOL

# Fix the download_sample_images.sh script to download sample MNIST digit images
cat > download_sample_images.sh << 'EOL'
#!/bin/bash
set -e

echo "Downloading sample MNIST digit images..."

# Create sample_images directory if it doesn't exist
mkdir -p sample_images

# Download sample images
for i in {0..9}; do
    wget -nc -q "https://raw.githubusercontent.com/tracel-ai/burn/main/examples/mnist/sample_images/digit_$i.png" -O "sample_images/digit_$i.png"
done

echo "✅ Sample images downloaded successfully!"
echo "You can now run 'cargo run -- predict --model-path ./model.json --image-path sample_images/digit_0.png' to test prediction."
EOL

echo "Post-generation fixes completed successfully!"
echo "You can now run the following commands:"
echo "  - ./download_mnist.sh to download the MNIST dataset"
echo "  - ./download_sample_images.sh to download sample images for testing"
echo "  - cargo run -- train to train the model"
echo "  - cargo run -- predict -m model.json -i sample_images/digit_0.png to test prediction"
