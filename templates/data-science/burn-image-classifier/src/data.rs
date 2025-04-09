// Data handling for the image classifier
// This file contains functions for loading and processing image data

use burn::data::dataset::Dataset;
use burn::tensor::{backend::Backend, Tensor, TensorData};
use burn::data::dataloader::batcher::Batcher;
use image::{DynamicImage, ImageBuffer, Rgb};
use rand::seq::SliceRandom;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{IMAGE_SIZE, NUM_CHANNELS, NUM_CLASSES};
use crate::error::{ImageClassifierError, Result};

/// A dataset of images and their class labels
#[derive(Clone)]
pub struct ImageDataset {
    images: Vec<DynamicImage>,
    labels: Vec<usize>,
    class_names: Vec<String>,
}

impl ImageDataset {
    /// Create a new image dataset
    pub fn new(images: Vec<DynamicImage>, labels: Vec<usize>, class_names: Vec<String>) -> Self {
        Self {
            images,
            labels,
            class_names,
        }
    }
    
    /// Get the number of images in the dataset
    pub fn len(&self) -> usize {
        self.images.len()
    }
    
    /// Get the number of classes in the dataset
    pub fn num_classes(&self) -> usize {
        self.class_names.len()
    }
    
    /// Get the class names
    #[allow(dead_code)]
    pub fn class_names(&self) -> &[String] {
        &self.class_names
    }
    
    /// Split the dataset into training and validation sets
    pub fn split(&self, train_ratio: f64) -> (Self, Self) {
        let n = self.len();
        let train_size = (n as f64 * train_ratio) as usize;
        
        // Create indices and shuffle them
        let mut indices: Vec<usize> = (0..n).collect();
        let mut rng = rand::thread_rng();
        indices.shuffle(&mut rng);
        
        // Split indices
        let train_indices = indices[..train_size].to_vec();
        let valid_indices = indices[train_size..].to_vec();
        
        // Create training dataset
        let train_images = train_indices.iter()
            .map(|&i| self.images[i].clone())
            .collect();
            
        let train_labels = train_indices.iter()
            .map(|&i| self.labels[i])
            .collect();
            
        let train_dataset = Self::new(
            train_images,
            train_labels,
            self.class_names.clone(),
        );
        
        // Create validation dataset
        let valid_images = valid_indices.iter()
            .map(|&i| self.images[i].clone())
            .collect();
            
        let valid_labels = valid_indices.iter()
            .map(|&i| self.labels[i])
            .collect();
            
        let valid_dataset = Self::new(
            valid_images,
            valid_labels,
            self.class_names.clone(),
        );
        
        (train_dataset, valid_dataset)
    }
}

/// Implementation of the Dataset trait for ImageDataset
impl Dataset<(DynamicImage, usize)> for ImageDataset {
    fn get(&self, index: usize) -> Option<(DynamicImage, usize)> {
        if index < self.len() {
            Some((self.images[index].clone(), self.labels[index]))
        } else {
            None
        }
    }
    
    fn len(&self) -> usize {
        self.images.len()
    }
}

/// A batcher for converting images to tensors
#[derive(Clone)]
pub struct ImageBatcher<B: Backend> {
    device: B::Device,
}

impl<B: Backend> ImageBatcher<B> {
    /// Create a new image batcher
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

/// Implementation of the Batcher trait for ImageBatcher
impl<B: Backend> Batcher<(DynamicImage, usize), (Tensor<B, 4>, Tensor<B, 2>)> for ImageBatcher<B> {
    fn batch(&self, items: Vec<(DynamicImage, usize)>) -> (Tensor<B, 4>, Tensor<B, 2>) {
        let batch_size = items.len();
        // Use the constant NUM_CLASSES instead of dynamically determining it
        let num_classes = NUM_CLASSES;
            
        // Convert images to tensors
        let mut image_tensors = Vec::with_capacity(batch_size);
        let mut label_tensors = Vec::with_capacity(batch_size);
        
        for (image, label) in items {
            // Process image
            let image_tensor = match image_to_tensor(&image) {
                Ok(tensor) => tensor,
                Err(_) => {
                    // If there's an error, create a zero tensor
                    vec![0.0; IMAGE_SIZE * IMAGE_SIZE * NUM_CHANNELS]
                }
            };
            
            image_tensors.push(image_tensor);
            
            // Create one-hot encoded label
            let mut label_tensor = vec![0.0; num_classes];
            if label < num_classes {
                label_tensor[label] = 1.0;
            }
            
            label_tensors.push(label_tensor);
        }
        
        // Create image tensor
        let image_data = TensorData::new(
            image_tensors.into_iter().flatten().collect(),
            [batch_size, NUM_CHANNELS, IMAGE_SIZE, IMAGE_SIZE],
        );
        
        let images = Tensor::<B, 4>::from_data(image_data, &self.device);
        
        // Create label tensor
        let label_data = TensorData::new(
            label_tensors.into_iter().flatten().collect(),
            [batch_size, num_classes],
        );
        
        let labels = Tensor::<B, 2>::from_data(label_data, &self.device);
        
        (images, labels)
    }
}

/// Load an image dataset from a directory
pub fn load_image_dataset(data_dir: &str, image_size: usize) -> Result<ImageDataset> {
    let data_path = Path::new(data_dir);
    
    if !data_path.exists() {
        return Err(ImageClassifierError::DirectoryReadError {
            path: data_path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Directory not found"),
        });
    }
    
    // Get class directories
    let class_dirs: Vec<PathBuf> = fs::read_dir(data_path)
        .map_err(|e| ImageClassifierError::DirectoryReadError {
            path: data_path.to_path_buf(),
            source: e,
        })?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect();
        
    if class_dirs.is_empty() {
        // If no class directories found, try to load all images in the data directory
        let mut images = Vec::new();
        let mut labels = Vec::new();
        
        for entry in fs::read_dir(data_path).map_err(|e| ImageClassifierError::DirectoryReadError {
            path: data_path.to_path_buf(),
            source: e,
        })? {
            let entry = entry.map_err(|e| ImageClassifierError::DirectoryReadError {
                path: data_path.to_path_buf(),
                source: e,
            })?;
            
            let path = entry.path();
            if is_image_file(&path) {
                match image::open(&path) {
                    Ok(img) => {
                        // Resize the image
                        let img = img.resize_exact(
                            image_size as u32,
                            image_size as u32,
                            image::imageops::FilterType::Triangle,
                        );
                        
                        images.push(img);
                        labels.push(0); // All images in the same class
                    },
                    Err(_) => continue, // Skip invalid images
                }
            }
        }
        
        if images.is_empty() {
            return Err(ImageClassifierError::EmptyDirectoryError(data_path.to_path_buf()));
        }
        
        return Ok(ImageDataset::new(
            images,
            labels,
            vec!["default".to_string()],
        ));
    }
    
    // Load images from each class directory
    let mut images = Vec::new();
    let mut labels = Vec::new();
    let mut class_names = Vec::new();
    
    for (class_idx, class_dir) in class_dirs.iter().enumerate() {
        let class_name = class_dir.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        class_names.push(class_name);
        
        for entry in fs::read_dir(class_dir).map_err(|e| ImageClassifierError::DirectoryReadError {
            path: class_dir.clone(),
            source: e,
        })? {
            let entry = entry.map_err(|e| ImageClassifierError::DirectoryReadError {
                path: class_dir.clone(),
                source: e,
            })?;
            
            let path = entry.path();
            if is_image_file(&path) {
                match image::open(&path) {
                    Ok(img) => {
                        // Resize the image
                        let img = img.resize_exact(
                            image_size as u32,
                            image_size as u32,
                            image::imageops::FilterType::Triangle,
                        );
                        
                        images.push(img);
                        labels.push(class_idx);
                    },
                    Err(_) => continue, // Skip invalid images
                }
            }
        }
    }
    
    if images.is_empty() {
        return Err(ImageClassifierError::EmptyDirectoryError(data_path.to_path_buf()));
    }
    
    Ok(ImageDataset::new(images, labels, class_names))
}

/// Generate a synthetic dataset for testing
#[allow(dead_code)]
pub fn generate_synthetic_dataset(num_classes: usize, images_per_class: usize) -> ImageDataset {
    let mut images = Vec::new();
    let mut labels = Vec::new();
    let mut class_names = Vec::new();
    
    for class_idx in 0..num_classes {
        class_names.push(format!("class_{}", class_idx));
        
        for _ in 0..images_per_class {
            // Generate a colored image with a hue based on the class
            let hue = (class_idx as f32) / (num_classes as f32);
            let img = create_colored_image(IMAGE_SIZE, hue);
            
            images.push(img);
            labels.push(class_idx);
        }
    }
    
    ImageDataset::new(images, labels, class_names)
}

/// Convert an image to a tensor
pub fn image_to_tensor(img: &DynamicImage) -> Result<Vec<f32>> {
    // Resize the image if needed
    let img = if img.width() != IMAGE_SIZE as u32 || img.height() != IMAGE_SIZE as u32 {
        img.resize_exact(
            IMAGE_SIZE as u32,
            IMAGE_SIZE as u32,
            image::imageops::FilterType::Triangle,
        )
    } else {
        img.clone()
    };
    
    // Convert to RGB
    let rgb_img = img.to_rgb8();
    
    // Create a tensor with shape [channels, height, width]
    let mut tensor = Vec::with_capacity(NUM_CHANNELS * IMAGE_SIZE * IMAGE_SIZE);
    
    // Normalize pixel values to [0, 1]
    for c in 0..NUM_CHANNELS {
        for y in 0..IMAGE_SIZE {
            for x in 0..IMAGE_SIZE {
                let pixel = rgb_img.get_pixel(x as u32, y as u32);
                let value = pixel[c] as f32 / 255.0;
                tensor.push(value);
            }
        }
    }
    
    Ok(tensor)
}

/// Check if a file is an image
fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext = ext.to_lowercase();
        matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp")
    } else {
        false
    }
}

/// Create a colored image with a specific hue
#[allow(dead_code)]
fn create_colored_image(size: usize, hue: f32) -> DynamicImage {
    let mut img = ImageBuffer::new(size as u32, size as u32);
    
    for y in 0..size {
        for x in 0..size {
            let (r, g, b) = hsv_to_rgb(hue, 0.8, 0.8);
            img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
        }
    }
    
    DynamicImage::ImageRgb8(img)
}

/// Convert HSV to RGB
#[allow(dead_code)]
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h * 6.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    
    (
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}
