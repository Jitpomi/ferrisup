use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use image::{ImageBuffer, Rgb};
use reqwest::blocking::Client;
use flate2::read::GzDecoder;
use tar::Archive;
use indicatif::{ProgressBar, ProgressStyle};

const CIFAR10_URL: &str = "https://www.cs.toronto.edu/~kriz/cifar-10-binary.tar.gz";
const CIFAR10_FILE: &str = "cifar-10-binary.tar.gz";
const CLASSES: [&str; 10] = [
    "airplane", "automobile", "bird", "cat", "deer",
    "dog", "frog", "horse", "ship", "truck"
];
const IMAGE_SIZE: usize = 32;
#[allow(dead_code)]
const NUM_CHANNELS: usize = 3;
const MAX_IMAGES_PER_CLASS: usize = 20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Setting up CIFAR-10 dataset for burn-image-classifier...");
    
    // Download CIFAR-10 dataset
    download_cifar10()?;
    
    // Extract CIFAR-10 dataset
    extract_cifar10()?;
    
    // Organize dataset into class folders
    organize_dataset()?;
    
    println!("Setup complete! You can find the sample data in the 'sample-data' directory.");
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
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    
    let mut file = File::create(cifar10_path)?;
    let mut content = Cursor::new(response.bytes()?);
    
    let mut buffer = [0; 8192];
    let mut downloaded = 0;
    
    while let Ok(n) = content.read(&mut buffer) {
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        downloaded += n as u64;
        progress_bar.set_position(downloaded);
    }
    
    progress_bar.finish_with_message("Download complete!");
    Ok(())
}

fn extract_cifar10() -> Result<(), Box<dyn std::error::Error>> {
    let extract_dir = Path::new("cifar-10-batches-bin");
    
    if extract_dir.exists() {
        println!("Found extracted CIFAR-10 data");
        return Ok(());
    }
    
    println!("Extracting CIFAR-10 dataset...");
    
    let file = File::open(CIFAR10_FILE)?;
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);
    archive.unpack(".")?;
    
    println!("Extraction complete!");
    Ok(())
}

fn organize_dataset() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample-data directory
    let base_dir = Path::new("sample-data");
    if !base_dir.exists() {
        fs::create_dir_all(base_dir)?;
    }
    
    // Create class directories
    for class_name in CLASSES.iter() {
        let class_dir = base_dir.join(class_name);
        if !class_dir.exists() {
            fs::create_dir_all(&class_dir)?;
        }
    }
    
    // Process training batch
    println!("Organizing CIFAR-10 images into class folders...");
    let batch_file = Path::new("cifar-10-batches-bin").join("data_batch_1.bin");
    
    let mut file = File::open(batch_file)?;
    let mut class_counts = [0; 10];
    let progress_bar = ProgressBar::new(10000); // CIFAR-10 has 10000 images per batch
    
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    
    for _ in 0..10000 {
        // Each record is 1 byte for label + 3072 bytes for image (32x32x3)
        let mut label_buf = [0u8; 1];
        file.read_exact(&mut label_buf)?;
        let label = label_buf[0];
        
        let mut buffer = vec![0u8; 3072];
        file.read_exact(&mut buffer)?;
        
        // Skip if we already have enough images for this class
        if class_counts[label as usize] >= MAX_IMAGES_PER_CLASS {
            progress_bar.inc(1);
            continue;
        }
        
        // Convert to RGB image
        let mut img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(IMAGE_SIZE as u32, IMAGE_SIZE as u32);
        
        for y in 0..IMAGE_SIZE {
            for x in 0..IMAGE_SIZE {
                let r = buffer[y * IMAGE_SIZE + x];
                let g = buffer[IMAGE_SIZE * IMAGE_SIZE + y * IMAGE_SIZE + x];
                let b = buffer[2 * IMAGE_SIZE * IMAGE_SIZE + y * IMAGE_SIZE + x];
                img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
            }
        }
        
        // Save to appropriate class folder
        let class_name = CLASSES[label as usize];
        let filename = format!("{}_{:03}.png", class_name, class_counts[label as usize]);
        let output_path = base_dir.join(class_name).join(filename);
        img.save(output_path)?;
        
        // Increment count for this class
        class_counts[label as usize] += 1;
        progress_bar.inc(1);
        
        // Break if we have enough images for all classes
        if class_counts.iter().all(|&count| count >= MAX_IMAGES_PER_CLASS) {
            break;
        }
    }
    
    progress_bar.finish();
    
    println!("Dataset organization complete!");
    println!("Created {} images across {} classes", class_counts.iter().sum::<usize>(), CLASSES.len());
    
    Ok(())
}
