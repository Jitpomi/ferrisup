// Data Handling for Image Classification
// This file handles loading and processing images for our custom classifier

use burn::data::dataset::{Dataset, InMemDataset};
use burn::data::dataloader::batcher::Batcher;
use burn::tensor::{backend::Backend, Data, Tensor};
use std::path::{Path, PathBuf};
use std::fs;
use image::{GenericImageView, DynamicImage};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

// Image Item - represents a single batch of image data
#[derive(Clone, Debug)]
pub struct ImageItem<B: Backend> {
    // Batch of images - shape [batch_size, 3, height, width]
    pub images: Tensor<B, 4>,
    // Batch of labels - shape [batch_size]
    pub targets: Tensor<B, 1, usize>,
}

// Image Batcher - converts raw data into batches for the model
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

// Raw Image item - represents a single image and its label
#[derive(Clone, Debug)]
pub struct RawImageItem {
    // Image data as RGB values
    pub image: Vec<f32>,
    // Image dimensions (channels, height, width)
    pub dims: [usize; 3],
    // Label (class index)
    pub label: usize,
}

// Implement the Batcher trait for our ImageBatcher
impl<B: Backend> Batcher<RawImageItem, ImageItem<B>> for ImageBatcher<B> {
    // Convert a batch of raw items into a processed ImageItem
    fn batch(&self, items: Vec<RawImageItem>) -> ImageItem<B> {
        // Number of items in this batch
        let batch_size = items.len();
        
        // All images should have the same dimensions
        let [channels, height, width] = items[0].dims;
        
        // Create tensors to hold the batch data
        let mut images_data = Data::new(
            vec![0.0; batch_size * channels * height * width],
            [batch_size, channels, height, width],
        );
        let mut targets_data = Data::new(vec![0; batch_size], [batch_size]);
        
        // Process each item in the batch
        for (i, item) in items.iter().enumerate() {
            // Set the target (label)
            targets_data.value_mut()[i] = item.label;
            
            // Copy the image data
            let flat_size = channels * height * width;
            for j in 0..flat_size {
                images_data.value_mut()[i * flat_size + j] = item.image[j];
            }
        }
        
        // Create tensors from the data
        let images = Tensor::<B, 4>::from_data(images_data);
        let targets = Tensor::<B, 1, usize>::from_data(targets_data);
        
        // Return the processed batch
        ImageItem { images, targets }
    }
}

// Image Dataset - holds our collection of images and their labels
pub struct ImageDataset {
    // List of image items
    items: Vec<RawImageItem>,
    // Mapping from class index to class name
    class_map: HashMap<usize, String>,
    // Mapping from class name to class index
    class_index_map: HashMap<String, usize>,
}

impl ImageDataset {
    // Create a new dataset with the given items and class maps
    pub fn new(
        items: Vec<RawImageItem>,
        class_map: HashMap<usize, String>,
        class_index_map: HashMap<String, usize>,
    ) -> Self {
        Self {
            items,
            class_map,
            class_index_map,
        }
    }
    
    // Get the number of classes in the dataset
    pub fn num_classes(&self) -> usize {
        self.class_map.len()
    }
    
    // Get the list of class names
    pub fn class_names(&self) -> Vec<String> {
        let mut names = vec!["".to_string(); self.class_map.len()];
        for (idx, name) in &self.class_map {
            names[*idx] = name.clone();
        }
        names
    }
    
    // Get the class index for a given class name
    pub fn class_index(&self, class_name: &str) -> Option<usize> {
        self.class_index_map.get(class_name).copied()
    }
}

// Implement the Dataset trait for our ImageDataset
impl Dataset<RawImageItem> for ImageDataset {
    // Get the number of items in the dataset
    fn len(&self) -> usize {
        self.items.len()
    }
    
    // Get a specific item by index
    fn get(&self, index: usize) -> Option<RawImageItem> {
        self.items.get(index).cloned()
    }
}

// Load an image dataset from a directory
// The directory should have subdirectories for each class
pub fn load_image_dataset(data_dir: &str, image_size: usize) -> Result<ImageDataset> {
    let data_path = Path::new(data_dir);
    
    // Check if the directory exists
    if !data_path.exists() || !data_path.is_dir() {
        return Err(anyhow!("Data directory does not exist: {}", data_dir));
    }
    
    // Find all subdirectories (one for each class)
    let mut class_dirs = Vec::new();
    for entry in fs::read_dir(data_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            class_dirs.push(path);
        }
    }
    
    // Check if we found any class directories
    if class_dirs.is_empty() {
        return Err(anyhow!("No class directories found in {}", data_dir));
    }
    
    // Sort class directories for consistent class indices
    class_dirs.sort();
    
    // Create class mappings
    let mut class_map = HashMap::new();
    let mut class_index_map = HashMap::new();
    for (idx, dir) in class_dirs.iter().enumerate() {
        let class_name = dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        class_map.insert(idx, class_name.clone());
        class_index_map.insert(class_name, idx);
    }
    
    println!("Found {} classes: {:?}", class_map.len(), class_map);
    
    // Load all images
    let mut items = Vec::new();
    
    for (class_idx, class_dir) in class_dirs.iter().enumerate() {
        let class_name = class_map.get(&class_idx).unwrap();
        println!("Loading images for class '{}' from {:?}", class_name, class_dir);
        
        // Find all image files in this class directory
        let mut image_paths = Vec::new();
        for entry in fs::read_dir(class_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Check if it's a file with a common image extension
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ["jpg", "jpeg", "png", "bmp"].contains(&ext_str.as_str()) {
                        image_paths.push(path);
                    }
                }
            }
        }
        
        println!("  Found {} images", image_paths.len());
        
        // Process each image
        for path in image_paths {
            // Load and preprocess the image
            match preprocess_image(&path, image_size) {
                Ok((image_data, dims)) => {
                    // Add the image to our dataset
                    items.push(RawImageItem {
                        image: image_data,
                        dims,
                        label: class_idx,
                    });
                }
                Err(e) => {
                    println!("  Warning: Failed to load image {:?}: {}", path, e);
                }
            }
        }
    }
    
    println!("Loaded {} images total", items.len());
    
    // Create and return the dataset
    Ok(ImageDataset::new(items, class_map, class_index_map))
}

// Preprocess an image for the neural network
fn preprocess_image(path: &Path, image_size: usize) -> Result<(Vec<f32>, [usize; 3])> {
    // Load the image
    let img = image::open(path)?;
    
    // Resize the image to the target size
    let img = img.resize_exact(
        image_size as u32,
        image_size as u32,
        image::imageops::FilterType::Triangle,
    );
    
    // Convert to RGB format
    let img = img.to_rgb8();
    
    // Get image dimensions
    let (width, height) = img.dimensions();
    let width = width as usize;
    let height = height as usize;
    
    // Create a vector to hold the image data
    let mut image_data = Vec::with_capacity(3 * height * width);
    
    // Convert to CHW format (channels, height, width) and normalize to [0, 1]
    for c in 0..3 {
        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x as u32, y as u32);
                let value = pixel[c] as f32 / 255.0;
                image_data.push(value);
            }
        }
    }
    
    // Return the preprocessed image data and dimensions
    Ok((image_data, [3, height, width]))
}

// Load and preprocess a single image for prediction
pub fn load_single_image(path: &str, image_size: usize) -> Result<Tensor<burn::backend::ndarray::NdArray, 4>> {
    // Preprocess the image
    let (image_data, dims) = preprocess_image(Path::new(path), image_size)?;
    
    // Create a tensor from the image data
    let data = Data::new(
        image_data,
        [1, dims[0], dims[1], dims[2]], // Add batch dimension
    );
    
    // Create and return the tensor
    Ok(Tensor::<burn::backend::ndarray::NdArray, 4>::from_data(data))
}
