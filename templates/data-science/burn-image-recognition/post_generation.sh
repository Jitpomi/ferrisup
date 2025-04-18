#!/bin/bash
set -e

echo "Setting up image-recognition project..."
echo "Generating MNIST digit recognition project..."

# Create basic project structure
mkdir -p src

# Create the necessary files for a basic MNIST project
echo "Creating project files..."

# Create lib.rs
cat > src/lib.rs << 'EOL'
//! MNIST digit recognition using the Burn framework
//!
//! This crate provides a simple implementation of a convolutional neural network
//! for recognizing handwritten digits from the MNIST dataset.

pub mod data;
pub mod model;
pub mod training;
EOL

# Create data.rs
cat > src/data.rs << 'EOL'
use burn::{
    data::{
        dataloader::{DataLoaderBuilder, batcher::Batcher}, 
        dataset::vision::{MnistDataset, MnistItem}
    },
    tensor::{backend::Backend, Data as TensorData, Tensor},
};

/// Create train and test dataloaders for MNIST
pub fn create_dataloaders<B: Backend>(
    device: &B::Device,
    batch_size: usize,
) -> (
    impl Iterator<Item = (Tensor<B, 4>, Tensor<B, 1>)>,
    impl Iterator<Item = (Tensor<B, 4>, Tensor<B, 1>)>,
) {
    // Create the datasets
    let train_dataset = MnistDataset::train();
    let test_dataset = MnistDataset::test();
    
    // Create the batcher
    let batcher = MnistBatcher::default();
    
    // Create the dataloaders
    let train_loader = DataLoaderBuilder::new(batcher.clone())
        .batch_size(batch_size)
        .shuffle(true)
        .build(train_dataset)
        .map(|batch| batch.to_device(device))
        .map(|batch| (batch.images, batch.targets));
    
    let test_loader = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .shuffle(false)
        .build(test_dataset)
        .map(|batch| batch.to_device(device))
        .map(|batch| (batch.images, batch.targets));
    
    (train_loader, test_loader)
}

#[derive(Clone, Debug, Default)]
pub struct MnistBatcher {}

#[derive(Clone, Debug)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 4>,
    pub targets: Tensor<B, 1>,
}

impl<B: Backend> Batcher<B, MnistItem, MnistBatch<B>> for MnistBatcher {
    fn batch(&self, items: Vec<MnistItem>, device: &B::Device) -> MnistBatch<B> {
        let images = items
            .iter()
            .map(|item| TensorData::from(item.image))
            .map(|data| Tensor::<B, 2>::from_data(data.convert::<B::FloatElem>(), device))
            // normalize: make between [0,1] and make the mean = 0 and std = 1
            // values mean=0.1307,std=0.3081 were copied from PyTorch MNIST Example
            .map(|tensor| ((tensor / 255) - 0.1307) / 0.3081)
            .map(|tensor| tensor.reshape([1, 1, 28, 28])) // [1, channels, height, width]
            .collect();

        let targets = items
            .iter()
            .map(|item| {
                Tensor::<B, 1>::from_data(
                    TensorData::from([(item.label as i64).elem::<B::IntElem>()]),
                    device,
                )
            })
            .collect();

        let images = Tensor::cat(images, 0);
        let targets = Tensor::cat(targets, 0);

        MnistBatch { images, targets }
    }
}
EOL

# Create model.rs
cat > src/model.rs << 'EOL'
use burn::{
    module::Module,
    nn::{BatchNorm, PaddingConfig2d},
    tensor::{backend::Backend, Tensor},
};

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    conv1: ConvBlock<B>,
    conv2: ConvBlock<B>,
    conv3: ConvBlock<B>,
    dropout: burn::nn::Dropout,
    fc1: burn::nn::Linear<B>,
    fc2: burn::nn::Linear<B>,
    activation: burn::nn::Gelu,
}

impl<B: Backend> Default for Model<B> {
    fn default() -> Self {
        let device = B::Device::default();
        Self::new(&device)
    }
}

const NUM_CLASSES: usize = 10;

impl<B: Backend> Model<B> {
    pub fn new(device: &B::Device) -> Self {
        let conv1 = ConvBlock::new([1, 8], [3, 3], device); // out: [Batch,8,26,26]
        let conv2 = ConvBlock::new([8, 16], [3, 3], device); // out: [Batch,16,24x24]
        let conv3 = ConvBlock::new([16, 24], [3, 3], device); // out: [Batch,24,22x22]
        let hidden_size = 24 * 22 * 22;
        let fc1 = burn::nn::LinearConfig::new(hidden_size, 32)
            .with_bias(false)
            .init(device);
        let fc2 = burn::nn::LinearConfig::new(32, NUM_CLASSES)
            .with_bias(false)
            .init(device);

        let dropout = burn::nn::DropoutConfig::new(0.5).init();

        Self {
            conv1,
            conv2,
            conv3,
            dropout,
            fc1,
            fc2,
            activation: burn::nn::Gelu::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 2> {
        // Input shape: [batch_size, channels, height, width]
        let x = self.conv1.forward(input);
        let x = self.conv2.forward(x);
        let x = self.conv3.forward(x);

        let [batch_size, channels, height, width] = x.dims();
        let x = x.reshape([batch_size, channels * height * width]);

        let x = self.dropout.forward(x);
        let x = self.fc1.forward(x);
        let x = self.activation.forward(x);

        self.fc2.forward(x)
    }
}

#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    conv: burn::nn::conv::Conv2d<B>,
    norm: BatchNorm<B, 2>,
    activation: burn::nn::Gelu,
}

impl<B: Backend> ConvBlock<B> {
    pub fn new(channels: [usize; 2], kernel_size: [usize; 2], device: &B::Device) -> Self {
        let conv = burn::nn::conv::Conv2dConfig::new(channels, kernel_size)
            .with_padding(PaddingConfig2d::Valid)
            .init(device);
        let norm = burn::nn::BatchNormConfig::new(channels[1]).init(device);

        Self {
            conv,
            norm,
            activation: burn::nn::Gelu::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(input);
        let x = self.norm.forward(x);

        self.activation.forward(x)
    }
}
EOL

# Create training.rs
cat > src/training.rs << 'EOL'
use crate::{data::create_dataloaders, model::Model};
use burn::{
    optim::AdamConfig,
    tensor::{backend::Backend, Tensor},
};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

/// Train the model
pub fn train<B: Backend>(
    device: &B::Device,
    model: Model<B>,
    num_epochs: usize,
    batch_size: usize,
    learning_rate: f64,
    model_path: impl AsRef<Path>,
) -> Model<B> {
    // Create dataloaders
    let (dataloader_train, dataloader_test) = create_dataloaders::<B>(device, batch_size);
    
    // Create optimizer
    let optimizer = AdamConfig::new().with_learning_rate(learning_rate).init();
    
    // Create training state
    let mut train_state = burn::train::TrainState::new(model, optimizer);
    
    // Training loop
    for epoch in 0..num_epochs {
        // Create progress bar
        let pb = ProgressBar::new(100); // Estimate based on dataset size
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        // Training phase
        let mut train_accuracy_sum = 0.0f32;
        let mut train_loss_sum = 0.0f32;
        let mut train_count = 0;
        
        for (images, targets) in dataloader_train {
            // Forward pass
            let output = train_state.model().forward(images.clone());
            
            // Compute loss
            let loss = cross_entropy_loss(&output, &targets);
            
            // Backward pass and optimization
            train_state.backward_step(&loss);
            
            // Compute accuracy
            let predicted = output.argmax(1).squeeze(1);
            let correct = (predicted.equal(&targets)).to_dtype::<f32>().mean().into_scalar();
            
            // Update metrics
            train_accuracy_sum += correct;
            train_loss_sum += loss.into_scalar();
            train_count += 1;
            
            pb.inc(1);
        }
        
        let train_accuracy = train_accuracy_sum / train_count as f32;
        let train_loss = train_loss_sum / train_count as f32;
        
        // Validation phase
        let mut valid_accuracy_sum = 0.0f32;
        let mut valid_loss_sum = 0.0f32;
        let mut valid_count = 0;
        
        for (images, targets) in dataloader_test {
            // Forward pass
            let output = train_state.model().forward(images);
            
            // Compute loss
            let loss = cross_entropy_loss(&output, &targets);
            
            // Compute accuracy
            let predicted = output.argmax(1).squeeze(1);
            let correct = (predicted.equal(&targets)).to_dtype::<f32>().mean().into_scalar();
            
            // Update metrics
            valid_accuracy_sum += correct;
            valid_loss_sum += loss.into_scalar();
            valid_count += 1;
        }
        
        let valid_accuracy = valid_accuracy_sum / valid_count as f32;
        let valid_loss = valid_loss_sum / valid_count as f32;
        
        pb.finish_and_clear();
        
        println!(
            "Epoch {}/{}: Train Loss = {:.4}, Train Accuracy = {:.4}, Valid Loss = {:.4}, Valid Accuracy = {:.4}",
            epoch + 1,
            num_epochs,
            train_loss,
            train_accuracy,
            valid_loss,
            valid_accuracy
        );
    }
    
    // Save the model if a path is provided
    if let Some(parent) = model_path.as_ref().parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }
    
    train_state.model().save(model_path).unwrap();
    
    // Return the trained model
    train_state.into_model()
}

/// Evaluate the model
pub fn evaluate<B: Backend>(
    device: &B::Device,
    model: Model<B>,
    batch_size: usize,
) -> (f32, f32) {
    // Create dataloaders
    let (_, dataloader_test) = create_dataloaders::<B>(device, batch_size);
    
    // Evaluation metrics
    let mut accuracy_sum = 0.0f32;
    let mut loss_sum = 0.0f32;
    let mut count = 0;
    
    // Create progress bar
    let pb = ProgressBar::new(100); // Estimate based on dataset size
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    
    // Evaluation loop
    for (images, targets) in dataloader_test {
        // Forward pass
        let output = model.forward(images);
        
        // Compute loss
        let loss = cross_entropy_loss(&output, &targets);
        
        // Compute accuracy
        let predicted = output.argmax(1).squeeze(1);
        let correct = (predicted.equal(&targets)).to_dtype::<f32>().mean().into_scalar();
        
        // Update metrics
        accuracy_sum += correct;
        loss_sum += loss.into_scalar();
        count += 1;
        
        pb.inc(1);
    }
    
    pb.finish_and_clear();
    
    let accuracy = accuracy_sum / count as f32;
    let loss = loss_sum / count as f32;
    
    println!("Evaluation: Loss = {:.4}, Accuracy = {:.4}", loss, accuracy);
    
    (accuracy, loss)
}

/// Cross-entropy loss function
fn cross_entropy_loss<B: Backend>(output: &Tensor<B, 2>, target: &Tensor<B, 1>) -> Tensor<B, 1> {
    let log_softmax = output.log_softmax(1);
    let nll = log_softmax.gather(1, &target.unsqueeze(1)).negative();
    nll.mean(0)
}
EOL

# Create main.rs
cat > src/main.rs << 'EOL'
use anyhow::Result;
use burn::tensor::{backend::Backend, Data as TensorData, Tensor};
use burn_ndarray::NdArray;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

# Ensure MNIST data is always downloaded before training
if [ ! -f data/mnist/train-images-idx3-ubyte ] || [ ! -f data/mnist/train-labels-idx1-ubyte ]; then
  echo "Downloading MNIST dataset automatically..."
  ./download_mnist.sh
fi

# Import our modules
use {{ project_name }}::{
    model::Model,
    training::{train, evaluate},
};

# Define the backend type
type Backend = NdArray<f32>;

# Command line interface for our application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

# Available commands: train, evaluate, or predict
#[derive(Subcommand)]
enum Commands {
    # Train a new model
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
    
    # Evaluate an existing model
    Evaluate {
        #[arg(short, long)]
        model_path: PathBuf,
        
        #[arg(short, long, default_value = "64")]
        batch_size: usize,
    },
    
    # Predict using an existing model
    Predict {
        #[arg(short, long)]
        model_path: PathBuf,
        
        #[arg(short, long)]
        image_path: PathBuf,
    },
}

# Main function - entry point of our program
fn main() -> Result<()> {
    # Parse command line arguments
    let cli = Cli::parse();
    
    # Create a device for computation
    let device = Default::default();
    
    # Handle the different commands
    match cli.command {
        Commands::Train { epochs, batch_size, learning_rate, model_path } => {
            println!("🚀 Training a new MNIST digit recognition model");
            println!("📊 Epochs: {}", epochs);
            println!("📦 Batch size: {}", batch_size);
            println!("📈 Learning rate: {}", learning_rate);
            
            # Create a new model
            let model = Model::<Backend>::default();
            
            # Train the model
            let trained_model = train(
                &device,
                model,
                epochs,
                batch_size,
                learning_rate,
                model_path,
            );
            
            println!("✅ Training completed successfully!");
        },
        
        Commands::Evaluate { model_path, batch_size } => {
            println!("🔍 Evaluating MNIST digit recognition model");
            
            # Load the model
            let model = Model::<Backend>::load(model_path)?;
            
            # Evaluate the model
            let (accuracy, loss) = evaluate(&device, model, batch_size);
            
            println!("📊 Test accuracy: {:.2}%", accuracy * 100.0);
            println!("📉 Test loss: {:.4}", loss);
        },
        
        Commands::Predict { model_path, image_path } => {
            println!("🔮 Predicting digit from image");
            
            # Load the model
            let model = Model::<Backend>::load(model_path)?;
            
            # Load and preprocess the image
            let image = image::open(image_path)?
                .to_luma8();
            
            # Resize to 28x28 if needed
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
            
            # Convert to tensor
            let image_data: Vec<f32> = image
                .pixels()
                .map(|p| (p[0] as f32 / 255.0 - 0.1307) / 0.3081)
                .collect();
            
            # Create a 4D tensor [batch_size=1, channels=1, height=28, width=28]
            let tensor = Tensor::<Backend, 4>::from_data(
                TensorData::new(image_data, [1, 1, 28, 28]),
                &device,
            );
            
            # Make prediction
            let output = model.forward(tensor);
            
            # Get the predicted digit
            let prediction = output
                .argmax(1)
                .squeeze(1)
                .into_scalar::<i64>();
            
            println!("✅ Predicted digit: {}", prediction);
        },
    }
    
    Ok(())
}
EOL

# Create README.md
cat > README.md << 'EOL'
# MNIST Digit Recognition

A handwritten digit recognition project using the Burn deep learning framework and the MNIST dataset.

## Overview

This project implements a Convolutional Neural Network (CNN) to recognize handwritten digits from the MNIST dataset. It uses the Burn framework, a deep learning framework written in Rust.

## Getting Started

### Prerequisites

- Rust and Cargo (install from [rustup.rs](https://rustup.rs/))
- The MNIST dataset (download script provided)

### Setup

1. Download the MNIST dataset:

```bash
./download_mnist.sh
```

2. Download sample images for testing:

```bash
./download_sample_images.sh
```

## Usage

### Training a Model

To train a new model with default parameters:

```bash
cargo run -- train
```

You can customize the training process with the following options:

```bash
cargo run -- train --epochs 10 --batch-size 128 --learning-rate 0.001 --model-path model.json
```

### Evaluating a Model

To evaluate a trained model on the test dataset:

```bash
cargo run -- evaluate --model-path model.json
```

### Making Predictions

To recognize a digit in a custom image:

```bash
cargo run -- predict --model-path model.json --image-path path/to/your/image.png
```

The image should ideally be a grayscale image with a white digit on a black background, similar to the MNIST format.

## Model Architecture

The CNN architecture consists of:
- Three convolutional layers with batch normalization and GELU activation
- Dropout for regularization
- Two fully connected layers

## Project Structure

- `src/data.rs`: Dataset loading and batching
- `src/model.rs`: Neural network architecture
- `src/training.rs`: Training and evaluation logic
- `src/main.rs`: CLI interface

## License

This project is licensed under the MIT License - see the LICENSE file for details.
EOL

# Create download_sample_images.sh
cat > download_sample_images.sh << 'EOL'
#!/bin/bash

# Create directory for sample images
mkdir -p sample_images

# Download 10 sample MNIST images (one for each digit)
echo "Downloading sample MNIST images..."

# Use a more reliable source for sample MNIST images
# These are hosted on GitHub in a public repository
URLS=(
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/0.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/1.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/2.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/3.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/4.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/5.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/6.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/7.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/8.png"
  "https://github.com/tracel-ai/burn/raw/main/examples/mnist/assets/9.png"
)

# Download each image
for i in {0..9}; do
  echo "Downloading digit $i sample..."
  curl -L -s "${URLS[$i]}" -o "sample_images/digit_$i.png"
done

echo "✅ Sample images downloaded to sample_images/ directory"
echo "You can use these images for testing with the predict command:"
echo "cargo run -- predict --model-path model.json --image-path sample_images/digit_0.png"
EOL

# Create download_mnist.sh
cat > download_mnist.sh << 'EOL'
#!/bin/bash

# Create directory for MNIST dataset
mkdir -p data/mnist

echo "Downloading MNIST dataset..."

# Use a more reliable mirror for the MNIST dataset
MNIST_BASE_URL="https://storage.googleapis.com/cvdf-datasets/mnist"

# Download MNIST dataset files
curl -L -o data/mnist/train-images-idx3-ubyte.gz "${MNIST_BASE_URL}/train-images-idx3-ubyte.gz"
curl -L -o data/mnist/train-labels-idx1-ubyte.gz "${MNIST_BASE_URL}/train-labels-idx1-ubyte.gz"
curl -L -o data/mnist/t10k-images-idx3-ubyte.gz "${MNIST_BASE_URL}/t10k-images-idx3-ubyte.gz"
curl -L -o data/mnist/t10k-labels-idx1-ubyte.gz "${MNIST_BASE_URL}/t10k-labels-idx1-ubyte.gz"

# Extract the files
echo "Extracting MNIST dataset files..."
gunzip -f data/mnist/train-images-idx3-ubyte.gz
gunzip -f data/mnist/train-labels-idx1-ubyte.gz
gunzip -f data/mnist/t10k-images-idx3-ubyte.gz
gunzip -f data/mnist/t10k-labels-idx1-ubyte.gz

echo "✅ MNIST dataset downloaded and extracted to data/mnist/ directory"
echo "You can now train the model with: cargo run -- train"
EOL

# Make the scripts executable
chmod +x download_sample_images.sh
chmod +x download_mnist.sh

# Automatically download MNIST dataset into the generated app's directory
if [ -d "$DESTINATION_DIR" ]; then
  cd "$DESTINATION_DIR"
  if [ ! -f data/mnist/train-images-idx3-ubyte ] || [ ! -f data/mnist/train-labels-idx1-ubyte ]; then
    echo "Downloading MNIST dataset automatically into new app..."
    ./download_mnist.sh
  fi
  cd -
fi

echo "✅ Setup completed successfully!"
