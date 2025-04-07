use anyhow::{anyhow, Result};
use linfa::prelude::*;
use ndarray::{Array1, Array2, Ix1};
use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use csv;
use rand::Rng;
use rand_xoshiro::Xoshiro256Plus;
use rand::SeedableRng;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct DataPoint {
    x: f64,
    y: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DataSet {
    data: Vec<DataPoint>,
}

/// Load dataset from CSV file
pub fn load_csv_dataset(path: &Path) -> Result<Dataset<f64, f64, Ix1>> {
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(file);
    
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    for result in reader.records() {
        let record = result?;
        
        if record.len() >= 2 {
            // First column is x (feature)
            let x = record[0].parse::<f64>()?;
            features_data.push(x);
            
            // Second column is y (target)
            let y = record[1].parse::<f64>()?;
            targets_data.push(y);
        }
    }
    
    create_dataset(features_data, targets_data)
}

/// Load dataset from JSON file
pub fn load_json_dataset(path: &Path) -> Result<Dataset<f64, f64, Ix1>> {
    let file = File::open(path)?;
    let dataset: DataSet = serde_json::from_reader(file)?;
    
    let mut features_data = Vec::new();
    let mut targets_data = Vec::new();
    
    for point in dataset.data {
        features_data.push(point.x);
        targets_data.push(point.y);
    }
    
    create_dataset(features_data, targets_data)
}

/// Create a dataset from features and targets
fn create_dataset(features_data: Vec<f64>, targets_data: Vec<f64>) -> Result<Dataset<f64, f64, Ix1>> {
    // Calculate number of samples
    let num_samples = targets_data.len();
    
    if num_samples == 0 {
        return Err(anyhow!("No data points found in the dataset"));
    }
    
    // Create feature array (each sample has 1 feature)
    let features = Array2::from_shape_vec((num_samples, 1), features_data)?;
    let targets = Array1::from_vec(targets_data);
    
    Ok(Dataset::new(features, targets))
}

/// Create a synthetic dataset for regression
pub fn create_synthetic_regression_dataset() -> Result<Dataset<f64, f64, Ix1>> {
    let mut rng = Xoshiro256Plus::seed_from_u64(42);
    let num_samples = 6;
    
    let mut features_data = Vec::with_capacity(num_samples);
    let mut targets_data = Vec::with_capacity(num_samples);
    
    // Create a simple linear relationship: y = 2x + 1 + noise
    for _ in 0..num_samples {
        let x = rng.gen_range(0.0..5.0);
        let noise = rng.gen_range(-0.1..0.1);
        let y = 2.0 * x + 1.0 + noise;
        
        features_data.push(x);
        targets_data.push(y);
    }
    
    create_dataset(features_data, targets_data)
}

/// Save dataset to CSV file
pub fn save_csv_dataset(dataset: &Dataset<f64, f64, Ix1>, path: &Path) -> Result<()> {
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

/// Save dataset to JSON file
pub fn save_json_dataset(dataset: &Dataset<f64, f64, Ix1>, path: &Path) -> Result<()> {
    let mut data_points = Vec::new();
    
    for i in 0..dataset.nsamples() {
        let x = dataset.records().row(i)[0];
        let y = dataset.targets()[i];
        
        data_points.push(DataPoint { x, y });
    }
    
    let dataset = DataSet { data: data_points };
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &dataset)?;
    
    Ok(())
}

/// Detect file format from extension
pub fn detect_file_format(path: &Path) -> Result<&'static str> {
    let extension = path.extension()
        .ok_or_else(|| anyhow!("File has no extension"))?
        .to_str()
        .ok_or_else(|| anyhow!("Invalid file extension"))?;
    
    match extension.to_lowercase().as_str() {
        "csv" => Ok("csv"),
        "json" => Ok("json"),
        _ => Err(anyhow!("Unsupported file format: {}", extension)),
    }
}

/// Load dataset from file (auto-detect format)
pub fn load_dataset(path: &Path) -> Result<Dataset<f64, f64, Ix1>> {
    let format = detect_file_format(path)?;
    
    match format {
        "csv" => load_csv_dataset(path),
        "json" => load_json_dataset(path),
        _ => Err(anyhow!("Unsupported file format: {}", format)),
    }
}

/// Create sample datasets in CSV and JSON formats
pub fn create_sample_datasets() -> Result<()> {
    let dataset = create_synthetic_regression_dataset()?;
    
    // Create data directory if it doesn't exist
    std::fs::create_dir_all("data")?;
    
    // Save in CSV format
    save_csv_dataset(&dataset, Path::new("data/sample_regression.csv"))?;
    
    // Save in JSON format
    save_json_dataset(&dataset, Path::new("data/sample_regression.json"))?;
    
    Ok(())
}
