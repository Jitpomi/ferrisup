// Data Handling for Image Classification
// This file handles loading and processing image data

use burn::data::dataset::{Dataset, InMemDataset};
use burn::data::dataloader::batcher::Batcher;
use burn::tensor::{backend::Backend, Data, Tensor};
use std::path::{Path, PathBuf};
use std::fs;
use image::{DynamicImage, GenericImageView, imageops};
use anyhow::{Result, anyhow};
use rand::{Rng, thread_rng};

// Import our configuration parameters
use crate::config::{
    IMAGE_SIZE, NUM_CHANNELS, NUM_CLASSES, CLASS_NAMES,
    USE_AUGMENTATION, RANDOM_FLIP, RANDOM_CROP, RANDOM_BRIGHTNESS
};

// Image Item - represents a single batch of image data
#[derive(Clone, Debug)]
pub struct ImageItem<B: Backend> {
    // Batch of images - shape [batch_size, channels, height, width]
    pub images: Tensor<B, 4>,
    // Batch of labels - shape [batch_size, num_classes]
    pub labels: Tensor<B, 2>,
}

// Image Batcher - converts raw image data into batches for the model
pub struct ImageBatcher<B: Backend> {
    batch_size: usize,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> ImageBatcher<B> {
    // Create a new batcher with the specified batch size
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            _phantom: std::marker::PhantomData,
        }
    }
}

// Raw Image item - represents a single example with image and label
#[derive(Clone, Debug)]
pub struct RawImageItem {
    // Path to the image file
    pub image_path: PathBuf,
    // Class label (integer)
    pub label: usize,
}

// Implement the Batcher trait for our ImageBatcher
impl<B: Backend> Batcher<RawImageItem, ImageItem<B>> for ImageBatcher<B> {
    // Convert a batch of raw items into a processed ImageItem
    fn batch(&self, items: Vec<RawImageItem>) -> ImageItem<B> {
        // Number of items in this batch
        let batch_size = items.len();
        
        // Create tensors to hold the batch data
        let mut images_data = Data::new(
            vec![0.0; batch_size * NUM_CHANNELS * IMAGE_SIZE * IMAGE_SIZE],
            [batch_size, NUM_CHANNELS, IMAGE_SIZE, IMAGE_SIZE],
        );
        
        let mut labels_data = Data::new(
            vec![0.0; batch_size * NUM_CLASSES],
            [batch_size, NUM_CLASSES],
        );
        
        // Process each item in the batch
        for (i, item) in items.iter().enumerate() {
            // Load and process the image
            let img = load_and_process_image(&item.image_path).unwrap_or_else(|_| {
                // If loading fails, create a blank image
                DynamicImage::new_rgb8(IMAGE_SIZE as u32, IMAGE_SIZE as u32)
            });
            
            // Convert image to tensor data
            let image_tensor = image_to_tensor(&img);
            
            // Copy the image data to the batch tensor
            let image_size = NUM_CHANNELS * IMAGE_SIZE * IMAGE_SIZE;
            let image_offset = i * image_size;
            for j in 0..image_size {
                images_data.value_mut()[image_offset + j] = image_tensor[j];
            }
            
            // Set the one-hot encoded label
            let label_offset = i * NUM_CLASSES;
            for j in 0..NUM_CLASSES {
                labels_data.value_mut()[label_offset + j] = if j == item.label { 1.0 } else { 0.0 };
            }
        }
        
        // Create tensors from the data
        let images = Tensor::<B, 4>::from_data(images_data);
        let labels = Tensor::<B, 2>::from_data(labels_data);
        
        // Return the processed batch
        ImageItem { images, labels }
    }
}

// Dataset structure to hold our image data
pub struct ImageDataset {
    // List of examples (images and labels)
    items: Vec<RawImageItem>,
    // Class names
    class_names: Vec<String>,
}

impl ImageDataset {
    // Create a new dataset with the given items and class names
    pub fn new(items: Vec<RawImageItem>, class_names: Vec<String>) -> Self {
        Self { items, class_names }
    }
    
    // Get the number of classes in this dataset
    pub fn num_classes(&self) -> usize {
        self.class_names.len()
    }
    
    // Get the class names
    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }
    
    // Split the dataset into training and validation sets
    pub fn split_by_ratio(&self, ratios: [f32; 2]) -> (InMemDataset<RawImageItem>, InMemDataset<RawImageItem>) {
        let total: f32 = ratios.iter().sum();
        let ratio_a = ratios[0] / total;
        
        let n_a = (self.items.len() as f32 * ratio_a).round() as usize;
        let n_a = n_a.min(self.items.len());
        
        let items_a = self.items[0..n_a].to_vec();
        let items_b = self.items[n_a..].to_vec();
        
        (
            InMemDataset::new(items_a),
            InMemDataset::new(items_b),
        )
    }
}

// Implement the Dataset trait for our ImageDataset
impl Dataset<RawImageItem> for ImageDataset {
    // Get the number of examples in the dataset
    fn len(&self) -> usize {
        self.items.len()
    }
    
    // Get a specific example by index
    fn get(&self, index: usize) -> Option<RawImageItem> {
        self.items.get(index).cloned()
    }
}

// Load an image dataset from a directory
// The directory should have subdirectories for each class
pub fn load_image_dataset(data_dir: &str, image_size: usize) -> Result<ImageDataset> {
    // CUSTOMIZE HERE: Modify how images are loaded and organized
    
    let data_path = Path::new(data_dir);
    if !data_path.exists() {
        return Err(anyhow!("Data directory does not exist: {}", data_dir));
    }
    
    let mut items = Vec::new();
    let mut class_names = Vec::new();
    
    // Read class directories (each subdirectory is a class)
    let class_dirs = fs::read_dir(data_path)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
        })
        .collect::<Vec<_>>();
    
    if class_dirs.is_empty() {
        return Err(anyhow!("No class directories found in {}", data_dir));
    }
    
    // Sort class directories to ensure consistent class indices
    let mut class_dirs = class_dirs;
    class_dirs.sort_by_key(|dir| dir.file_name());
    
    // Process each class directory
    for (class_idx, class_dir) in class_dirs.iter().enumerate() {
        let class_name = class_dir.file_name().to_string_lossy().to_string();
        class_names.push(class_name.clone());
        
        // Read all image files in this class directory
        let image_files = fs::read_dir(class_dir.path())?
            .filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                let extension = path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");
                
                // Only include common image formats
                matches!(extension.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "bmp")
            })
            .collect::<Vec<_>>();
        
        // Add each image to our dataset
        for image_file in image_files {
            items.push(RawImageItem {
                image_path: image_file.path(),
                label: class_idx,
            });
        }
    }
    
    // If no class names were found in the directory, use the default ones
    if class_names.is_empty() {
        class_names = CLASS_NAMES.iter().map(|&s| s.to_string()).collect();
    }
    
    // If no items were loaded, return an error
    if items.is_empty() {
        return Err(anyhow!("No images found in {}", data_dir));
    }
    
    // Create and return the dataset
    Ok(ImageDataset::new(items, class_names))
}

// Load and process an image file
fn load_and_process_image(path: &Path) -> Result<DynamicImage> {
    // CUSTOMIZE HERE: Modify image preprocessing
    
    // Load the image
    let img = image::open(path)?;
    
    // Apply data augmentation if enabled
    let img = if USE_AUGMENTATION {
        apply_augmentation(img)
    } else {
        img
    };
    
    // Resize the image to the required dimensions
    let img = img.resize_exact(
        IMAGE_SIZE as u32,
        IMAGE_SIZE as u32,
        image::imageops::FilterType::Triangle
    );
    
    Ok(img)
}

// Apply data augmentation to an image
fn apply_augmentation(img: DynamicImage) -> DynamicImage {
    // CUSTOMIZE HERE: Add or modify augmentation techniques
    
    let mut rng = thread_rng();
    let mut result = img;
    
    // Random horizontal flip
    if RANDOM_FLIP && rng.gen_bool(0.5) {
        result = result.fliph();
    }
    
    // Random crop and resize
    if RANDOM_CROP && rng.gen_bool(0.5) {
        let (width, height) = result.dimensions();
        let crop_percent = rng.gen_range(0.8..1.0);
        
        let crop_width = (width as f32 * crop_percent) as u32;
        let crop_height = (height as f32 * crop_percent) as u32;
        
        let x = rng.gen_range(0..width - crop_width);
        let y = rng.gen_range(0..height - crop_height);
        
        result = result.crop_imm(x, y, crop_width, crop_height);
        result = result.resize_exact(
            width,
            height,
            image::imageops::FilterType::Triangle
        );
    }
    
    // Random brightness adjustment
    if RANDOM_BRIGHTNESS && rng.gen_bool(0.5) {
        let brightness_factor = rng.gen_range(0.8..1.2);
        result = DynamicImage::ImageRgba8(imageops::brighten(
            &result.to_rgba8(),
            (brightness_factor * 255.0 - 255.0) as i32
        ));
    }
    
    result
}

// Convert an image to a normalized tensor
fn image_to_tensor(img: &DynamicImage) -> Vec<f32> {
    // CUSTOMIZE HERE: Modify normalization or preprocessing
    
    // Convert to RGB if it's not already
    let img_rgb = img.to_rgb8();
    
    // Create a vector to hold the tensor data
    let mut tensor = vec![0.0; NUM_CHANNELS * IMAGE_SIZE * IMAGE_SIZE];
    
    // Fill the tensor with normalized pixel values
    for y in 0..IMAGE_SIZE {
        for x in 0..IMAGE_SIZE {
            let pixel = img_rgb.get_pixel(x as u32, y as u32);
            
            // Normalize pixel values to the range [0, 1]
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            
            // Store in CHW format (channels, height, width)
            let idx_r = 0 * IMAGE_SIZE * IMAGE_SIZE + y * IMAGE_SIZE + x;
            let idx_g = 1 * IMAGE_SIZE * IMAGE_SIZE + y * IMAGE_SIZE + x;
            let idx_b = 2 * IMAGE_SIZE * IMAGE_SIZE + y * IMAGE_SIZE + x;
            
            tensor[idx_r] = r;
            tensor[idx_g] = g;
            tensor[idx_b] = b;
        }
    }
    
    tensor
}

// Create a sample dataset with synthetic images for testing
pub fn create_sample_dataset(num_samples: usize) -> ImageDataset {
    // CUSTOMIZE HERE: Modify the synthetic data generation
    
    let mut rng = thread_rng();
    let mut items = Vec::with_capacity(num_samples);
    
    // Generate synthetic data
    for _ in 0..num_samples {
        // Generate a random label
        let label = rng.gen_range(0..NUM_CLASSES);
        
        // Create a dummy image path (won't be used for synthetic data)
        let image_path = PathBuf::from(format!("synthetic_image_{}.jpg", label));
        
        // Add this example to our dataset
        items.push(RawImageItem { image_path, label });
    }
    
    // Create class names
    let class_names = CLASS_NAMES.iter().map(|&s| s.to_string()).collect();
    
    // Create and return the dataset
    ImageDataset::new(items, class_names)
}
