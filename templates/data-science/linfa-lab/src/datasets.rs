use anyhow::Result;
use csv::ReaderBuilder;
use linfa::prelude::*;
use linfa_datasets::{iris, diabetes, winequality};
use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use ndarray_csv::Array2Reader;
use ndarray_rand::rand::SeedableRng;
use ndarray_rand::rand_distr::{StandardNormal, Uniform};
use ndarray_rand::RandomExt;
use rand::rngs::StdRng;
use rand_distr::Normal;
use std::fs::File;
use std::path::Path;

/// Load the Iris dataset
pub fn load_iris() -> Result<Dataset<f64, usize>> {
    Ok(iris())
}

/// Load the Iris dataset features only
pub fn load_iris_features() -> Result<Array2<f64>> {
    Ok(iris().records)
}

/// Load the Iris dataset with targets
pub fn load_iris_with_targets() -> Result<(Array2<f64>, Array1<usize>)> {
    let dataset = iris();
    Ok((dataset.records, dataset.targets))
}

/// Load the Diabetes dataset
pub fn load_diabetes() -> Result<Dataset<f64, f64>> {
    Ok(diabetes())
}

/// Load the Wine Quality dataset
pub fn load_winequality() -> Result<Dataset<f64, f64>> {
    Ok(winequality())
}

/// Load the Wine Quality dataset as a classification problem
pub fn load_winequality_classification() -> Result<Dataset<f64, usize>> {
    let dataset = winequality();
    let targets = dataset.targets.mapv(|x| x as usize);
    Ok(Dataset::new(dataset.records, targets))
}

/// Load the Wine Quality dataset features only
pub fn load_winequality_features() -> Result<Array2<f64>> {
    Ok(winequality().records)
}

/// Load the Wine Quality dataset with targets
pub fn load_winequality_with_targets() -> Result<(Array2<f64>, Array1<usize>)> {
    let dataset = winequality();
    Ok((dataset.records, dataset.targets.mapv(|x| x as usize)))
}

/// Load a CSV file as a regression dataset
pub fn load_csv<P: AsRef<Path>>(path: P, target_column: &str) -> Result<Dataset<f64, f64>> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    
    let headers = reader.headers()?.clone();
    let target_idx = headers.iter().position(|h| h == target_column)
        .ok_or_else(|| anyhow::anyhow!("Target column '{}' not found", target_column))?;
    
    let file = File::open(path.as_ref())?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    
    let array: Array2<f64> = reader.deserialize_array2_dynamic()?;
    
    let n_rows = array.nrows();
    let n_cols = array.ncols();
    
    let mut features = Array2::zeros((n_rows, n_cols - 1));
    let mut targets = Array1::zeros(n_rows);
    
    for i in 0..n_rows {
        let mut feature_idx = 0;
        for j in 0..n_cols {
            if j == target_idx {
                targets[i] = array[[i, j]];
            } else {
                features[[i, feature_idx]] = array[[i, j]];
                feature_idx += 1;
            }
        }
    }
    
    Ok(Dataset::new(features, targets))
}

/// Load a CSV file as a classification dataset
pub fn load_csv_classification<P: AsRef<Path>>(path: P, target_column: &str) -> Result<Dataset<f64, usize>> {
    let dataset = load_csv(path, target_column)?;
    let targets = dataset.targets.mapv(|x| x as usize);
    Ok(Dataset::new(dataset.records, targets))
}

/// Load a CSV file features only
pub fn load_csv_features<P: AsRef<Path>>(path: P) -> Result<Array2<f64>> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    
    let array: Array2<f64> = reader.deserialize_array2_dynamic()?;
    Ok(array)
}

/// Load a CSV file with targets
pub fn load_csv_with_targets<P: AsRef<Path>>(path: P) -> Result<(Array2<f64>, Array1<usize>)> {
    let file = File::open(path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    
    let array: Array2<f64> = reader.deserialize_array2_dynamic()?;
    
    // Assume the last column is the target
    let n_rows = array.nrows();
    let n_cols = array.ncols();
    
    let features = array.slice(s![.., 0..n_cols-1]).to_owned();
    let targets = array.column(n_cols - 1).mapv(|x| x as usize).to_owned();
    
    Ok((features, targets))
}

/// Split a dataset into training and testing sets
pub fn train_test_split<T: Clone, U: Clone>(
    dataset: Dataset<T, U>,
    test_size: f64,
    seed: u64,
) -> Result<(Dataset<T, U>, Dataset<T, U>)> {
    let n_samples = dataset.nsamples();
    let n_test = (n_samples as f64 * test_size) as usize;
    let n_train = n_samples - n_test;
    
    let mut rng = StdRng::seed_from_u64(seed);
    let indices = ndarray::Array::random_using(n_samples, Uniform::new(0., 1.), &mut rng)
        .argsort();
    
    let train_indices = indices.slice(s![0..n_train]).to_vec();
    let test_indices = indices.slice(s![n_train..]).to_vec();
    
    let train_records = dataset.records.select(Axis(0), &train_indices);
    let train_targets = dataset.targets.select(Axis(0), &train_indices);
    let test_records = dataset.records.select(Axis(0), &test_indices);
    let test_targets = dataset.targets.select(Axis(0), &test_indices);
    
    let train_dataset = Dataset::new(train_records, train_targets);
    let test_dataset = Dataset::new(test_records, test_targets);
    
    Ok((train_dataset, test_dataset))
}

/// Generate a synthetic classification dataset
pub fn generate_classification(
    n_samples: usize,
    n_features: usize,
    n_classes: usize,
    seed: u64,
) -> Result<Dataset<f64, usize>> {
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Generate features
    let features = Array2::random_using((n_samples, n_features), StandardNormal, &mut rng);
    
    // Generate class centers
    let centers = Array2::random_using((n_classes, n_features), StandardNormal, &mut rng);
    
    // Assign samples to classes
    let mut targets = Array1::zeros(n_samples);
    for i in 0..n_samples {
        let mut min_dist = f64::INFINITY;
        let mut min_class = 0;
        
        for c in 0..n_classes {
            let dist = (0..n_features)
                .map(|j| (features[[i, j]] - centers[[c, j]]).powi(2))
                .sum::<f64>()
                .sqrt();
            
            if dist < min_dist {
                min_dist = dist;
                min_class = c;
            }
        }
        
        targets[i] = min_class as f64;
    }
    
    Ok(Dataset::new(features, targets.mapv(|x| x as usize)))
}

/// Generate a synthetic regression dataset
pub fn generate_regression(
    n_samples: usize,
    n_features: usize,
    seed: u64,
) -> Result<Dataset<f64, f64>> {
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Generate features
    let features = Array2::random_using((n_samples, n_features), StandardNormal, &mut rng);
    
    // Generate coefficients
    let coefficients = Array1::random_using(n_features, StandardNormal, &mut rng);
    
    // Generate targets with noise
    let noise = Array1::random_using(n_samples, Normal::new(0.0, 0.1).unwrap(), &mut rng);
    let targets = features.dot(&coefficients) + noise;
    
    Ok(Dataset::new(features, targets))
}

/// Generate a synthetic clustering dataset
pub fn generate_clustering(
    n_samples: usize,
    n_features: usize,
    n_clusters: usize,
    seed: u64,
) -> Result<Dataset<f64, usize>> {
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Generate cluster centers
    let centers = Array2::random_using((n_clusters, n_features), StandardNormal, &mut rng);
    
    // Generate samples around centers
    let samples_per_cluster = n_samples / n_clusters;
    let mut features = Array2::zeros((n_samples, n_features));
    let mut targets = Array1::zeros(n_samples);
    
    for c in 0..n_clusters {
        let start_idx = c * samples_per_cluster;
        let end_idx = if c == n_clusters - 1 {
            n_samples
        } else {
            (c + 1) * samples_per_cluster
        };
        
        for i in start_idx..end_idx {
            for j in 0..n_features {
                let noise = rng.sample(StandardNormal) * 0.1;
                features[[i, j]] = centers[[c, j]] + noise;
            }
            targets[i] = c as f64;
        }
    }
    
    Ok(Dataset::new(features, targets.mapv(|x| x as usize)))
}

/// Save a dataset to a CSV file
pub fn save_dataset<T: linfa::Float, U: std::fmt::Display>(
    dataset: Dataset<T, U>,
    path: &Path,
) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    
    // Write headers
    let mut headers = Vec::new();
    for i in 0..dataset.nfeatures() {
        headers.push(format!("feature_{}", i));
    }
    headers.push("target".to_string());
    wtr.write_record(&headers)?;
    
    // Write data
    for i in 0..dataset.nsamples() {
        let mut row = Vec::new();
        for j in 0..dataset.nfeatures() {
            row.push(dataset.records[[i, j]].to_string());
        }
        row.push(dataset.targets[i].to_string());
        wtr.write_record(&row)?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Save clustering results to a CSV file
pub fn save_clustering_results(
    data: &Array2<f64>,
    clusters: &Array1<usize>,
    path: &Path,
) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    
    // Write headers
    let mut headers = Vec::new();
    for i in 0..data.ncols() {
        headers.push(format!("feature_{}", i));
    }
    headers.push("cluster".to_string());
    wtr.write_record(&headers)?;
    
    // Write data
    for i in 0..data.nrows() {
        let mut row = Vec::new();
        for j in 0..data.ncols() {
            row.push(data[[i, j]].to_string());
        }
        row.push(clusters[i].to_string());
        wtr.write_record(&row)?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Save reduced data to a CSV file
pub fn save_reduced_data<T: std::fmt::Display>(
    data: &Array2<f64>,
    targets: &Array1<T>,
    path: &Path,
) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    
    // Write headers
    let mut headers = Vec::new();
    for i in 0..data.ncols() {
        headers.push(format!("component_{}", i));
    }
    headers.push("target".to_string());
    wtr.write_record(&headers)?;
    
    // Write data
    for i in 0..data.nrows() {
        let mut row = Vec::new();
        for j in 0..data.ncols() {
            row.push(data[[i, j]].to_string());
        }
        row.push(targets[i].to_string());
        wtr.write_record(&row)?;
    }
    
    wtr.flush()?;
    Ok(())
}
