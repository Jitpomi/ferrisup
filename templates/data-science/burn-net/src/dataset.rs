use burn::{
    data::{dataloader::batcher::Batcher, dataset::vision::MnistItem},
    prelude::*,
};

use anyhow::Result;
use burn::tensor::{backend::Backend, Tensor};
use ndarray::{Array2, Array1, ArrayView1};
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;
use std::path::Path;
use image::GenericImageView;

#[derive(Clone, Debug)]
pub struct MnistBatcher<B: Backend> {
    device: B::Device,
}

#[derive(Clone, Debug)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 3>,
    pub targets: Tensor<B, 1, Int>,
}

impl<B: Backend> MnistBatcher<B> {
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

impl<B: Backend> Batcher<MnistItem, MnistBatch<B>> for MnistBatcher<B> {
    fn batch(&self, items: Vec<MnistItem>) -> MnistBatch<B> {
        let images = items
            .iter()
            .map(|item| TensorData::from(item.image))
            .map(|data| Tensor::<B, 2>::from_data(data.convert::<B::FloatElem>(), &self.device))
            .map(|tensor| tensor.reshape([1, 28, 28]))
            // normalize: make between [0,1] and make the mean = 0 and std = 1
            // values mean=0.1307,std=0.3081 were copied from Pytorch Mist Example
            // https://github.com/pytorch/examples/blob/54f4572509891883a947411fd7239237dd2a39c3/mnist/main.py#L122
            .map(|tensor| ((tensor / 255) - 0.1307) / 0.3081)
            .collect();

        let targets = items
            .iter()
            .map(|item| {
                Tensor::<B, 1, Int>::from_data(
                    TensorData::from([(item.label as i64).elem::<B::IntElem>()]),
                    &self.device,
                )
            })
            .collect();

        let images = Tensor::cat(images, 0);
        let targets = Tensor::cat(targets, 0);

        MnistBatch { images, targets }
    }
}

/// Load the MNIST dataset for training and validation
pub fn load_mnist() -> Result<(Array2<f32>, Array2<f32>, Array2<f32>, Array2<f32>)> {
    // In a real implementation, we would download and parse the MNIST dataset
    // For simplicity, we'll generate random data here
    println!("Note: This is a placeholder for MNIST data loading.");
    println!("In a real application, you would download and parse the actual MNIST dataset.");
    
    // Generate a simple dataset for training (1000 samples with 784 features)
    let n_train_samples = 1000;
    let n_features = 784; // 28x28 images
    let n_classes = 10;
    
    // Create training data
    let x_train = Array2::random((n_train_samples, n_features), Uniform::new(0.0, 1.0));
    
    // Create one-hot encoded training labels
    let mut y_train = Array2::zeros((n_train_samples, n_classes));
    for i in 0..n_train_samples {
        let label = i % 10;
        y_train[[i, label]] = 1.0;
    }
    
    // Generate a simple dataset for validation (200 samples)
    let n_val_samples = 200;
    
    // Create validation data
    let x_val = Array2::random((n_val_samples, n_features), Uniform::new(0.0, 1.0));
    
    // Create one-hot encoded validation labels
    let mut y_val = Array2::zeros((n_val_samples, n_classes));
    for i in 0..n_val_samples {
        let label = i % 10;
        y_val[[i, label]] = 1.0;
    }
    
    Ok((x_train, y_train, x_val, y_val))
}

/// Generate a simple dataset for testing
pub fn generate_simple_dataset() -> Result<(Array2<f32>, Array2<f32>, Array2<f32>, Array2<f32>)> {
    // Similar to load_mnist but with different data generation logic
    println!("Generating a simple dataset for testing...");
    
    let n_train_samples = 500;
    let n_features = 784; // 28x28 images
    let n_classes = 10;
    
    // Create training data with patterns
    let mut x_train = Array2::zeros((n_train_samples, n_features));
    for i in 0..n_train_samples {
        let digit = i % 10;
        
        // Create a simple pattern for each digit
        for j in 0..n_features {
            let row = j / 28;
            let col = j % 28;
            
            if row == digit * 2 || col == digit * 2 {
                x_train[[i, j]] = 0.8;
            } else {
                x_train[[i, j]] = 0.1;
            }
        }
    }
    
    // Create one-hot encoded training labels
    let mut y_train = Array2::zeros((n_train_samples, n_classes));
    for i in 0..n_train_samples {
        let label = i % 10;
        y_train[[i, label]] = 1.0;
    }
    
    // Generate validation data (100 samples)
    let n_val_samples = 100;
    
    // Create validation data with slightly different patterns
    let mut x_val = Array2::zeros((n_val_samples, n_features));
    for i in 0..n_val_samples {
        let digit = i % 10;
        
        // Create a simple pattern for each digit (slightly different from training)
        for j in 0..n_features {
            let row = j / 28;
            let col = j % 28;
            
            if row == digit * 2 + 1 || col == digit * 2 + 1 {
                x_val[[i, j]] = 0.7;
            } else {
                x_val[[i, j]] = 0.2;
            }
        }
    }
    
    // Create one-hot encoded validation labels
    let mut y_val = Array2::zeros((n_val_samples, n_classes));
    for i in 0..n_val_samples {
        let label = i % 10;
        y_val[[i, label]] = 1.0;
    }
    
    Ok((x_train, y_train, x_val, y_val))
}

/// Load the MNIST test dataset
pub fn load_mnist_test() -> Result<(Array2<f32>, Array2<f32>)> {
    // In a real implementation, we would download and parse the MNIST test dataset
    // For simplicity, we'll generate random data here
    println!("Note: This is a placeholder for MNIST test data loading.");
    
    // Generate a simple dataset for testing (200 samples with 784 features)
    let n_samples = 200;
    let n_features = 784; // 28x28 images
    let n_classes = 10;
    
    // Create test data
    let x_test = Array2::random((n_samples, n_features), Uniform::new(0.0, 1.0));
    
    // Create one-hot encoded test labels
    let mut y_test = Array2::zeros((n_samples, n_classes));
    for i in 0..n_samples {
        let label = i % 10;
        y_test[[i, label]] = 1.0;
    }
    
    Ok((x_test, y_test))
}

/// Load and preprocess an image for prediction
pub fn load_image<B: Backend, P: AsRef<Path>>(path: P, device: &B) -> Result<Tensor<B, 2>> {
    // Load the image
    let img = image::open(path)?;
    
    // Resize to 28x28 (MNIST size) and convert to grayscale
    let img = img.resize_exact(28, 28, image::imageops::FilterType::Lanczos3)
        .grayscale();
    
    // Convert to a flat array of normalized pixel values
    let raw_pixels = img.to_luma8();
    let mut pixels = Vec::with_capacity(28 * 28);
    
    for pixel in raw_pixels.pixels() {
        // Normalize to [0, 1] range
        pixels.push(pixel[0] as f32 / 255.0);
    }
    
    // Create a 1x784 tensor for a single image
    let tensor = Tensor::from_vec(pixels, [1, 784], device);
    
    Ok(tensor)
}
