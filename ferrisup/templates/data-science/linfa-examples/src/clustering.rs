use anyhow::Result;
use linfa::prelude::*;
use ndarray::{Array1, Array2, Ix1};
use std::path::Path;
use std::fs::File;

use crate::data_utils;

pub fn run_clustering_example() -> Result<()> {
    println!("Linfa 0.7.1 Clustering Example");
    
    // Check for data files in different formats
    let csv_path = Path::new("data/sample_clustering.csv");
    let json_path = Path::new("data/sample_clustering.json");
    
    let dataset = if csv_path.exists() {
        println!("Loading data from CSV file: {}", csv_path.display());
        load_clustering_dataset(csv_path)?
    } else if json_path.exists() {
        println!("Loading data from JSON file: {}", json_path.display());
        load_clustering_dataset(json_path)?
    } else {
        println!("No data files found, using synthetic data");
        create_synthetic_clustering_dataset()?
    };
    
    println!("Dataset shape: [{}, {}]", dataset.nsamples(), dataset.nfeatures());
    println!("Number of samples: {}", dataset.nsamples());
    
    // Display dataset information
    println!("Dataset information:");
    println!("  Number of samples: {}", dataset.nsamples());
    println!("  Feature dimension: {}", dataset.nfeatures());
    
    // Display first few records
    println!("First 5 records:");
    for i in 0..std::cmp::min(5, dataset.nsamples()) {
        println!("  Record {}: {:?}", i, dataset.records().row(i));
    }
    
    // Perform simple clustering analysis
    println!("\nPerforming simple clustering analysis...");
    
    // Calculate the centroid of the dataset
    let mut centroid_x = 0.0;
    let mut centroid_y = 0.0;
    
    for i in 0..dataset.nsamples() {
        centroid_x += dataset.records().row(i)[0];
        centroid_y += dataset.records().row(i)[1];
    }
    
    centroid_x /= dataset.nsamples() as f64;
    centroid_y /= dataset.nsamples() as f64;
    
    println!("Dataset centroid: ({:.2}, {:.2})", centroid_x, centroid_y);
    
    // Calculate average distance from centroid
    let mut total_distance = 0.0;
    
    for i in 0..dataset.nsamples() {
        let x = dataset.records().row(i)[0];
        let y = dataset.records().row(i)[1];
        
        let distance = ((x - centroid_x).powi(2) + (y - centroid_y).powi(2)).sqrt();
        total_distance += distance;
    }
    
    let avg_distance = total_distance / dataset.nsamples() as f64;
    println!("Average distance from centroid: {:.2}", avg_distance);
    
    Ok(())
}

// Function to create a synthetic clustering dataset
fn create_synthetic_clustering_dataset() -> Result<Dataset<f64, f64, Ix1>> {
    use rand::Rng;
    use rand_xoshiro::Xoshiro256Plus;
    use rand::SeedableRng;
    
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let num_clusters = 3;
    let points_per_cluster = 5;
    let num_samples = num_clusters * points_per_cluster;
    
    let mut features_data = Vec::with_capacity(num_samples * 2);
    
    // Create clusters of points
    for cluster in 0..num_clusters {
        let x_center = match cluster {
            0 => 1.0,
            1 => 5.0,
            _ => 3.0,
        };
        
        let y_center = match cluster {
            0 => 1.0,
            1 => 1.0,
            _ => 5.0,
        };
        
        for _ in 0..points_per_cluster {
            // Add some noise
            let x = x_center + rng.gen_range(-0.5..0.5);
            let y = y_center + rng.gen_range(-0.5..0.5);
            
            features_data.push(x);
            features_data.push(y);
        }
    }
    
    // Calculate number of samples
    let num_samples = features_data.len() / 2;
    
    // Create feature array (each sample has 2 features)
    let features = Array2::from_shape_vec((num_samples, 2), features_data)?;
    
    // For clustering, we don't need targets, but Linfa's Dataset requires them
    // We'll use dummy targets (all zeros)
    let targets = Array1::zeros(num_samples);
    
    Ok(Dataset::new(features, targets))
}

// Function to load clustering dataset from file (auto-detect format)
fn load_clustering_dataset(path: &Path) -> Result<Dataset<f64, f64, Ix1>> {
    let format = data_utils::detect_file_format(path)?;
    
    match format {
        "csv" => load_csv_clustering_dataset(path),
        "json" => load_json_clustering_dataset(path),
        _ => Err(anyhow::anyhow!("Unsupported file format: {}", format)),
    }
}

// Function to load clustering dataset from CSV
fn load_csv_clustering_dataset(path: &Path) -> Result<Dataset<f64, f64, Ix1>> {
    use csv;
    
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    
    let mut features_data = Vec::new();
    
    for result in reader.records() {
        let record = result?;
        
        if record.len() >= 2 {
            // Two columns for x, y coordinates
            let x = record[0].parse::<f64>()?;
            let y = record[1].parse::<f64>()?;
            features_data.push(x);
            features_data.push(y);
        }
    }
    
    // Calculate number of samples
    let num_samples = features_data.len() / 2;
    
    // Create feature array (each sample has 2 features)
    let features = Array2::from_shape_vec((num_samples, 2), features_data)?;
    
    // For clustering, we don't need targets, but Linfa's Dataset requires them
    // We'll use dummy targets (all zeros)
    let targets = Array1::zeros(num_samples);
    
    Ok(Dataset::new(features, targets))
}

// Function to load clustering dataset from JSON
fn load_json_clustering_dataset(path: &Path) -> Result<Dataset<f64, f64, Ix1>> {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct ClusteringPoint {
        x: f64,
        y: f64,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    struct ClusteringDataSet {
        data: Vec<ClusteringPoint>,
    }
    
    let file = File::open(path)?;
    let dataset: ClusteringDataSet = serde_json::from_reader(file)?;
    
    let mut features_data = Vec::new();
    
    for point in dataset.data {
        features_data.push(point.x);
        features_data.push(point.y);
    }
    
    // Calculate number of samples
    let num_samples = features_data.len() / 2;
    
    // Create feature array (each sample has 2 features)
    let features = Array2::from_shape_vec((num_samples, 2), features_data)?;
    
    // For clustering, we don't need targets, but Linfa's Dataset requires them
    // We'll use dummy targets (all zeros)
    let targets = Array1::zeros(num_samples);
    
    Ok(Dataset::new(features, targets))
}