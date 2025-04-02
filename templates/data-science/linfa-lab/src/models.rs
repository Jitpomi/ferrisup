use anyhow::{Result, anyhow};
use linfa::prelude::*;
use linfa::traits::{Fit, Predict};
use linfa_clustering::{KMeans, KMeansParams, DbscanParams, GaussianMixtureParams};
use linfa_linear::{LinearRegression, ElasticNet};
use linfa_logistic::LogisticRegression;
use linfa_reduction::{Pca, PcaParams, Tsne, TsneParams};
use linfa_svm::{Svm, SvmParams};
use linfa_trees::{DecisionTree, RandomForest};
use linfa_nn::KNNRegressor;
use ndarray::{Array1, Array2, ArrayBase, Axis, Data, Ix2};
use ndarray_rand::rand::SeedableRng;
use rand::rngs::StdRng;
use std::marker::PhantomData;

/// Train a regression model
pub fn train_regression_model<'a>(
    model_name: &str,
    dataset: Dataset<f64, f64>,
) -> Result<Box<dyn Predict<Array1<f64>, Output = Array1<f64>> + 'a>> {
    match model_name {
        "linear" => {
            let model = LinearRegression::default()
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "ridge" => {
            let model = LinearRegression::default()
                .alpha(0.1)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "lasso" => {
            let model = ElasticNet::params()
                .alpha(0.1)
                .l1_ratio(1.0)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "elasticnet" => {
            let model = ElasticNet::params()
                .alpha(0.1)
                .l1_ratio(0.5)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "decision_tree" => {
            let model = DecisionTree::params()
                .max_depth(5)
                .min_samples_split(5)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "random_forest" => {
            let model = RandomForest::params()
                .max_depth(5)
                .n_trees(100)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "knn" => {
            let model = KNNRegressor::params()
                .k(5)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        _ => Err(anyhow!("Unknown regression model: {}", model_name)),
    }
}

/// Train a classification model
pub fn train_classification_model<'a>(
    model_name: &str,
    dataset: Dataset<f64, usize>,
) -> Result<Box<dyn Predict<Array1<usize>, Output = Array1<usize>> + 'a>> {
    match model_name {
        "logistic" => {
            let model = LogisticRegression::default()
                .max_iterations(100)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "decision_tree" => {
            let model = DecisionTree::params()
                .max_depth(5)
                .min_samples_split(5)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "random_forest" => {
            let model = RandomForest::params()
                .max_depth(5)
                .n_trees(100)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "svm" => {
            let model = Svm::<_, _, _>::params()
                .gaussian_kernel(0.1)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        "knn" => {
            let model = linfa_nn::KNNClassifier::params()
                .k(5)
                .fit(&dataset)?;
            Ok(Box::new(model))
        },
        _ => Err(anyhow!("Unknown classification model: {}", model_name)),
    }
}

/// Perform clustering on a dataset
pub fn perform_clustering(
    algorithm: &str,
    data: Array2<f64>,
    n_clusters: usize,
    seed: u64,
) -> Result<(Array1<usize>, Array2<f64>)> {
    let mut rng = StdRng::seed_from_u64(seed);
    
    match algorithm {
        "kmeans" => {
            let model = KMeans::params_with_rng(n_clusters, rng)
                .max_n_iterations(100)
                .fit(&data)?;
            
            let clusters = model.predict(&data);
            let centroids = model.centroids().to_owned();
            
            Ok((clusters, centroids))
        },
        "dbscan" => {
            let model = DbscanParams::new(0.3)
                .min_points(5)
                .fit(&data)?;
            
            let clusters = model.predict(&data);
            
            // DBSCAN doesn't have centroids, so we compute them manually
            let n_found_clusters = clusters.iter().max().unwrap_or(&0) + 1;
            let mut centroids = Array2::zeros((n_found_clusters, data.ncols()));
            let mut counts = Array1::zeros(n_found_clusters);
            
            for (i, &cluster) in clusters.iter().enumerate() {
                if cluster != usize::MAX { // Skip noise points (marked as usize::MAX)
                    for j in 0..data.ncols() {
                        centroids[[cluster, j]] += data[[i, j]];
                    }
                    counts[cluster] += 1;
                }
            }
            
            for i in 0..n_found_clusters {
                if counts[i] > 0 {
                    for j in 0..data.ncols() {
                        centroids[[i, j]] /= counts[i] as f64;
                    }
                }
            }
            
            Ok((clusters, centroids))
        },
        "gaussian_mixture" => {
            let model = GaussianMixtureParams::new(n_clusters)
                .with_rng(rng)
                .with_max_n_iterations(100)
                .fit(&data)?;
            
            let clusters = model.predict(&data);
            let centroids = model.means().to_owned();
            
            Ok((clusters, centroids))
        },
        _ => Err(anyhow!("Unknown clustering algorithm: {}", algorithm)),
    }
}

/// Perform dimensionality reduction on a dataset
pub fn perform_reduction(
    algorithm: &str,
    data: Array2<f64>,
    n_components: usize,
    seed: u64,
) -> Result<Array2<f64>> {
    let mut rng = StdRng::seed_from_u64(seed);
    
    match algorithm {
        "pca" => {
            let model = PcaParams::new(n_components)
                .fit(&data)?;
            
            let transformed = model.transform(&data);
            Ok(transformed)
        },
        "tsne" => {
            let model = TsneParams::new(n_components)
                .with_rng(rng)
                .with_max_iter(1000)
                .fit(&data)?;
            
            let transformed = model.embedding();
            Ok(transformed.to_owned())
        },
        _ => Err(anyhow!("Unknown dimensionality reduction algorithm: {}", algorithm)),
    }
}
