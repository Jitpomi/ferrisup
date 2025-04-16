use std::fs;
use std::path::{Path, PathBuf};
use image::{DynamicImage, ImageBuffer, Rgb};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use clap::Parser;

/// Command line arguments for the synthetic dataset generator
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output directory for the dataset
    #[arg(short, long, default_value = "sample-data")]
    output_dir: String,
    
    /// Number of classes to generate
    #[arg(short = 'c', long, default_value_t = 10)]
    num_classes: usize,
    
    /// Number of images per class
    #[arg(short = 'n', long, default_value_t = 100)]
    images_per_class: usize,
    
    /// Image size (width and height in pixels)
    #[arg(short = 's', long, default_value_t = 32)]
    image_size: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("Generating synthetic dataset...");
    println!("Output directory: {}", args.output_dir);
    println!("Number of classes: {}", args.num_classes);
    println!("Images per class: {}", args.images_per_class);
    println!("Image size: {}", args.image_size);
    
    // Create output directory
    let output_path = Path::new(&args.output_dir);
    if output_path.exists() {
        fs::remove_dir_all(output_path)?;
    }
    fs::create_dir_all(output_path)?;
    
    // Create a seeded random number generator for reproducibility
    let mut rng = StdRng::seed_from_u64(42);
    
    // Generate classes
    for class_idx in 0..args.num_classes {
        let class_name = format!("class_{}", class_idx);
        let class_dir = output_path.join(&class_name);
        fs::create_dir_all(&class_dir)?;
        
        println!("Generating images for class: {}", class_name);
        
        // Base hue for this class (evenly distributed around the color wheel)
        let base_hue = (class_idx as f32) / (args.num_classes as f32);
        
        // Generate images for this class
        for img_idx in 0..args.images_per_class {
            // Vary the hue slightly for each image
            let hue_variation = rng.gen_range(-0.05..0.05);
            let hue = (base_hue + hue_variation + 1.0) % 1.0; // Keep in [0, 1] range
            
            // Create a colored image
            let img = create_colored_image(args.image_size, hue, &mut rng);
            
            // Save the image
            let img_path = class_dir.join(format!("image_{}.png", img_idx));
            img.save(img_path)?;
        }
    }
    
    println!("Synthetic dataset generation complete!");
    println!("Created {} images across {} classes", args.num_classes * args.images_per_class, args.num_classes);
    println!("Dataset is available in: {}", args.output_dir);
    
    Ok(())
}

/// Create a colored image with a specific hue and some random patterns
fn create_colored_image(size: usize, hue: f32, rng: &mut StdRng) -> DynamicImage {
    let mut img = ImageBuffer::new(size as u32, size as u32);
    
    // Base saturation and value
    let saturation = rng.gen_range(0.7..0.9);
    let value = rng.gen_range(0.7..0.9);
    
    // Generate random patterns
    let pattern_type = rng.gen_range(0..4);
    
    match pattern_type {
        0 => {
            // Solid color with noise
            for y in 0..size {
                for x in 0..size {
                    let noise = rng.gen_range(-0.1..0.1);
                    let h = (hue + noise + 1.0) % 1.0;
                    let (r, g, b) = hsv_to_rgb(h, saturation, value);
                    img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
                }
            }
        },
        1 => {
            // Gradient
            for y in 0..size {
                for x in 0..size {
                    let gradient = (x as f32) / (size as f32);
                    let h = (hue + gradient * 0.2 + 1.0) % 1.0;
                    let (r, g, b) = hsv_to_rgb(h, saturation, value);
                    img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
                }
            }
        },
        2 => {
            // Concentric circles
            let center_x = size as f32 / 2.0;
            let center_y = size as f32 / 2.0;
            
            for y in 0..size {
                for x in 0..size {
                    let dx = x as f32 - center_x;
                    let dy = y as f32 - center_y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    let normalized_distance = distance / (size as f32 / 2.0);
                    
                    let h = (hue + normalized_distance * 0.3 + 1.0) % 1.0;
                    let (r, g, b) = hsv_to_rgb(h, saturation, value);
                    img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
                }
            }
        },
        _ => {
            // Checkered pattern
            let check_size = size / 8;
            
            for y in 0..size {
                for x in 0..size {
                    let check_x = x / check_size;
                    let check_y = y / check_size;
                    
                    let h = if (check_x + check_y) % 2 == 0 {
                        hue
                    } else {
                        (hue + 0.5) % 1.0
                    };
                    
                    let (r, g, b) = hsv_to_rgb(h, saturation, value);
                    img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
                }
            }
        }
    }
    
    DynamicImage::ImageRgb8(img)
}

/// Convert HSV to RGB
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
