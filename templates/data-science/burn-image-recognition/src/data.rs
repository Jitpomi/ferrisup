// Data Handling for Image Recognition
// This file handles loading and processing the MNIST dataset

use burn::data::dataset::{Dataset, DatasetLoader, InMemDataset};
use burn::data::dataloader::batcher::Batcher;
use burn::tensor::{backend::Backend, Data, Tensor};
use std::path::Path;

// MNIST Item - represents a single batch of MNIST data
#[derive(Clone, Debug)]
pub struct MnistItem<B: Backend> {
    // Batch of images - shape [batch_size, 1, 28, 28]
    pub images: Tensor<B, 4>,
    // Batch of labels - shape [batch_size]
    pub targets: Tensor<B, 1, usize>,
}

// MNIST Batcher - converts raw data into batches for the model
pub struct MnistBatcher<B: Backend> {
    batch_size: usize,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> MnistBatcher<B> {
    // Create a new batcher with the specified batch size
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

// Raw MNIST item - represents a single image and its label
#[derive(Clone, Debug)]
pub struct RawMnistItem {
    // Image data as a flattened array of 784 pixels (28x28)
    pub image: Vec<f32>,
    // Label (digit 0-9)
    pub label: usize,
}

// Implement the Batcher trait for our MnistBatcher
impl<B: Backend> Batcher<RawMnistItem, MnistItem<B>> for MnistBatcher<B> {
    // Convert a batch of raw items into a processed MnistItem
    fn batch(&self, items: Vec<RawMnistItem>) -> MnistItem<B> {
        // Number of items in this batch
        let batch_size = items.len();
        
        // Create tensors to hold the batch data
        let mut images_data = Data::new(
            vec![0.0; batch_size * 1 * 28 * 28],
            [batch_size, 1, 28, 28],
        );
        let mut targets_data = Data::new(vec![0; batch_size], [batch_size]);
        
        // Process each item in the batch
        for (i, item) in items.iter().enumerate() {
            // Set the target (label)
            targets_data.value_mut()[i] = item.label;
            
            // Copy the image data
            for j in 0..784 {
                // Convert from [0, 255] to [0, 1] range and store in the tensor
                images_data.value_mut()[i * 784 + j] = item.image[j] / 255.0;
            }
        }
        
        // Create tensors from the data
        let images = Tensor::<B, 4>::from_data(images_data);
        let targets = Tensor::<B, 1, usize>::from_data(targets_data);
        
        // Return the processed batch
        MnistItem { images, targets }
    }
}

// Load the MNIST dataset
// In a real application, this would download the actual MNIST dataset
// For this example, we generate synthetic data
pub fn load_mnist_dataset() -> impl Dataset<RawMnistItem> {
    // Create a vector to hold our synthetic data
    let mut items = Vec::new();
    
    // Generate 1000 synthetic training examples
    for i in 0..1000 {
        // Create a simple pattern: even indices are labeled 0, odd are labeled 1
        let label = i % 10;
        
        // Create a synthetic image (in a real app, this would be actual MNIST data)
        // We'll create a simple pattern based on the label
        let mut image = vec![0.0; 784];
        
        // Fill the image with a pattern based on the label
        // This is just a placeholder - real MNIST images would be loaded here
        for j in 0..784 {
            // Create a simple pattern based on the label and position
            image[j] = ((j as f32 + label as f32) % 255.0) / 255.0;
        }
        
        // Add this example to our dataset
        items.push(RawMnistItem { image, label });
    }
    
    // Create an in-memory dataset from our items
    InMemDataset::new(items)
}

// Load the MNIST test dataset
// Similar to the training dataset, but used for evaluation
pub fn load_mnist_test_dataset() -> impl Dataset<RawMnistItem> {
    // Create a vector to hold our synthetic test data
    let mut items = Vec::new();
    
    // Generate 200 synthetic test examples
    for i in 0..200 {
        // Create a simple pattern: even indices are labeled 0, odd are labeled 1
        let label = i % 10;
        
        // Create a synthetic image
        let mut image = vec![0.0; 784];
        
        // Fill the image with a pattern based on the label
        for j in 0..784 {
            // Create a simple pattern based on the label and position
            image[j] = ((j as f32 + label as f32) % 255.0) / 255.0;
        }
        
        // Add this example to our dataset
        items.push(RawMnistItem { image, label });
    }
    
    // Create an in-memory dataset from our items
    InMemDataset::new(items)
}

// Note: In a real application, you would download the actual MNIST dataset
// The code below shows how you might load real MNIST data:
/*
pub fn load_real_mnist_dataset() -> impl Dataset<RawMnistItem> {
    // Download the dataset if it doesn't exist
    let mnist_dir = Path::new("data/mnist");
    if !mnist_dir.exists() {
        std::fs::create_dir_all(mnist_dir).unwrap();
        // Download code would go here
    }
    
    // Load the training images and labels
    let train_images = std::fs::read(mnist_dir.join("train-images-idx3-ubyte")).unwrap();
    let train_labels = std::fs::read(mnist_dir.join("train-labels-idx1-ubyte")).unwrap();
    
    // Parse the MNIST format (skip headers)
    let images = &train_images[16..];
    let labels = &train_labels[8..];
    
    // Create dataset items
    let mut items = Vec::new();
    for i in 0..60000 {  // MNIST has 60,000 training examples
        let offset = i * 784;
        let mut image = Vec::with_capacity(784);
        
        for j in 0..784 {
            image.push(images[offset + j] as f32);
        }
        
        let label = labels[i] as usize;
        items.push(RawMnistItem { image, label });
    }
    
    InMemDataset::new(items)
}
*/
