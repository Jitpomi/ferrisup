// Data Predictor Main Application
// This file contains the CLI interface and main program logic

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::Path;
use std::fs;
use std::io::{self, Write};
use burn::tensor::backend::{AutodiffBackend, Backend};
use burn::optim::{AdamConfig, SGDConfig, RMSpropConfig, GradientsParams};
use burn::record::{Recorder, DefaultRecorder};
use burn::module::Module;
use burn::data::dataloader::DataLoaderBuilder;
use burn::train::{TrainOutput, TrainStep, ValidStep};
use indicatif::{ProgressBar, ProgressStyle};

// Import our modules
mod config;
mod model;
mod data;

// Import types and constants from our modules
use config::{
    BATCH_SIZE, LEARNING_RATE, EPOCHS, DEFAULT_DATA_FILE,
    DEFAULT_MODEL_FILE, DEFAULT_STATS_FILE, EARLY_STOPPING_PATIENCE,
    TEST_SPLIT_RATIO, OPTIMIZER, WEIGHT_DECAY, CLIP_GRADIENT
};
use model::{PredictorModel, PredictorConfig, create_default_model};
use data::{
    DataBatcher, RawDataItem, DataItem, NumericalDataset, DataStats,
    load_csv_dataset, create_sample_dataset, create_housing_dataset
};

// Regression output metrics
struct RegressionOutput<B: Backend> {
    loss: f32,
    mse: f32,
    mae: f32,
    r2: f32,
}

// CLI Arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Train a new prediction model
    Train {
        /// Path to the CSV data file
        #[arg(short, long, default_value = DEFAULT_DATA_FILE)]
        data_file: String,
        
        /// Path to save the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_file: String,
        
        /// Path to save the data statistics
        #[arg(short, long, default_value = DEFAULT_STATS_FILE)]
        stats_file: String,
        
        /// Use sample data instead of loading from disk
        #[arg(short, long, default_value = "false")]
        sample: bool,
        
        /// Use housing price data instead of loading from disk
        #[arg(short, long, default_value = "false")]
        housing: bool,
    },
    
    /// Evaluate model performance on a test dataset
    Evaluate {
        /// Path to the CSV data file
        #[arg(short, long, default_value = DEFAULT_DATA_FILE)]
        data_file: String,
        
        /// Path to the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_file: String,
        
        /// Path to the data statistics
        #[arg(short, long, default_value = DEFAULT_STATS_FILE)]
        stats_file: String,
        
        /// Use sample data instead of loading from disk
        #[arg(short, long, default_value = "false")]
        sample: bool,
        
        /// Use housing price data instead of loading from disk
        #[arg(short, long, default_value = "false")]
        housing: bool,
    },
    
    /// Predict values for new data
    Predict {
        /// Feature values (comma-separated)
        #[arg(short, long)]
        features: String,
        
        /// Path to the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_file: String,
        
        /// Path to the data statistics
        #[arg(short, long, default_value = DEFAULT_STATS_FILE)]
        stats_file: String,
    },
    
    /// Predict values for data in a CSV file
    PredictCsv {
        /// Path to the CSV file
        #[arg(short, long)]
        file: String,
        
        /// Path to the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_file: String,
        
        /// Path to the data statistics
        #[arg(short, long, default_value = DEFAULT_STATS_FILE)]
        stats_file: String,
    },
}

// Define the training step
struct TrainingStep<B: AutodiffBackend> {
    model: PredictorModel<B>,
    optimizer_type: String,
    learning_rate: f32,
    weight_decay: f32,
    clip_gradient: f32,
}

impl<B: AutodiffBackend> TrainStep<DataItem<B>, RegressionOutput<B>> for TrainingStep<B> {
    fn step(&self, item: DataItem<B>) -> TrainOutput<RegressionOutput<B>> {
        // Forward pass
        let predictions = self.model.forward(item.features.clone());
        
        // Calculate MSE loss
        let loss = predictions.sub(&item.targets).pow_scalar(2.0).mean();
        
        // Calculate metrics
        let mse = loss.clone().into_scalar();
        let mae = predictions.sub(&item.targets).abs().mean().into_scalar();
        
        // Calculate R² (coefficient of determination)
        let targets_mean = item.targets.mean();
        let total_sum_squares = item.targets.sub(&targets_mean).pow_scalar(2.0).sum();
        let residual_sum_squares = predictions.sub(&item.targets).pow_scalar(2.0).sum();
        let r2 = (1.0 - (residual_sum_squares / total_sum_squares)).into_scalar();
        
        // Backward pass
        let mut gradients = loss.backward();
        
        // Gradient clipping
        if self.clip_gradient > 0.0 {
            gradients.clip_norm_(self.clip_gradient);
        }
        
        // Create and apply optimizer
        let updated_params = match self.optimizer_type.as_str() {
            "sgd" => {
                let optimizer = SGDConfig::new()
                    .with_learning_rate(self.learning_rate)
                    .with_weight_decay(self.weight_decay)
                    .init();
                optimizer.step(&self.model, &gradients)
            },
            "rmsprop" => {
                let optimizer = RMSpropConfig::new()
                    .with_learning_rate(self.learning_rate)
                    .with_weight_decay(self.weight_decay)
                    .init();
                optimizer.step(&self.model, &gradients)
            },
            _ => { // Default to Adam
                let optimizer = AdamConfig::new()
                    .with_learning_rate(self.learning_rate)
                    .with_weight_decay(self.weight_decay)
                    .init();
                optimizer.step(&self.model, &gradients)
            }
        };
        
        TrainOutput::new(
            self.model.clone_with(updated_params),
            RegressionOutput { loss: mse, mse, mae, r2 },
        )
    }
}

// Define the validation step
struct ValidationStep<B: Backend> {
    model: PredictorModel<B>,
}

impl<B: Backend> ValidStep<DataItem<B>, RegressionOutput<B>> for ValidationStep<B> {
    fn step(&self, item: DataItem<B>) -> RegressionOutput<B> {
        // Forward pass
        let predictions = self.model.forward(item.features.clone());
        
        // Calculate MSE loss
        let mse = predictions.sub(&item.targets).pow_scalar(2.0).mean().into_scalar();
        
        // Calculate MAE
        let mae = predictions.sub(&item.targets).abs().mean().into_scalar();
        
        // Calculate R² (coefficient of determination)
        let targets_mean = item.targets.mean();
        let total_sum_squares = item.targets.sub(&targets_mean).pow_scalar(2.0).sum();
        let residual_sum_squares = predictions.sub(&item.targets).pow_scalar(2.0).sum();
        let r2 = (1.0 - (residual_sum_squares / total_sum_squares)).into_scalar();
        
        RegressionOutput { loss: mse, mse, mae, r2 }
    }
}

// Main function
fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Run the appropriate command
    match cli.command {
        Commands::Train { data_file, model_file, stats_file, sample, housing } => {
            train_model(data_file, model_file, stats_file, sample, housing)?;
        }
        Commands::Evaluate { data_file, model_file, stats_file, sample, housing } => {
            evaluate_model(data_file, model_file, stats_file, sample, housing)?;
        }
        Commands::Predict { features, model_file, stats_file } => {
            predict_values(features, model_file, stats_file)?;
        }
        Commands::PredictCsv { file, model_file, stats_file } => {
            predict_csv(file, model_file, stats_file)?;
        }
    }
    
    Ok(())
}

// Train a new prediction model
fn train_model(data_file: String, model_file: String, stats_file: String, use_sample: bool, use_housing: bool) -> Result<()> {
    // CUSTOMIZE HERE: Modify the training process
    
    println!("Starting training process...");
    
    // Load dataset
    let dataset = if use_sample {
        println!("Using generic sample dataset");
        create_sample_dataset()
    } else if use_housing {
        println!("Using housing price sample dataset");
        create_housing_dataset()
    } else {
        println!("Loading dataset from {}", data_file);
        load_csv_dataset(&data_file)?
    };
    
    // Save statistics
    println!("Saving data statistics to {}", stats_file);
    dataset.stats().save(&stats_file)?;
    
    println!("Dataset loaded with {} examples", dataset.len());
    println!("Features: {} dimensions", dataset.num_features());
    println!("Targets: {} dimensions", dataset.num_targets());
    
    // Split into training and testing sets
    let (train_dataset, test_dataset) = dataset.split_train_test(TEST_SPLIT_RATIO);
    
    println!("Training set: {} examples", train_dataset.len());
    println!("Testing set: {} examples", test_dataset.len());
    
    // Create data loaders
    let batcher = DataBatcher::<burn::backend::NdArray<f32>>::new(BATCH_SIZE);
    
    let train_loader = DataLoaderBuilder::new(batcher.clone())
        .batch_size(BATCH_SIZE)
        .shuffle(true)
        .build(train_dataset);
    
    let test_loader = DataLoaderBuilder::new(batcher)
        .batch_size(BATCH_SIZE)
        .shuffle(false)
        .build(test_dataset);
    
    // Create model
    let model = create_default_model::<burn::backend::NdArray<f32>>(
        dataset.num_features(),
        dataset.num_targets(),
    );
    
    // Create training and validation steps
    let training_step = TrainingStep {
        model: model.clone(),
        optimizer_type: OPTIMIZER.to_string(),
        learning_rate: LEARNING_RATE,
        weight_decay: WEIGHT_DECAY,
        clip_gradient: CLIP_GRADIENT,
    };
    
    let validation_step = ValidationStep { model };
    
    // Create recorder for saving model
    let recorder = DefaultRecorder::new();
    
    // Setup progress bar
    let progress_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap();
    
    // Training loop
    println!("Starting training for {} epochs", EPOCHS);
    
    let mut best_val_loss = f32::MAX;
    let mut patience_counter = 0;
    
    for epoch in 0..EPOCHS {
        println!("Epoch {}/{}", epoch + 1, EPOCHS);
        
        // Training phase
        let mut train_loss_sum = 0.0;
        let mut train_mse_sum = 0.0;
        let mut train_mae_sum = 0.0;
        let mut train_r2_sum = 0.0;
        let train_batches = train_loader.len();
        
        let progress_bar = ProgressBar::new(train_batches as u64);
        progress_bar.set_style(progress_style.clone());
        
        let mut current_model = training_step.model.clone();
        
        for (batch_idx, batch) in train_loader.enumerate() {
            let output = training_step.step(batch);
            current_model = output.model;
            
            train_loss_sum += output.context.loss;
            train_mse_sum += output.context.mse;
            train_mae_sum += output.context.mae;
            train_r2_sum += output.context.r2;
            
            progress_bar.set_position((batch_idx + 1) as u64);
        }
        
        progress_bar.finish_with_message("Training complete");
        
        let train_loss = train_loss_sum / train_batches as f32;
        let train_mse = train_mse_sum / train_batches as f32;
        let train_mae = train_mae_sum / train_batches as f32;
        let train_r2 = train_r2_sum / train_batches as f32;
        
        // Validation phase
        let mut valid_loss_sum = 0.0;
        let mut valid_mse_sum = 0.0;
        let mut valid_mae_sum = 0.0;
        let mut valid_r2_sum = 0.0;
        let valid_batches = test_loader.len();
        
        let progress_bar = ProgressBar::new(valid_batches as u64);
        progress_bar.set_style(progress_style.clone());
        
        let validation_step = ValidationStep { model: current_model.clone() };
        
        for (batch_idx, batch) in test_loader.enumerate() {
            let output = validation_step.step(batch);
            
            valid_loss_sum += output.loss;
            valid_mse_sum += output.mse;
            valid_mae_sum += output.mae;
            valid_r2_sum += output.r2;
            
            progress_bar.set_position((batch_idx + 1) as u64);
        }
        
        progress_bar.finish_with_message("Validation complete");
        
        let valid_loss = valid_loss_sum / valid_batches as f32;
        let valid_mse = valid_mse_sum / valid_batches as f32;
        let valid_mae = valid_mae_sum / valid_batches as f32;
        let valid_r2 = valid_r2_sum / valid_batches as f32;
        
        println!("Train Loss: {:.6}, MSE: {:.6}, MAE: {:.6}, R²: {:.4}", 
                train_loss, train_mse, train_mae, train_r2);
        println!("Valid Loss: {:.6}, MSE: {:.6}, MAE: {:.6}, R²: {:.4}", 
                valid_loss, valid_mse, valid_mae, valid_r2);
        
        // Save best model
        if valid_loss < best_val_loss {
            println!("Validation loss improved from {:.6} to {:.6}, saving model", 
                    best_val_loss, valid_loss);
            
            best_val_loss = valid_loss;
            patience_counter = 0;
            
            // Save model
            recorder.record(current_model.clone(), &model_file)?;
        } else {
            patience_counter += 1;
            println!("Validation loss did not improve. Best: {:.6}", best_val_loss);
            
            // Early stopping
            if patience_counter >= EARLY_STOPPING_PATIENCE {
                println!("Early stopping triggered after {} epochs without improvement", 
                        patience_counter);
                break;
            }
        }
    }
    
    println!("Training complete. Best validation loss: {:.6}", best_val_loss);
    println!("Model saved to {}", model_file);
    println!("Statistics saved to {}", stats_file);
    
    Ok(())
}

// Evaluate model performance on a test dataset
fn evaluate_model(data_file: String, model_file: String, stats_file: String, use_sample: bool, use_housing: bool) -> Result<()> {
    // CUSTOMIZE HERE: Modify the evaluation process
    
    println!("Evaluating model...");
    
    // Check if model exists
    if !Path::new(&model_file).exists() {
        return Err(anyhow!("Model file not found: {}", model_file));
    }
    
    // Check if statistics exist
    if !Path::new(&stats_file).exists() {
        return Err(anyhow!("Statistics file not found: {}", stats_file));
    }
    
    // Load statistics
    let stats = DataStats::load(&stats_file)?;
    
    // Load dataset
    let dataset = if use_sample {
        println!("Using generic sample dataset");
        create_sample_dataset()
    } else if use_housing {
        println!("Using housing price sample dataset");
        create_housing_dataset()
    } else {
        println!("Loading dataset from {}", data_file);
        load_csv_dataset(&data_file)?
    };
    
    println!("Dataset loaded with {} examples", dataset.len());
    
    // Create data loader
    let batcher = DataBatcher::<burn::backend::NdArray<f32>>::new(BATCH_SIZE);
    
    let test_loader = DataLoaderBuilder::new(batcher)
        .batch_size(BATCH_SIZE)
        .shuffle(false)
        .build(dataset);
    
    // Load model
    let recorder = DefaultRecorder::new();
    let model: PredictorModel<burn::backend::NdArray<f32>> = recorder.load(&model_file)?;
    
    // Create validation step
    let validation_step = ValidationStep { model };
    
    // Setup progress bar
    let progress_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap();
    
    let progress_bar = ProgressBar::new(test_loader.len() as u64);
    progress_bar.set_style(progress_style);
    
    // Evaluation loop
    let mut test_loss_sum = 0.0;
    let mut test_mse_sum = 0.0;
    let mut test_mae_sum = 0.0;
    let mut test_r2_sum = 0.0;
    let test_batches = test_loader.len();
    
    // Collect predictions and actual values for detailed analysis
    let mut all_predictions = Vec::new();
    let mut all_targets = Vec::new();
    
    for (batch_idx, batch) in test_loader.enumerate() {
        let output = validation_step.step(batch.clone());
        
        test_loss_sum += output.loss;
        test_mse_sum += output.mse;
        test_mae_sum += output.mae;
        test_r2_sum += output.r2;
        
        // Store predictions and targets
        let predictions = validation_step.model.forward(batch.features.clone());
        let pred_data = predictions.into_data();
        let target_data = batch.targets.into_data();
        
        for i in 0..pred_data.value.len() {
            all_predictions.push(pred_data.value[i]);
            all_targets.push(target_data.value[i]);
        }
        
        progress_bar.set_position((batch_idx + 1) as u64);
    }
    
    progress_bar.finish_with_message("Evaluation complete");
    
    let test_loss = test_loss_sum / test_batches as f32;
    let test_mse = test_mse_sum / test_batches as f32;
    let test_mae = test_mae_sum / test_batches as f32;
    let test_r2 = test_r2_sum / test_batches as f32;
    
    println!("\nTest Metrics:");
    println!("MSE: {:.6}", test_mse);
    println!("RMSE: {:.6}", test_mse.sqrt());
    println!("MAE: {:.6}", test_mae);
    println!("R²: {:.4}", test_r2);
    
    // Denormalize predictions and targets for interpretability
    if all_predictions.len() > 0 {
        let mut denorm_predictions = all_predictions.clone();
        let mut denorm_targets = all_targets.clone();
        
        for i in 0..denorm_predictions.len() {
            let mut pred = vec![denorm_predictions[i]];
            let mut target = vec![denorm_targets[i]];
            
            stats.denormalize_targets(&mut pred);
            stats.denormalize_targets(&mut target);
            
            denorm_predictions[i] = pred[0];
            denorm_targets[i] = target[0];
        }
        
        // Show some example predictions
        println!("\nSample Predictions (showing first 10):");
        println!("Actual\t\tPredicted\tError\t\tError %");
        
        let n_samples = std::cmp::min(10, denorm_predictions.len());
        for i in 0..n_samples {
            let error = denorm_predictions[i] - denorm_targets[i];
            let error_pct = if denorm_targets[i] != 0.0 {
                (error / denorm_targets[i]) * 100.0
            } else {
                0.0
            };
            
            println!("{:.2}\t\t{:.2}\t\t{:.2}\t\t{:.2}%", 
                    denorm_targets[i], denorm_predictions[i], error, error_pct);
        }
    }
    
    Ok(())
}

// Predict values for new data
fn predict_values(features_str: String, model_file: String, stats_file: String) -> Result<()> {
    // CUSTOMIZE HERE: Modify the prediction process
    
    println!("Predicting values for new data...");
    
    // Check if model exists
    if !Path::new(&model_file).exists() {
        return Err(anyhow!("Model file not found: {}", model_file));
    }
    
    // Check if statistics exist
    if !Path::new(&stats_file).exists() {
        return Err(anyhow!("Statistics file not found: {}", stats_file));
    }
    
    // Load statistics
    let stats = DataStats::load(&stats_file)?;
    
    // Parse feature values
    let feature_values: Result<Vec<f32>, _> = features_str.split(',')
        .map(|s| s.trim().parse::<f32>())
        .collect();
    
    let mut features = feature_values?;
    
    // Check if number of features matches
    if features.len() != stats.feature_means.len() {
        return Err(anyhow!("Expected {} features, but got {}", 
                          stats.feature_means.len(), features.len()));
    }
    
    // Normalize features
    stats.normalize_features(&mut features);
    
    // Create a tensor from the features
    let features_data = burn::tensor::Data::new(
        features.clone(),
        [1, features.len()],
    );
    
    let features_tensor = burn::tensor::Tensor::<burn::backend::NdArray<f32>, 2>::from_data(features_data);
    
    // Load model
    let recorder = DefaultRecorder::new();
    let model: PredictorModel<burn::backend::NdArray<f32>> = recorder.load(&model_file)?;
    
    // Forward pass
    let predictions = model.forward(features_tensor);
    
    // Get prediction
    let mut prediction = vec![predictions.into_data().value[0]];
    
    // Denormalize prediction
    stats.denormalize_targets(&mut prediction);
    
    // Print results
    println!("\nPrediction Result:");
    
    // Print feature names and values
    println!("\nInput Features:");
    for (i, name) in stats.feature_columns.iter().enumerate() {
        let mut orig_feature = vec![features[i]];
        stats.denormalize_features(&mut orig_feature);
        println!("{}: {:.4}", name, orig_feature[0]);
    }
    
    // Print prediction
    println!("\nPredicted {}:", stats.target_columns[0]);
    println!("{:.4}", prediction[0]);
    
    Ok(())
}

// Predict values for data in a CSV file
fn predict_csv(file: String, model_file: String, stats_file: String) -> Result<()> {
    // CUSTOMIZE HERE: Modify the CSV prediction process
    
    println!("Predicting values for data in CSV file: {}", file);
    
    // Check if model exists
    if !Path::new(&model_file).exists() {
        return Err(anyhow!("Model file not found: {}", model_file));
    }
    
    // Check if statistics exist
    if !Path::new(&stats_file).exists() {
        return Err(anyhow!("Statistics file not found: {}", stats_file));
    }
    
    // Load statistics
    let stats = DataStats::load(&stats_file)?;
    
    // Load CSV file
    let file = fs::File::open(&file)?;
    let reader = io::BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    
    // Get headers
    let headers = csv_reader.headers()?.clone();
    
    // Find feature column indices
    let mut feature_indices = Vec::new();
    for feature_col in &stats.feature_columns {
        if let Some(idx) = headers.iter().position(|h| h == feature_col) {
            feature_indices.push(idx);
        } else {
            return Err(anyhow!("Feature column '{}' not found in CSV", feature_col));
        }
    }
    
    // Create output CSV
    let mut output_path = Path::new(&file).to_path_buf();
    output_path.set_file_name(format!("{}_predicted.csv", 
                                    Path::new(&file).file_stem().unwrap().to_string_lossy()));
    
    let output_file = fs::File::create(output_path.clone())?;
    let mut writer = csv::Writer::from_writer(output_file);
    
    // Write headers
    let mut output_headers = headers.clone();
    output_headers.push_field(format!("predicted_{}", stats.target_columns[0]));
    writer.write_record(&output_headers)?;
    
    // Load model
    let recorder = DefaultRecorder::new();
    let model: PredictorModel<burn::backend::NdArray<f32>> = recorder.load(&model_file)?;
    
    // Setup progress bar
    let record_count = csv_reader.records().count();
    let progress_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap();
    
    let progress_bar = ProgressBar::new(record_count as u64);
    progress_bar.set_style(progress_style);
    
    // Reset reader
    let file = fs::File::open(&file)?;
    let reader = io::BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    csv_reader.headers()?; // Skip headers
    
    // Process each record
    for (i, result) in csv_reader.records().enumerate() {
        let record = result?;
        
        // Extract features
        let mut features = Vec::new();
        for &idx in &feature_indices {
            let value = record.get(idx)
                .ok_or_else(|| anyhow!("Missing feature value"))?
                .parse::<f32>()?;
            features.push(value);
        }
        
        // Normalize features
        let mut normalized_features = features.clone();
        stats.normalize_features(&mut normalized_features);
        
        // Create a tensor from the features
        let features_data = burn::tensor::Data::new(
            normalized_features.clone(),
            [1, normalized_features.len()],
        );
        
        let features_tensor = burn::tensor::Tensor::<burn::backend::NdArray<f32>, 2>::from_data(features_data);
        
        // Forward pass
        let predictions = model.forward(features_tensor);
        
        // Get prediction
        let mut prediction = vec![predictions.into_data().value[0]];
        
        // Denormalize prediction
        stats.denormalize_targets(&mut prediction);
        
        // Write output record
        let mut output_record = record.clone();
        output_record.push_field(format!("{:.4}", prediction[0]));
        writer.write_record(&output_record)?;
        
        progress_bar.set_position((i + 1) as u64);
    }
    
    progress_bar.finish_with_message("Prediction complete");
    
    println!("Prediction complete. Results saved to {}", output_path.display());
    
    Ok(())
}
