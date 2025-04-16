use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use image::{ImageBuffer, Rgb};
use reqwest::blocking::Client;
use flate2::read::GzDecoder;
use tar::Archive;
use indicatif::{ProgressBar, ProgressStyle};
use clap::Parser;

const CIFAR10_URL: &str = "https://www.cs.toronto.edu/~kriz/cifar-10-binary.tar.gz";
const CIFAR10_FILE: &str = "cifar-10-binary.tar.gz";
const CLASSES: [&str; 10] = [
    "airplane", "automobile", "bird", "cat", "deer",
    "dog", "frog", "horse", "ship", "truck"
];
const IMAGE_SIZE: usize = 32;
const NUM_CHANNELS: usize = 3;
const MAX_IMAGES_PER_CLASS: usize = 20;

/// Command line arguments for the CIFAR-10 downloader
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output directory for the dataset
    #[arg(short, long, default_value = "sample-data")]
    output_dir: String,
    
    /// Maximum number of images per class
    #[arg(short, long, default_value_t = MAX_IMAGES_PER_CLASS)]
    max_images: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("Setting up CIFAR-10 dataset for burn-image-classifier...");
    println!("Output directory: {}", args.output_dir);
    println!("Max images per class: {}", args.max_images);
    
    // Download CIFAR-10 dataset
    download_cifar10()?;
    
    // Extract CIFAR-10 dataset
    extract_cifar10()?;
    
    // Organize dataset into class folders
    organize_dataset(&args.output_dir, args.max_images)?;
    
    println!("Setup complete! You can find the sample data in the '{}' directory.", args.output_dir);
    Ok(())
}

fn download_cifar10() -> Result<(), Box<dyn std::error::Error>> {
    let cifar10_path = Path::new(CIFAR10_FILE);
    
    if cifar10_path.exists() {
        println!("Found existing CIFAR-10 dataset at {}", CIFAR10_FILE);
        return Ok(());
    }
    
    println!("Downloading CIFAR-10 dataset from {}...", CIFAR10_URL);
    
    let client = Client::new();
    let response = client.get(CIFAR10_URL).send()?;
    let total_size = response.content_length().unwrap_or(0);
    
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    
    let mut file = File::create(cifar10_path)?;
    let  downloaded: u64 = 0;
    let  stream = response.bytes()?;
    
    io::copy(&mut stream.as_ref(), &mut file)?;
    
    progress_bar.finish_with_message("Download complete");
    
    Ok(())
}

fn extract_cifar10() -> Result<(), Box<dyn std::error::Error>> {
    println!("Extracting CIFAR-10 dataset...");
    
    let file = File::open(CIFAR10_FILE)?;
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);
    
    archive.unpack(".")?;
    
    Ok(())
}

fn organize_dataset(output_dir: &str, max_images_per_class: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Organizing dataset into class folders...");
    
    // Create output directory
    let output_path = Path::new(output_dir);
    if output_path.exists() {
        fs::remove_dir_all(output_path)?;
    }
    fs::create_dir_all(output_path)?;
    
    // Create class directories
    for class in CLASSES.iter() {
        let class_dir = output_path.join(class);
        fs::create_dir_all(&class_dir)?;
    }
    
    // Process CIFAR-10 binary files
    let cifar_dir = Path::new("cifar-10-batches-bin");
    let batch_files = [
        "data_batch_1.bin",
        "data_batch_2.bin",
        "data_batch_3.bin",
        "data_batch_4.bin",
        "data_batch_5.bin",
    ];
    
    let mut images_per_class = vec![0; CLASSES.len()];
    
    for batch_file in batch_files.iter() {
        let file_path = cifar_dir.join(batch_file);
        let mut file = File::open(&file_path)?;
        
        // Each record is 1 byte for label and 3072 bytes for image (32x32x3)
        let record_size = 1 + (IMAGE_SIZE * IMAGE_SIZE * NUM_CHANNELS);
        
        let mut buffer = vec![0u8; record_size];
        
        while let Ok(()) = file.read_exact(&mut buffer) {
            let label = buffer[0] as usize;
            
            // Skip if we already have enough images for this class
            if images_per_class[label] >= max_images_per_class {
                continue;
            }
            
            // Extract image data (3072 bytes = 32x32x3)
            let image_data = &buffer[1..];
            
            // Create RGB image
            let mut img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(IMAGE_SIZE as u32, IMAGE_SIZE as u32);
            
            for y in 0..IMAGE_SIZE {
                for x in 0..IMAGE_SIZE {
                    let r = image_data[y * IMAGE_SIZE + x];
                    let g = image_data[IMAGE_SIZE * IMAGE_SIZE + y * IMAGE_SIZE + x];
                    let b = image_data[2 * IMAGE_SIZE * IMAGE_SIZE + y * IMAGE_SIZE + x];
                    
                    img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
                }
            }
            
            // Save image
            let class_name = CLASSES[label];
            let image_path = output_path
                .join(class_name)
                .join(format!("image_{}.png", images_per_class[label]));
                
            img.save(image_path)?;
            
            images_per_class[label] += 1;
        }
    }
    
    // Print summary
    println!("Dataset organized into the following classes:");
    for (i, class) in CLASSES.iter().enumerate() {
        println!("  {}: {} images", class, images_per_class[i]);
    }
    
    Ok(())
}
