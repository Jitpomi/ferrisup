use anyhow::{anyhow, Result};
use linfa::dataset::Dataset;
use linfa::prelude::*;
use ndarray::{Array1, Array2, ArrayBase, ArrayView1, ArrayView2, Data, Dim, s};
use ndarray_rand::rand::prelude::StdRng;
use ndarray_rand::rand::SeedableRng;
use ndarray_rand::rand_distr::{Normal, Uniform};
use ndarray_rand::RandomExt;
use rand_xoshiro::Xoshiro256Plus;
use std::fs::File;
use std::path::Path;
use csv::ReaderBuilder;
use linfa_datasets::{iris, diabetes, winequality};
use ndarray_csv::Array2Reader;
use rand::prelude::SliceRandom;
use rand::Rng;
use rand::distributions::Distribution;

/// Load the Iris dataset
pub fn load_iris() -> Result<Dataset<f64, usize, Ix1>> {
    // In Linfa 0.7.1, iris() returns a Dataset directly, not a Result
    let iris_data = iris();
    Ok(iris_data)
}

/// Load the Iris dataset features only
pub fn load_iris_features() -> Result<Array2<f64>> {
    let dataset = load_iris()?;
    Ok(dataset.records().to_owned())
}

/// Load the Iris dataset with targets
pub fn load_iris_with_targets() -> Result<(Array2<f64>, Array1<usize>)> {
    let dataset = load_iris()?;
    Ok((dataset.records().to_owned(), dataset.targets().to_owned()))
}

/// Load the Diabetes dataset
pub fn load_diabetes() -> Result<Dataset<f64, usize, Ix1>> {
    // In Linfa 0.7.1, diabetes() returns a Dataset directly
    let diabetes_data = diabetes();
    let targets = diabetes_data.targets().mapv(|x| x as usize);
    Ok(Dataset::from(diabetes_data.records().to_owned()).with_targets(targets))
}

/// Load the Wine Quality dataset
pub fn load_winequality() -> Result<Dataset<f64, usize, Ix1>> {
    // In Linfa 0.7.1, winequality() returns a Dataset directly
    let wine_data = winequality();
    Ok(Dataset::from(wine_data.records().to_owned()).with_targets(wine_data.targets().to_owned()))
}

/// Load the Wine Quality dataset as a classification problem
pub fn load_winequality_classification() -> Result<Dataset<f64, usize, Ix1>> {
    let dataset = load_winequality()?;
    let targets = dataset.targets().mapv(|x| x );
    Ok(Dataset::from(dataset.records().to_owned()).with_targets(targets))
}

/// Load the Wine Quality dataset features only
pub fn load_winequality_features() -> Result<Array2<f64>> {
    let dataset = load_winequality()?;
    Ok(dataset.records().to_owned())
}

/// Load the Wine Quality dataset with targets
pub fn load_winequality_with_targets() -> Result<(Array2<f64>, Array1<usize>)> {
    let dataset = load_winequality_classification()?;
    Ok((dataset.records().to_owned(), dataset.targets().to_owned()))
}

/// Load a CSV file as a regression dataset
pub fn load_csv<P: AsRef<Path>>(path: P, target_column: &str) -> Result<Dataset<f64, usize, Ix1>> {
    // Open the file once and read the headers
    let file = File::open(path.as_ref())?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    
    let headers = reader.headers()?.clone();
    let target_idx = headers.iter().position(|h| h == target_column)
        .ok_or_else(|| anyhow::anyhow!("Target column '{}' not found", target_column))?;
    
    // Open the file again to read the data
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
    
    // Convert targets to usize for classification
    let targets_usize = targets.mapv(|x| x as usize);
    
    // Use Dataset::from().with_targets() pattern for Linfa 0.7.1 compatibility
    // but maintain the Dataset<f64, usize, Ix1> return type
    Ok(Dataset::from(features).with_targets(targets_usize))
}

/// Load a CSV file as a classification dataset
pub fn load_csv_classification<P: AsRef<Path>>(path: P, target_column: &str) -> Result<Dataset<f64, usize, Ix1>> {
    let dataset = load_csv(path, target_column)?;
    // Already converted to usize in load_csv
    Ok(dataset)
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
    
    let n_cols = array.ncols();
    
    // Assume the last column is the target
    let features = array.slice(s![.., 0..n_cols-1]).to_owned();
    let targets = array.slice(s![.., n_cols-1]).mapv(|x| x as usize).to_owned();
    
    Ok((features, targets))
}

/// Load a custom dataset from a CSV file
pub fn load_custom_dataset(
    file_path: &PathBuf,
    target_column: &str,
    is_classification: bool,
) -> Result<(Array2<f64>, Array1<f64>)> {
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    
    // Read the headers to find the target column index
    let headers = reader.headers()?;
    let target_idx = headers
        .iter()
        .position(|h| h == target_column)
        .ok_or_else(|| anyhow!("Target column '{}' not found in dataset", target_column))?;
    
    // Read the data into an Array2
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    
    let data = reader.deserialize_array2_dynamic()?;
    
    // Split into features and targets
    let n_cols = data.ncols();
    let mut features = Array2::zeros((data.nrows(), n_cols - 1));
    let mut targets = Array1::zeros(data.nrows());
    
    for i in 0..data.nrows() {
        let mut feat_idx = 0;
        for j in 0..n_cols {
            if j == target_idx {
                targets[i] = data[[i, j]];
            } else {
                features[[i, feat_idx]] = data[[i, j]];
                feat_idx += 1;
            }
        }
    }
    
    // For classification tasks, ensure targets are 0/1 for binary classification
    if is_classification {
        let unique_targets: Vec<f64> = targets.iter().copied().collect::<std::collections::HashSet<f64>>().into_iter().collect();
        if unique_targets.len() == 2 {
            // Binary classification - convert to 0/1
            let min_target = unique_targets.iter().copied().fold(f64::INFINITY, f64::min);
            let max_target = unique_targets.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            
            for i in 0..targets.len() {
                if targets[i] == min_target {
                    targets[i] = 0.0;
                } else if targets[i] == max_target {
                    targets[i] = 1.0;
                }
            }
        }
    }
    
    Ok((features, targets))
}

/// Split a dataset into training and testing sets
pub fn split_dataset<T>(
    features: Array2<f64>,
    targets: Array1<T>,
    test_size: f64,
    seed: u64,
) -> Result<(Array2<f64>, Array1<T>, Array2<f64>, Array1<T>)>
where
    T: Clone,
{
    let n_samples = features.nrows();
    let n_test = (n_samples as f64 * test_size).round() as usize;
    
    if n_test == 0 || n_test >= n_samples {
        return Err(anyhow!("Invalid test size: {}", test_size));
    }
    
    let mut rng = StdRng::seed_from_u64(seed);
    let mut indices: Vec<usize> = (0..n_samples).collect();
    indices.shuffle(&mut rng);
    
    let test_indices: Vec<usize> = indices.iter().take(n_test).cloned().collect();
    let train_indices: Vec<usize> = indices.iter().skip(n_test).cloned().collect();
    
    // Create empty arrays for train and test data
    let mut train_features = Array2::zeros((n_samples - n_test, features.ncols()));
    let mut test_features = Array2::zeros((n_test, features.ncols()));
    
    // Create vectors to hold target values
    let mut train_targets_vec = Vec::with_capacity(n_samples - n_test);
    let mut test_targets_vec = Vec::with_capacity(n_test);
    
    // Fill train data
    for (i, &idx) in train_indices.iter().enumerate() {
        for j in 0..features.ncols() {
            train_features[[i, j]] = features[[idx, j]];
        }
        train_targets_vec.push(targets[idx].clone());
    }
    
    // Fill test data
    for (i, &idx) in test_indices.iter().enumerate() {
        for j in 0..features.ncols() {
            test_features[[i, j]] = features[[idx, j]];
        }
        test_targets_vec.push(targets[idx].clone());
    }
    
    // Convert target vectors to arrays
    let train_targets = Array1::from(train_targets_vec);
    let test_targets = Array1::from(test_targets_vec);
    
    Ok((train_features, train_targets, test_features, test_targets))
}

/// Split a dataset into training and testing sets
pub fn train_test_split<U: Clone>(
    dataset: &Dataset<f64, U>,
    test_size: f64,
    seed: u64,
) -> Result<(Dataset<f64, U>, Dataset<f64, U>)> {
    let features = dataset.records().to_owned();
    let targets = dataset.targets().to_owned();
    
    let n_samples = features.nrows();
    let n_test = (n_samples as f64 * test_size).round() as usize;
    
    if n_test == 0 || n_test >= n_samples {
        return Err(anyhow!("Invalid test size: {}", test_size));
    }
    
    let (train_features, train_targets, test_features, test_targets) = 
        split_dataset(features, targets, test_size, seed)?;
    
    let train_dataset = Dataset::new(train_features, train_targets);
    let test_dataset = Dataset::new(test_features, test_targets);
    
    Ok((train_dataset, test_dataset))
}

/// Split features and targets into training and testing sets
pub fn train_test_split_arrays<T: Float, U: Clone>(
    features: &ArrayView2<'_, T>,
    targets: &ArrayView1<'_, U>,
    test_size: f64,
    seed: u64,
) -> Result<(Array2<T>, Array2<T>, Array1<U>, Array1<U>)> {
    if test_size <= 0.0 || test_size >= 1.0 {
        return Err(anyhow::anyhow!("test_size must be between 0 and 1"));
    }
    
    let n_samples = features.nrows();
    let n_test = (n_samples as f64 * test_size).round() as usize;
    
    if n_test == 0 || n_test >= n_samples {
        return Err(anyhow::anyhow!("test_size results in empty train or test set"));
    }
    
    let mut rng = StdRng::seed_from_u64(seed);
    let mut indices: Vec<usize> = (0..n_samples).collect();
    indices.shuffle(&mut rng);
    
    let test_indices: Vec<usize> = indices.iter().take(n_test).cloned().collect();
    let train_indices: Vec<usize> = indices.iter().skip(n_test).cloned().collect();
    
    let train_features = features.select(Axis(0), &train_indices).to_owned();
    let train_targets = targets.select(Axis(0), &train_indices).to_owned();
    let test_features = features.select(Axis(0), &test_indices).to_owned();
    let test_targets = targets.select(Axis(0), &test_indices).to_owned();
    
    Ok((train_features, test_features, train_targets, test_targets))
}

/// Generate a synthetic classification dataset
pub fn generate_classification(
    n_samples: usize,
    n_features: usize,
    n_classes: usize,
    noise: f64,
) -> Result<(Array2<f64>, Array1<usize>)> {
    // Validate parameters
    if n_samples < 1 {
        return Err(anyhow!("Number of samples must be at least 1"));
    }
    if n_features < 1 {
        return Err(anyhow!("Number of features must be at least 1"));
    }
    if n_classes < 2 {
        return Err(anyhow!("Number of classes must be at least 2"));
    }
    
    // Initialize random number generator
    let mut rng = StdRng::seed_from_u64(42);
    
    // Generate features
    let features = Array2::random_using((n_samples, n_features), StandardNormal, &mut rng);
    
    // Generate targets
    let mut targets = Array1::zeros(n_samples);
    for i in 0..n_samples {
        // Simple classification rule: assign class based on the sum of features
        let sum: f64 = features.row(i).sum();
        let class = (sum.abs() * n_classes as f64 / 5.0).floor() as usize % n_classes;
        targets[i] = class;
    }
    
    // Add noise if requested
    if noise > 0.0 {
        let noise_threshold = noise.min(1.0);
        for i in 0..n_samples {
            if rng.gen::<f64>() < noise_threshold {
                // Randomly reassign class
                targets[i] = rng.gen_range(0..n_classes);
            }
        }
    }
    
    Ok((features, targets))
}

/// Generate a synthetic regression dataset
pub fn generate_regression(
    n_samples: usize,
    n_features: usize,
    seed: u64,
    noise: f64,
) -> Result<(Array2<f64>, Array1<f64>)> {
    // Validate parameters
    if n_samples < 1 {
        return Err(anyhow!("Number of samples must be at least 1"));
    }
    if n_features < 1 {
        return Err(anyhow!("Number of features must be at least 1"));
    }
    
    // Initialize random number generator
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Generate features
    let features = Array2::random_using((n_samples, n_features), StandardNormal, &mut rng);
    
    // Generate coefficients
    let coefficients = Array1::random_using(n_features, StandardNormal, &mut rng);
    
    // Generate targets
    let mut targets = Array1::zeros(n_samples);
    for i in 0..n_samples {
        let mut target = 0.0;
        for j in 0..n_features {
            target += features[[i, j]] * coefficients[j];
        }
        targets[i] = target;
    }
    
    // Add noise if requested
    if noise > 0.0 {
        let noise_dist = Normal::new(0.0, noise).unwrap();
        for i in 0..n_samples {
            targets[i] += noise_dist.sample(&mut rng);
        }
    }
    
    Ok((features, targets))
}

/// Generate a synthetic clustering dataset
pub fn generate_clustering(
    n_samples: usize,
    n_features: usize,
    n_clusters: usize,
    seed: u64,
) -> Result<(Array2<f64>, Array1<usize>)> {
    // Validate parameters
    if n_samples < 1 {
        return Err(anyhow!("Number of samples must be at least 1"));
    }
    if n_features < 1 {
        return Err(anyhow!("Number of features must be at least 1"));
    }
    if n_clusters < 2 {
        return Err(anyhow!("Number of clusters must be at least 2"));
    }
    
    // Initialize random number generator
    let mut rng = StdRng::seed_from_u64(seed);
    
    // Generate cluster centers
    let mut centers = Array2::zeros((n_clusters, n_features));
    for i in 0..n_clusters {
        for j in 0..n_features {
            centers[[i, j]] = rng.gen_range(-10.0..10.0);
        }
    }
    
    // Calculate samples per cluster
    let base_samples_per_cluster = n_samples / n_clusters;
    let remainder = n_samples % n_clusters;
    
    // Generate features and targets
    let mut features = Array2::zeros((n_samples, n_features));
    let mut targets = Array1::zeros(n_samples);
    
    let mut sample_idx = 0;
    for i in 0..n_clusters {
        let cluster_samples = if i < remainder {
            base_samples_per_cluster + 1
        } else {
            base_samples_per_cluster
        };
        
        for _ in 0..cluster_samples {
            if sample_idx >= n_samples {
                break;
            }
            
            // Generate sample around cluster center
            for j in 0..n_features {
                let noise = Normal::new(0.0, 1.0).unwrap().sample(&mut rng);
                features[[sample_idx, j]] = centers[[i, j]] + noise;
            }
            
            targets[sample_idx] = i;
            sample_idx += 1;
        }
    }
    
    Ok((features, targets))
}

/// Save a dataset to a CSV file
pub fn save_dataset<T: std::fmt::Display, S1, S2>(
    features: &ArrayBase<S1, Dim<[usize; 2]>>,
    targets: &ArrayBase<S2, Dim<[usize; 1]>>,
    path: &Path,
) -> Result<()>
where
    S1: Data<Elem = f64>,
    S2: Data<Elem = T>,
{
    // Create a CSV writer
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);
    
    // Write header
    let mut header = Vec::new();
    for i in 0..features.ncols() {
        header.push(format!("feature_{}", i));
    }
    header.push("target".to_string());
    writer.write_record(&header)?;
    
    // Write data
    for i in 0..features.nrows() {
        let mut row = Vec::new();
        for j in 0..features.ncols() {
            row.push(format!("{}", features[[i, j]]));
        }
        row.push(format!("{}", targets[i]));
        writer.write_record(&row)?;
    }
    
    writer.flush()?;
    Ok(())
}

/// Save clustering results to a CSV file
pub fn save_clustering_results(
    data: &Array2<f64>,
    clusters: &Array1<usize>,
    output_path: &PathBuf,
) -> Result<()> {
    // Create a new file
    let file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(file);
    
    // Write header
    let mut header = Vec::new();
    for i in 0..data.ncols() {
        header.push(format!("feature_{}", i));
    }
    header.push("cluster".to_string());
    wtr.write_record(&header)?;
    
    // Write data
    for i in 0..data.nrows() {
        let mut row = Vec::new();
        for j in 0..data.ncols() {
            row.push(format!("{}", data[[i, j]]));
        }
        row.push(format!("{}", clusters[i]));
        wtr.write_record(&row)?;
    }
    
    wtr.flush()?;
    Ok(())
}

/// Save reduced data to a CSV file
pub fn save_reduced_data(
    data: &Array2<f64>,
    targets: &Array1<f64>,
    output_path: &PathBuf,
) -> Result<()> {
    // Create a new file
    let file = File::create(output_path)?;
    let mut wtr = csv::Writer::from_writer(file);
    
    // Write header
    let mut header = Vec::new();
    for i in 0..data.ncols() {
        header.push(format!("component_{}", i));
    }
    header.push("target".to_string());
    wtr.write_record(&header)?;
    
    // Write data
    for i in 0..data.nrows() {
        let mut row = Vec::new();
        for j in 0..data.ncols() {
            row.push(format!("{}", data[[i, j]]));
        }
        row.push(format!("{}", targets[i]));
        wtr.write_record(&row)?;
    }
    
    wtr.flush()?;
    Ok(())
}
