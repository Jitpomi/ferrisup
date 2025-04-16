// Example: Image Classification
// This example demonstrates how to use the image classifier

use std::path::Path;
use image::DynamicImage;

// Import from the main crate
use {{ project_name }}::model::ImageClassifierModel;
use {{ project_name }}::config::{IMAGE_SIZE, NUM_CHANNELS, CLASS_NAMES};
use {{ project_name }}::error::Result;

// For this example, we'll use the ndarray backend
type Backend = burn_ndarray::NdArrayBackend<f32>;

fn main() -> Result<()> {
    println!("Image Classification Example");
    
    // Path to the model file
    let model_path = "model.json";
    
    // Check if the model exists
    if !Path::new(model_path).exists() {
        println!("Model file not found. Please train a model first using:");
        println!("cargo run -- train");
        return Ok(());
    }
    
    // Load the model
    println!("Loading model from {}...", model_path);
    let model = ImageClassifierModel::<Backend>::load(model_path)?;
    println!("Model loaded successfully!");
    
    // Create or load a sample image
    // For this example, we'll create a simple synthetic image
    let image = create_sample_image();
    
    // Classify the image
    let class_index = classify_image(&model, image)?;
    
    // Get the class name
    let class_name = if class_index < CLASS_NAMES.len() {
        CLASS_NAMES[class_index]
    } else {
        "Unknown"
    };
    
    println!("Classification result: {} (class {})", class_name, class_index);
    
    Ok(())
}

// Create a sample image for testing
fn create_sample_image() -> DynamicImage {
    // Create a simple image with a pattern
    let mut img_buffer = image::RgbImage::new(IMAGE_SIZE as u32, IMAGE_SIZE as u32);
    
    // Fill with a pattern (horizontal stripes)
    for y in 0..IMAGE_SIZE {
        for x in 0..IMAGE_SIZE {
            let color = if y % 4 == 0 {
                [200, 100, 100] // Red stripes
            } else {
                [100, 100, 200] // Blue background
            };
            
            img_buffer.put_pixel(x as u32, y as u32, image::Rgb(color));
        }
    }
    
    DynamicImage::ImageRgb8(img_buffer)
}

// Classify an image using the model
fn classify_image(model: &ImageClassifierModel<Backend>, img: DynamicImage) -> Result<usize> {
    use burn::tensor::{backend::Backend, Data, Tensor};
    
    // Resize the image if needed
    let img = img.resize_exact(
        IMAGE_SIZE as u32,
        IMAGE_SIZE as u32,
        image::imageops::FilterType::Lanczos3,
    );
    
    // Convert image to tensor data
    let mut tensor_data = Vec::with_capacity(IMAGE_SIZE * IMAGE_SIZE * NUM_CHANNELS);
    let img_rgb = img.to_rgb8();
    
    // Normalize pixel values to [0, 1] and store in CHW format
    for c in 0..NUM_CHANNELS {
        for y in 0..IMAGE_SIZE {
            for x in 0..IMAGE_SIZE {
                let pixel = img_rgb.get_pixel(x as u32, y as u32);
                let value = pixel[c] as f32 / 255.0;
                tensor_data.push(value);
            }
        }
    }
    
    // Create tensor
    let tensor = Tensor::<Backend, 4>::from_data(
        Data::new(
            tensor_data,
            [1, NUM_CHANNELS, IMAGE_SIZE, IMAGE_SIZE]
        ),
        &Default::default(),
    );
    
    // Forward pass
    let output = model.forward(tensor);
    
    // Get the predicted class (argmax)
    let predictions = output.argmax(1);
    let class_index = predictions.get([0]).item() as usize;
    
    Ok(class_index)
}
