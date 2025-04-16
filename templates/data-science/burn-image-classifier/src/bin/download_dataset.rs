use std::fs::{self, File};
use std::process::Command;
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dataset to download
    #[arg(short, long, value_enum, default_value_t = DatasetType::Cifar10)]
    dataset: DatasetType,

    /// Output directory
    #[arg(short, long, default_value = "sample-data")]
    output_dir: String,

    /// Number of classes (for synthetic dataset)
    #[arg(long, default_value_t = 10)]
    num_classes: usize,

    /// Number of images per class (for synthetic dataset)
    #[arg(long, default_value_t = 100)]
    images_per_class: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
enum DatasetType {
    /// CIFAR-10 dataset (10 classes, 60000 images)
    Cifar10,
    /// MNIST dataset (10 classes, handwritten digits)
    Mnist,
    /// Fashion-MNIST dataset (10 classes, fashion items)
    FashionMnist,
    /// Generate synthetic dataset
    Synthetic,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Create output directory if it doesn't exist
    fs::create_dir_all(&args.output_dir)?;

    match args.dataset {
        DatasetType::Cifar10 => {
            println!("Downloading CIFAR-10 dataset...");
            download_cifar10(&args.output_dir)?;
        }
        DatasetType::Mnist => {
            println!("Downloading MNIST dataset...");
            download_mnist(&args.output_dir)?;
        }
        DatasetType::FashionMnist => {
            println!("Downloading Fashion-MNIST dataset...");
            download_fashion_mnist(&args.output_dir)?;
        }
        DatasetType::Synthetic => {
            println!("Generating synthetic dataset...");
            println!("Output directory: {}", args.output_dir);
            println!("Number of classes: {}", args.num_classes);
            println!("Images per class: {}", args.images_per_class);
            
            // Call the generate_synthetic binary
            let status = Command::new("cargo")
                .args([
                    "run", 
                    "--bin", "generate_synthetic", 
                    "--", 
                    "--num-classes", &args.num_classes.to_string(),
                    "--images-per-class", &args.images_per_class.to_string(),
                    "--output-dir", &args.output_dir
                ])
                .status()?;
            
            if !status.success() {
                return Err("Failed to generate synthetic dataset".into());
            }
        }
    }

    println!("Dataset preparation complete!");
    println!("You can find the dataset in the '{}' directory", args.output_dir);
    Ok(())
}

fn download_cifar10(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Call the existing download_cifar10 binary
    let status = Command::new("cargo")
        .args([
            "run", 
            "--bin", "download_cifar10", 
            "--", 
            "--output-dir", output_dir
        ])
        .status()?;
    
    if !status.success() {
        return Err("Failed to download CIFAR-10 dataset".into());
    }
    
    Ok(())
}

fn download_mnist(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // URLs for MNIST dataset
    let train_images_url = "http://yann.lecun.com/exdb/mnist/train-images-idx3-ubyte.gz";
    let train_labels_url = "http://yann.lecun.com/exdb/mnist/train-labels-idx1-ubyte.gz";
    let test_images_url = "http://yann.lecun.com/exdb/mnist/t10k-images-idx3-ubyte.gz";
    let test_labels_url = "http://yann.lecun.com/exdb/mnist/t10k-labels-idx1-ubyte.gz";
    
    let temp_dir = format!("{}/mnist_temp", output_dir);
    fs::create_dir_all(&temp_dir)?;
    
    // Download files
    println!("Downloading MNIST dataset files...");
    download_file(train_images_url, &format!("{}/train-images-idx3-ubyte.gz", temp_dir))?;
    download_file(train_labels_url, &format!("{}/train-labels-idx1-ubyte.gz", temp_dir))?;
    download_file(test_images_url, &format!("{}/t10k-images-idx3-ubyte.gz", temp_dir))?;
    download_file(test_labels_url, &format!("{}/t10k-labels-idx1-ubyte.gz", temp_dir))?;
    
    // Extract files
    println!("Extracting MNIST dataset...");
    extract_gz(&format!("{}/train-images-idx3-ubyte.gz", temp_dir))?;
    extract_gz(&format!("{}/train-labels-idx1-ubyte.gz", temp_dir))?;
    extract_gz(&format!("{}/t10k-images-idx3-ubyte.gz", temp_dir))?;
    extract_gz(&format!("{}/t10k-labels-idx1-ubyte.gz", temp_dir))?;
    
    // Process MNIST dataset into the required format
    println!("Processing MNIST dataset...");
    process_mnist(&temp_dir, output_dir)?;
    
    // Clean up temporary files
    fs::remove_dir_all(&temp_dir)?;
    
    Ok(())
}

fn download_fashion_mnist(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // URLs for Fashion-MNIST dataset
    let train_images_url = "http://fashion-mnist.s3-website.eu-central-1.amazonaws.com/train-images-idx3-ubyte.gz";
    let train_labels_url = "http://fashion-mnist.s3-website.eu-central-1.amazonaws.com/train-labels-idx1-ubyte.gz";
    let test_images_url = "http://fashion-mnist.s3-website.eu-central-1.amazonaws.com/t10k-images-idx3-ubyte.gz";
    let test_labels_url = "http://fashion-mnist.s3-website.eu-central-1.amazonaws.com/t10k-labels-idx1-ubyte.gz";
    
    let temp_dir = format!("{}/fashion_mnist_temp", output_dir);
    fs::create_dir_all(&temp_dir)?;
    
    // Download files
    println!("Downloading Fashion-MNIST dataset files...");
    download_file(train_images_url, &format!("{}/train-images-idx3-ubyte.gz", temp_dir))?;
    download_file(train_labels_url, &format!("{}/train-labels-idx1-ubyte.gz", temp_dir))?;
    download_file(test_images_url, &format!("{}/t10k-images-idx3-ubyte.gz", temp_dir))?;
    download_file(test_labels_url, &format!("{}/t10k-labels-idx1-ubyte.gz", temp_dir))?;
    
    // Extract files
    println!("Extracting Fashion-MNIST dataset...");
    extract_gz(&format!("{}/train-images-idx3-ubyte.gz", temp_dir))?;
    extract_gz(&format!("{}/train-labels-idx1-ubyte.gz", temp_dir))?;
    extract_gz(&format!("{}/t10k-images-idx3-ubyte.gz", temp_dir))?;
    extract_gz(&format!("{}/t10k-labels-idx1-ubyte.gz", temp_dir))?;
    
    // Process Fashion-MNIST dataset into the required format
    println!("Processing Fashion-MNIST dataset...");
    process_fashion_mnist(&temp_dir, output_dir)?;
    
    // Clean up temporary files
    fs::remove_dir_all(&temp_dir)?;
    
    Ok(())
}

fn download_file(url: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading {} to {}", url, dest);
    let status = Command::new("curl")
        .args(["-L", "-o", dest, url])
        .status()?;
    
    if !status.success() {
        return Err(format!("Failed to download {}", url).into());
    }
    
    Ok(())
}

fn extract_gz(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = file_path.trim_end_matches(".gz");
    println!("Extracting {} to {}", file_path, output_path);
    
    let status = Command::new("gunzip")
        .args(["-c", file_path])
        .stdout(File::create(output_path)?)
        .status()?;
    
    if !status.success() {
        return Err(format!("Failed to extract {}", file_path).into());
    }
    
    Ok(())
}

fn process_mnist(temp_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Class names for MNIST
    let class_names = [
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"
    ];
    
    // Create class directories
    for class_name in &class_names {
        fs::create_dir_all(format!("{}/class_{}", output_dir, class_name))?;
    }
    
    // Process training images
    let train_images_path = format!("{}/train-images-idx3-ubyte", temp_dir);
    let train_labels_path = format!("{}/train-labels-idx1-ubyte", temp_dir);
    
    // Read training labels
    let train_labels = read_idx_labels(&train_labels_path)?;
    
    // Read and process training images
    let train_images = read_idx_images(&train_images_path)?;
    
    println!("Processing training images...");
    for (i, (image, &label)) in train_images.iter().zip(train_labels.iter()).enumerate() {
        let class_name = class_names[label as usize];
        let output_path = format!("{}/class_{}/train_{}.png", output_dir, class_name, i);
        save_image(image, 28, 28, &output_path)?;
    }
    
    // Process test images
    let test_images_path = format!("{}/t10k-images-idx3-ubyte", temp_dir);
    let test_labels_path = format!("{}/t10k-labels-idx1-ubyte", temp_dir);
    
    // Read test labels
    let test_labels = read_idx_labels(&test_labels_path)?;
    
    // Read and process test images
    let test_images = read_idx_images(&test_images_path)?;
    
    println!("Processing test images...");
    for (i, (image, &label)) in test_images.iter().zip(test_labels.iter()).enumerate() {
        let class_name = class_names[label as usize];
        let output_path = format!("{}/class_{}/test_{}.png", output_dir, class_name, i);
        save_image(image, 28, 28, &output_path)?;
    }
    
    Ok(())
}

fn process_fashion_mnist(temp_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Class names for Fashion-MNIST
    let class_names = [
        "tshirt_top", "trouser", "pullover", "dress", "coat", 
        "sandal", "shirt", "sneaker", "bag", "ankle_boot"
    ];
    
    // Create class directories
    for class_name in &class_names {
        fs::create_dir_all(format!("{}/{}", output_dir, class_name))?;
    }
    
    // Process training images
    let train_images_path = format!("{}/train-images-idx3-ubyte", temp_dir);
    let train_labels_path = format!("{}/train-labels-idx1-ubyte", temp_dir);
    
    // Read training labels
    let train_labels = read_idx_labels(&train_labels_path)?;
    
    // Read and process training images
    let train_images = read_idx_images(&train_images_path)?;
    
    println!("Processing training images...");
    for (i, (image, &label)) in train_images.iter().zip(train_labels.iter()).enumerate() {
        let class_name = class_names[label as usize];
        let output_path = format!("{}/{}/train_{}.png", output_dir, class_name, i);
        save_image(image, 28, 28, &output_path)?;
    }
    
    // Process test images
    let test_images_path = format!("{}/t10k-images-idx3-ubyte", temp_dir);
    let test_labels_path = format!("{}/t10k-labels-idx1-ubyte", temp_dir);
    
    // Read test labels
    let test_labels = read_idx_labels(&test_labels_path)?;
    
    // Read and process test images
    let test_images = read_idx_images(&test_images_path)?;
    
    println!("Processing test images...");
    for (i, (image, &label)) in test_images.iter().zip(test_labels.iter()).enumerate() {
        let class_name = class_names[label as usize];
        let output_path = format!("{}/{}/test_{}.png", output_dir, class_name, i);
        save_image(image, 28, 28, &output_path)?;
    }
    
    Ok(())
}

fn read_idx_labels(file_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let data = fs::read(file_path)?;
    
    // Check magic number (should be 2049 for labels)
    let magic_number = ((data[0] as u32) << 24) | ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32);
    if magic_number != 2049 {
        return Err(format!("Invalid magic number for labels: {}", magic_number).into());
    }
    
    // Read number of items
    let num_items = ((data[4] as u32) << 24) | ((data[5] as u32) << 16) | ((data[6] as u32) << 8) | (data[7] as u32);
    
    // Extract labels (starting from byte 8)
    let labels = data[8..8 + num_items as usize].to_vec();
    
    Ok(labels)
}

fn read_idx_images(file_path: &str) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let data = fs::read(file_path)?;
    
    // Check magic number (should be 2051 for images)
    let magic_number = ((data[0] as u32) << 24) | ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32);
    if magic_number != 2051 {
        return Err(format!("Invalid magic number for images: {}", magic_number).into());
    }
    
    // Read header information
    let num_images = ((data[4] as u32) << 24) | ((data[5] as u32) << 16) | ((data[6] as u32) << 8) | (data[7] as u32);
    let num_rows = ((data[8] as u32) << 24) | ((data[9] as u32) << 16) | ((data[10] as u32) << 8) | (data[11] as u32);
    let num_cols = ((data[12] as u32) << 24) | ((data[13] as u32) << 16) | ((data[14] as u32) << 8) | (data[15] as u32);
    
    let image_size = (num_rows * num_cols) as usize;
    let mut images = Vec::with_capacity(num_images as usize);
    
    // Extract images (starting from byte 16)
    for i in 0..num_images as usize {
        let start = 16 + i * image_size;
        let end = start + image_size;
        images.push(data[start..end].to_vec());
    }
    
    Ok(images)
}

fn save_image(pixels: &[u8], width: usize, height: usize, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::BufWriter;
    let file = File::create(output_path)?;
    let ref mut w = BufWriter::new(file);
    
    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    
    let mut writer = encoder.write_header()?;
    
    // Convert single channel grayscale to RGB
    writer.write_image_data(pixels)?;
    
    Ok(())
}
