use anyhow::Result;
use clap::{Arg, Command, Parser};
use ndarray::{Array1, Array2, Ix1};
use std::env;
use std::fs::{self, File};
use std::path::Path;
use linfa::prelude::*;
use rand::Rng;
use rand_xoshiro::Xoshiro256Plus;
use rand::SeedableRng;
use csv;
use serde_json;

mod clustering;
mod regression;
mod classification;
mod decision_tree;
mod data_utils;

fn print_usage() {
    println!("Linfa 0.7.1 Examples");
    println!("Usage: cargo run -- [example]");
    println!("Available examples:");
    println!("  classification - Run classification example with LogisticRegression");
    println!("  clustering     - Run clustering example");
    println!("  regression     - Run regression example with LinearRegression");
    println!("  decision_tree  - Run classification example with Decision Tree");
    println!("  all            - Run all examples sequentially");
    println!("  generate       - Generate sample data files in CSV and JSON formats");
    println!("  help           - Show this help message");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "classification" => classification::run_logistic_regression_example()?,
        "clustering" => clustering::run_clustering_example()?,
        "regression" => regression::run_regression_example()?,
        "decision_tree" => decision_tree::run_decision_tree_example()?,
        "help" => print_usage(),
        "generate" => {
            if args.len() < 3 {
                println!("Please specify what kind of data to generate: classification, regression, clustering");
                return Ok(());
            }
            
            let data_type = args[2].as_str();
            let format = if args.len() >= 4 { args[3].as_str() } else { "all" };
            
            match data_type {
                "classification" => generate_classification_data(format)?,
                "regression" => generate_regression_data(format)?,
                "clustering" => generate_clustering_data(format)?,
                _ => {
                    println!("Unknown data type: {}", data_type);
                    println!("Available data types: classification, regression, clustering");
                }
            }
        },
        "all" => {
            println!("\n=== Running LogisticRegression Classification Example ===\n");
            classification::run_logistic_regression_example()?;
            
            println!("\n=== Running Decision Tree Classification Example ===\n");
            decision_tree::run_decision_tree_example()?;
            
            println!("\n=== Running Linear Regression Example ===\n");
            regression::run_regression_example()?;
            
            println!("\n=== Running Clustering Example ===\n");
            clustering::run_clustering_example()?;
            
            println!("\n=== All examples completed successfully ===\n");
        },
        _ => {
            println!("Unknown example: {}", args[1]);
            print_usage();
        }
    }
    
    Ok(())
}

// Function to generate classification data in different formats
fn generate_classification_data(format: &str) -> Result<()> {
    println!("Generating classification data...");
    
    // Create synthetic dataset
    let dataset = create_synthetic_classification_dataset()?;
    
    // Create data directory if it doesn't exist
    fs::create_dir_all("data")?;
    
    // Save in requested format(s)
    match format {
        "csv" => {
            save_classification_csv(&dataset, "data/sample_classification.csv")?;
            println!("Generated CSV data: data/sample_classification.csv");
        },
        "json" => {
            save_classification_json(&dataset, "data/sample_classification.json")?;
            println!("Generated JSON data: data/sample_classification.json");
        },
        "all" => {
            save_classification_csv(&dataset, "data/sample_classification.csv")?;
            save_classification_json(&dataset, "data/sample_classification.json")?;
            println!("Generated data in all formats:");
            println!("  CSV: data/sample_classification.csv");
            println!("  JSON: data/sample_classification.json");
        },
        _ => {
            println!("Unknown format: {}", format);
            println!("Available formats: csv, json, all");
        }
    }
    
    Ok(())
}

// Function to generate regression data in different formats
fn generate_regression_data(format: &str) -> Result<()> {
    println!("Generating regression data...");
    
    // Create synthetic dataset
    let dataset = create_synthetic_regression_dataset()?;
    
    // Create data directory if it doesn't exist
    fs::create_dir_all("data")?;
    
    // Save in requested format(s)
    match format {
        "csv" => {
            save_regression_csv(&dataset, "data/sample_regression.csv")?;
            println!("Generated CSV data: data/sample_regression.csv");
        },
        "json" => {
            save_regression_json(&dataset, "data/sample_regression.json")?;
            println!("Generated JSON data: data/sample_regression.json");
        },
        "all" => {
            save_regression_csv(&dataset, "data/sample_regression.csv")?;
            save_regression_json(&dataset, "data/sample_regression.json")?;
            println!("Generated data in all formats:");
            println!("  CSV: data/sample_regression.csv");
            println!("  JSON: data/sample_regression.json");
        },
        _ => {
            println!("Unknown format: {}", format);
            println!("Available formats: csv, json, all");
        }
    }
    
    Ok(())
}

// Function to generate clustering data in different formats
fn generate_clustering_data(format: &str) -> Result<()> {
    println!("Generating clustering data...");
    
    // Create synthetic dataset
    let dataset = create_synthetic_clustering_dataset()?;
    
    // Create data directory if it doesn't exist
    fs::create_dir_all("data")?;
    
    // Save in requested format(s)
    match format {
        "csv" => {
            save_clustering_csv(&dataset, "data/sample_clustering.csv")?;
            println!("Generated CSV data: data/sample_clustering.csv");
        },
        "json" => {
            save_clustering_json(&dataset, "data/sample_clustering.json")?;
            println!("Generated JSON data: data/sample_clustering.json");
        },
        "all" => {
            save_clustering_csv(&dataset, "data/sample_clustering.csv")?;
            save_clustering_json(&dataset, "data/sample_clustering.json")?;
            println!("Generated data in all formats:");
            println!("  CSV: data/sample_clustering.csv");
            println!("  JSON: data/sample_clustering.json");
        },
        _ => {
            println!("Unknown format: {}", format);
            println!("Available formats: csv, json, all");
        }
    }
    
    Ok(())
}

// Function to create a synthetic classification dataset
fn create_synthetic_classification_dataset() -> Result<Dataset<f64, usize, Ix1>> {
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let num_samples = 100;
    
    let mut features_data = Vec::with_capacity(num_samples);
    let mut targets_data = Vec::with_capacity(num_samples);
    
    // Create two clusters of points
    for i in 0..num_samples {
        let cluster = i % 2; // Alternating between cluster 0 and 1
        
        let (center_x, center_y) = if cluster == 0 {
            (0.0, 0.0) // Cluster 0 centered at origin
        } else {
            (3.0, 3.0) // Cluster 1 centered at (3,3)
        };
        
        // Add some noise
        let x = center_x + rng.gen_range(-0.5..0.5);
        let _y = center_y + rng.gen_range(-0.5..0.5);
        
        features_data.push(x);
        targets_data.push(cluster);
    }
    
    // Create feature array (each sample has 1 feature)
    let features = Array2::from_shape_vec((num_samples, 1), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}

// Function to create a synthetic regression dataset
fn create_synthetic_regression_dataset() -> Result<Dataset<f64, f64, Ix1>> {
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let num_samples = 100;
    
    let mut features_data = Vec::with_capacity(num_samples);
    let mut targets_data = Vec::with_capacity(num_samples);
    
    // Create a simple linear relationship: y = 2x + 1 + noise
    for _ in 0..num_samples {
        let x = rng.gen_range(0.0..10.0);
        let noise = rng.gen_range(-1.0..1.0);
        let y = 2.0 * x + 1.0 + noise;
        
        features_data.push(x);
        targets_data.push(y);
    }
    
    // Create feature array (each sample has 1 feature)
    let features = Array2::from_shape_vec((num_samples, 1), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}

// Function to create a synthetic clustering dataset
fn create_synthetic_clustering_dataset() -> Result<Dataset<f64, f64, Ix1>> {
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let num_samples = 100;
    
    let mut features_data = Vec::with_capacity(num_samples * 2);
    let mut targets_data = Vec::with_capacity(num_samples);
    
    // Create three clusters
    let centers = [(0.0, 0.0), (5.0, 5.0), (0.0, 5.0)];
    
    for i in 0..num_samples {
        let cluster_idx = i % 3;
        let (center_x, center_y) = centers[cluster_idx];
        
        // Add noise to create a cluster
        let x = center_x + rng.gen_range(-0.5..0.5);
        let y = center_y + rng.gen_range(-0.5..0.5);
        
        features_data.push(x);
        features_data.push(y);
        targets_data.push(0.0); // Dummy target (not used for clustering)
    }
    
    // Create feature array (each sample has 2 features)
    let features = Array2::from_shape_vec((num_samples, 2), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}

// Functions to save classification data in different formats
fn save_classification_csv(dataset: &Dataset<f64, usize, Ix1>, path: &str) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);
    
    // Write header
    writer.write_record(&["x", "y", "class"])?;
    
    // Write data
    for i in 0..dataset.nsamples() {
        let x = dataset.records().row(i)[0];
        let y = 0.0; // Add a dummy y value for compatibility with the classification example
        let class = dataset.targets()[i];
        writer.write_record(&[x.to_string(), y.to_string(), class.to_string()])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn save_classification_json(dataset: &Dataset<f64, usize, Ix1>, path: &str) -> Result<()> {
    #[derive(serde::Serialize)]
    struct DataPoint {
        x: f64,
        y: f64,
        target: usize,
    }
    
    #[derive(serde::Serialize)]
    struct DataSet {
        data: Vec<DataPoint>,
    }
    
    let mut data_points = Vec::new();
    
    for i in 0..dataset.nsamples() {
        let x = dataset.records().row(i)[0];
        let y = 0.0; // Add a dummy y value for compatibility with the classification example
        let target = dataset.targets()[i];
        
        data_points.push(DataPoint { x, y, target });
    }
    
    let dataset = DataSet { data: data_points };
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &dataset)?;
    
    Ok(())
}

// Functions to save regression data in different formats
fn save_regression_csv(dataset: &Dataset<f64, f64, Ix1>, path: &str) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);
    
    // Write header
    writer.write_record(&["x", "y"])?;
    
    // Write data
    for i in 0..dataset.nsamples() {
        let x = dataset.records().row(i)[0];
        let y = dataset.targets()[i];
        writer.write_record(&[x.to_string(), y.to_string()])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn save_regression_json(dataset: &Dataset<f64, f64, Ix1>, path: &str) -> Result<()> {
    #[derive(serde::Serialize)]
    struct DataPoint {
        x: f64,
        y: f64,
    }
    
    #[derive(serde::Serialize)]
    struct DataSet {
        data: Vec<DataPoint>,
    }
    
    let mut data_points = Vec::new();
    
    for i in 0..dataset.nsamples() {
        let x = dataset.records().row(i)[0];
        let y = dataset.targets()[i];
        
        data_points.push(DataPoint { x, y });
    }
    
    let dataset_json = DataSet { data: data_points };
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &dataset_json)?;
    
    Ok(())
}

// Functions to save clustering data in different formats
fn save_clustering_csv(dataset: &Dataset<f64, f64, Ix1>, path: &str) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);
    
    // Write header
    writer.write_record(&["x", "y"])?;
    
    // Write data
    for i in 0..dataset.nsamples() {
        let x = dataset.records().row(i)[0];
        let y = dataset.records().row(i)[1];
        writer.write_record(&[x.to_string(), y.to_string()])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn save_clustering_json(dataset: &Dataset<f64, f64, Ix1>, path: &str) -> Result<()> {
    #[derive(serde::Serialize)]
    struct DataPoint {
        x: f64,
        y: f64,
    }
    
    #[derive(serde::Serialize)]
    struct DataSet {
        data: Vec<DataPoint>,
    }
    
    let mut data_points = Vec::new();
    
    for i in 0..dataset.nsamples() {
        let x = dataset.records().row(i)[0];
        let y = dataset.records().row(i)[1];
        
        data_points.push(DataPoint { x, y });
    }
    
    let dataset_json = DataSet { data: data_points };
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &dataset_json)?;
    
    Ok(())
}
