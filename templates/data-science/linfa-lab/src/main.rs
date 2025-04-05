mod datasets;
mod models;
mod evaluation;

use anyhow::Result;
use clap::{Parser, Subcommand};
use linfa::dataset::Records;
use std::path::PathBuf;

/// A machine learning application using Linfa for classical ML algorithms
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Train and evaluate a regression model
    Regression {
        /// Dataset to use (diabetes, winequality, or custom)
        #[arg(short, long, default_value = "diabetes")]
        dataset: String,
        
        /// Path to custom dataset (CSV)
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Target column name (for custom datasets)
        #[arg(short, long)]
        target: Option<String>,
        
        /// Model to use (linear, ridge, lasso, elasticnet, decision_tree, random_forest)
        #[arg(short, long, default_value = "linear")]
        model: String,
        
        /// Test size ratio (0.0-1.0)
        #[arg(short, long, default_value_t = 0.2)]
        test_size: f64,
        
        /// Random seed
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        
        /// Path to save the model
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Train and evaluate a classification model
    Classification {
        /// Dataset to use (iris, winequality, or custom)
        #[arg(short, long, default_value = "iris")]
        dataset: String,
        
        /// Path to custom dataset (CSV)
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Target column name (for custom datasets)
        #[arg(short, long)]
        target: Option<String>,
        
        /// Model to use (logistic, decision_tree, random_forest, svm, knn)
        #[arg(short, long, default_value = "logistic")]
        model: String,
        
        /// Test size ratio (0.0-1.0)
        #[arg(short, long, default_value_t = 0.2)]
        test_size: f64,
        
        /// Random seed
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        
        /// Path to save the model
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Perform clustering on a dataset
    Clustering {
        /// Dataset to use (iris, winequality, or custom)
        #[arg(short, long, default_value = "iris")]
        dataset: String,
        
        /// Path to custom dataset (CSV)
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Algorithm to use (kmeans, dbscan, gaussian_mixture)
        #[arg(short, long, default_value = "kmeans")]
        algorithm: String,
        
        /// Number of clusters (for k-means)
        #[arg(short, long, default_value_t = 3)]
        n_clusters: usize,
        
        /// Random seed
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        
        /// Path to save the results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Perform dimensionality reduction on a dataset
    Reduction {
        /// Dataset to use (iris, winequality, or custom)
        #[arg(short, long, default_value = "iris")]
        dataset: String,
        
        /// Path to custom dataset (CSV)
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Algorithm to use (pca, tsne)
        #[arg(short, long, default_value = "pca")]
        algorithm: String,
        
        /// Number of components
        #[arg(short, long, default_value_t = 2)]
        n_components: usize,
        
        /// Random seed
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        
        /// Path to save the results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Generate a synthetic dataset
    Generate {
        /// Type of dataset (classification, regression, clustering)
        #[arg(short, long, default_value = "classification")]
        dataset_type: String,
        
        /// Number of samples
        #[arg(short, long, default_value_t = 1000)]
        n_samples: usize,
        
        /// Number of features
        #[arg(short, long, default_value_t = 10)]
        n_features: usize,
        
        /// Number of classes (for classification)
        #[arg(short, long, default_value_t = 2)]
        n_classes: usize,
        
        /// Random seed
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        
        /// Path to save the dataset
        #[arg(short, long, default_value = "synthetic_data.csv")]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Regression {
            dataset,
            file,
            target,
            model,
            test_size,
            seed,
            output,
        } => {
            println!("🧮 Regression Analysis");
            println!("Dataset: {}", dataset);
            println!("Model: {}", model);
            
            // Load dataset
            let (data, target_name) = match dataset.as_str() {
                "diabetes" => {
                    println!("Loading diabetes dataset...");
                    let dataset = datasets::load_diabetes()?;
                    (dataset, "target".to_string())
                },
                "winequality" => {
                    println!("Loading wine quality dataset...");
                    let dataset = datasets::load_winequality()?;
                    (dataset, "quality".to_string())
                },
                "custom" => {
                    if let Some(file_path) = file {
                        println!("Loading custom dataset from {}...", file_path.display());
                        let target_col = target.clone().unwrap_or_else(|| {
                            println!("No target column specified, using 'target'");
                            "target".to_string()
                        });
                        let dataset = datasets::load_csv(file_path, &target_col)?;
                        (dataset, target_col)
                    } else {
                        println!("No file specified for custom dataset, using diabetes dataset");
                        let dataset = datasets::load_diabetes()?;
                        (dataset, "target".to_string())
                    }
                },
                _ => {
                    println!("Unknown dataset '{}', using diabetes dataset", dataset);
                    let dataset = datasets::load_diabetes()?;
                    (dataset, "target".to_string())
                }
            };
            
            // Split dataset
            let (train, test) = datasets::train_test_split(data, *test_size, *seed)?;
            println!("Train set size: {}", train.nsamples());
            println!("Test set size: {}", test.nsamples());
            
            // Train model
            println!("Training {} regression model...", model);
            let trained_model = models::train_regression_model(model, train.clone())?;
            
            // Evaluate model
            println!("Evaluating model...");
            let metrics = evaluation::evaluate_regression(trained_model, test.clone())?;
            println!("Results:");
            println!("  MSE: {:.4}", metrics.mse);
            println!("  RMSE: {:.4}", metrics.rmse);
            println!("  MAE: {:.4}", metrics.mae);
            println!("  R²: {:.4}", metrics.r2);
            
            // Save model if requested
            if let Some(output_path) = output {
                println!("Saving model to {}...", output_path.display());
                // Note: Linfa doesn't have a standard way to save models yet
                println!("Model saving not implemented yet");
            }
            
            // Generate a simple plot
            println!("Generating prediction vs actual plot...");
            let plot_path = "regression_results.png";
            evaluation::plot_regression_results(test, &metrics, plot_path)?;
            println!("Plot saved to {}", plot_path);
        },
        
        Commands::Classification {
            dataset,
            file,
            target,
            model,
            test_size,
            seed,
            output,
        } => {
            println!("🔍 Classification Analysis");
            println!("Dataset: {}", dataset);
            println!("Model: {}", model);
            
            // Load dataset
            let (data, target_name) = match dataset.as_str() {
                "iris" => {
                    println!("Loading iris dataset...");
                    let dataset = datasets::load_iris()?;
                    (dataset, "species".to_string())
                },
                "winequality" => {
                    println!("Loading wine quality dataset (as classification)...");
                    let dataset = datasets::load_winequality_classification()?;
                    (dataset, "quality".to_string())
                },
                "custom" => {
                    if let Some(file_path) = file {
                        println!("Loading custom dataset from {}...", file_path.display());
                        let target_col = target.clone().unwrap_or_else(|| {
                            println!("No target column specified, using 'target'");
                            "target".to_string()
                        });
                        let dataset = datasets::load_csv_classification(file_path, &target_col)?;
                        (dataset, target_col)
                    } else {
                        println!("No file specified for custom dataset, using iris dataset");
                        let dataset = datasets::load_iris()?;
                        (dataset, "species".to_string())
                    }
                },
                _ => {
                    println!("Unknown dataset '{}', using iris dataset", dataset);
                    let dataset = datasets::load_iris()?;
                    (dataset, "species".to_string())
                }
            };
            
            // Split dataset
            let (train, test) = datasets::train_test_split(data, *test_size, *seed)?;
            println!("Train set size: {}", train.nsamples());
            println!("Test set size: {}", test.nsamples());
            
            // Train model
            println!("Training {} classification model...", model);
            let trained_model = models::train_classification_model(model, train.clone())?;
            
            // Evaluate model
            println!("Evaluating model...");
            let metrics = evaluation::evaluate_classification(trained_model, test.clone())?;
            println!("Results:");
            println!("  Accuracy: {:.4}", metrics.accuracy);
            println!("  Precision: {:.4}", metrics.precision);
            println!("  Recall: {:.4}", metrics.recall);
            println!("  F1 Score: {:.4}", metrics.f1);
            
            // Save model if requested
            if let Some(output_path) = output {
                println!("Saving model to {}...", output_path.display());
                // Note: Linfa doesn't have a standard way to save models yet
                println!("Model saving not implemented yet");
            }
            
            // Generate a simple plot
            println!("Generating confusion matrix plot...");
            let plot_path = "classification_results.png";
            evaluation::plot_confusion_matrix(test, &metrics, plot_path)?;
            println!("Plot saved to {}", plot_path);
        },
        
        Commands::Clustering {
            dataset,
            file,
            algorithm,
            n_clusters,
            seed,
            output,
        } => {
            println!("🔮 Clustering Analysis");
            println!("Dataset: {}", dataset);
            println!("Algorithm: {}", algorithm);
            println!("Number of clusters: {}", n_clusters);
            
            // Load dataset
            let data = match dataset.as_str() {
                "iris" => {
                    println!("Loading iris dataset (features only)...");
                    datasets::load_iris_features()?
                },
                "winequality" => {
                    println!("Loading wine quality dataset (features only)...");
                    datasets::load_winequality_features()?
                },
                "custom" => {
                    if let Some(file_path) = file {
                        println!("Loading custom dataset from {}...", file_path.display());
                        datasets::load_csv_features(file_path)?
                    } else {
                        println!("No file specified for custom dataset, using iris dataset");
                        datasets::load_iris_features()?
                    }
                },
                _ => {
                    println!("Unknown dataset '{}', using iris dataset", dataset);
                    datasets::load_iris_features()?
                }
            };
            
            // Perform clustering
            println!("Performing {} clustering...", algorithm);
            let (clusters, centroids) = models::perform_clustering(
                algorithm, 
                data.clone(), 
                *n_clusters, 
                *seed
            )?;
            
            // Evaluate clustering
            println!("Evaluating clustering...");
            let metrics = evaluation::evaluate_clustering(&data, &clusters, &centroids)?;
            println!("Results:");
            println!("  Silhouette Score: {:.4}", metrics.silhouette);
            println!("  Inertia: {:.4}", metrics.inertia);
            println!("  Number of clusters: {}", metrics.n_clusters);
            
            // Save results if requested
            if let Some(output_path) = output {
                println!("Saving clustering results to {}...", output_path.display());
                datasets::save_clustering_results(&data, &clusters, output_path)?;
            }
            
            // Generate a simple plot
            println!("Generating clustering plot...");
            let plot_path = "clustering_results.png";
            evaluation::plot_clustering(&data, &clusters, &centroids, plot_path)?;
            println!("Plot saved to {}", plot_path);
        },
        
        Commands::Reduction {
            dataset,
            file,
            algorithm,
            n_components,
            seed,
            output,
        } => {
            println!("📉 Dimensionality Reduction");
            println!("Dataset: {}", dataset);
            println!("Algorithm: {}", algorithm);
            println!("Number of components: {}", n_components);
            
            // Load dataset
            let (data, targets) = match dataset.as_str() {
                "iris" => {
                    println!("Loading iris dataset...");
                    datasets::load_iris_with_targets()?
                },
                "winequality" => {
                    println!("Loading wine quality dataset...");
                    datasets::load_winequality_with_targets()?
                },
                "custom" => {
                    if let Some(file_path) = file {
                        println!("Loading custom dataset from {}...", file_path.display());
                        datasets::load_csv_with_targets(file_path)?
                    } else {
                        println!("No file specified for custom dataset, using iris dataset");
                        datasets::load_iris_with_targets()?
                    }
                },
                _ => {
                    println!("Unknown dataset '{}', using iris dataset", dataset);
                    datasets::load_iris_with_targets()?
                }
            };
            
            // Perform dimensionality reduction
            println!("Performing {} dimensionality reduction...", algorithm);
            let reduced_data = models::perform_reduction(
                algorithm, 
                data.clone(), 
                *n_components, 
                *seed
            )?;
            
            // Save results if requested
            if let Some(output_path) = output {
                println!("Saving reduced data to {}...", output_path.display());
                datasets::save_reduced_data(&reduced_data, &targets, output_path)?;
            }
            
            // Generate a simple plot
            println!("Generating dimensionality reduction plot...");
            let plot_path = "reduction_results.png";
            evaluation::plot_reduction(&reduced_data, &targets, plot_path)?;
            println!("Plot saved to {}", plot_path);
        },
        
        Commands::Generate {
            dataset_type,
            n_samples,
            n_features,
            n_classes,
            seed,
            output,
        } => {
            println!("🔄 Generating Synthetic Dataset");
            println!("Type: {}", dataset_type);
            println!("Samples: {}", n_samples);
            println!("Features: {}", n_features);
            
            match dataset_type.as_str() {
                "classification" => {
                    println!("Classes: {}", n_classes);
                    println!("Generating classification dataset...");
                    let dataset = datasets::generate_classification(*n_samples, *n_features, *n_classes, *seed)?;
                    datasets::save_dataset(dataset, output)?;
                },
                "regression" => {
                    println!("Generating regression dataset...");
                    let dataset = datasets::generate_regression(*n_samples, *n_features, *seed)?;
                    datasets::save_dataset(dataset, output)?;
                },
                "clustering" => {
                    println!("Generating clustering dataset...");
                    let dataset = datasets::generate_clustering(*n_samples, *n_features, *n_classes, *seed)?;
                    datasets::save_dataset(dataset, output)?;
                },
                _ => {
                    println!("Unknown dataset type '{}', using classification", dataset_type);
                    let dataset = datasets::generate_classification(*n_samples, *n_features, *n_classes, *seed)?;
                    datasets::save_dataset(dataset, output)?;
                }
            }
            
            println!("Dataset saved to {}", output.display());
        },
    }

    Ok(())
}
