// Text Sentiment Analyzer Main Application
// This file contains the CLI interface and main program logic

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::Path;
use std::fs;
use std::io::{self, Write};
use burn::tensor::backend::{AutodiffBackend, Backend};
use burn::optim::{AdamConfig, GradientsParams};
use burn::record::{Recorder, DefaultRecorder};
use burn::module::Module;
use burn::data::dataloader::DataLoaderBuilder;
use burn::train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep};
use indicatif::{ProgressBar, ProgressStyle};

// Import our modules
mod config;
mod model;
mod data;

// Import types and constants from our modules
use config::{
    BATCH_SIZE, LEARNING_RATE, EPOCHS, DEFAULT_DATA_DIR,
    DEFAULT_MODEL_FILE, DEFAULT_VOCAB_FILE, EARLY_STOPPING_PATIENCE,
    NUM_CLASSES, CLASS_NAMES
};
use model::{TextAnalyzerModel, TextAnalyzerConfig, create_default_model};
use data::{
    TextBatcher, RawTextItem, TextItem, TextDataset,
    load_text_dataset, load_text_dataset_from_csv, create_sample_dataset
};

// CLI Arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Train a new sentiment analysis model
    Train {
        /// Directory containing training data
        #[arg(short, long, default_value = DEFAULT_DATA_DIR)]
        data_dir: String,
        
        /// Path to save the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_file: String,
        
        /// Path to save the vocabulary
        #[arg(short, long, default_value = DEFAULT_VOCAB_FILE)]
        vocab_file: String,
        
        /// Use sample data instead of loading from disk
        #[arg(short, long, default_value = "false")]
        sample: bool,
    },
    
    /// Evaluate model performance on a test dataset
    Evaluate {
        /// Directory containing test data
        #[arg(short, long, default_value = DEFAULT_DATA_DIR)]
        data_dir: String,
        
        /// Path to the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_file: String,
        
        /// Path to the vocabulary
        #[arg(short, long, default_value = DEFAULT_VOCAB_FILE)]
        vocab_file: String,
        
        /// Use sample data instead of loading from disk
        #[arg(short, long, default_value = "false")]
        sample: bool,
    },
    
    /// Analyze sentiment of a single text
    Analyze {
        /// Text to analyze
        #[arg(short, long)]
        text: String,
        
        /// Path to the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_file: String,
        
        /// Path to the vocabulary
        #[arg(short, long, default_value = DEFAULT_VOCAB_FILE)]
        vocab_file: String,
    },
    
    /// Analyze sentiment of texts from a CSV file
    AnalyzeCsv {
        /// Path to the CSV file
        #[arg(short, long)]
        file: String,
        
        /// Column containing the text
        #[arg(short, long, default_value = "text")]
        text_column: String,
        
        /// Path to the trained model
        #[arg(short, long, default_value = DEFAULT_MODEL_FILE)]
        model_file: String,
        
        /// Path to the vocabulary
        #[arg(short, long, default_value = DEFAULT_VOCAB_FILE)]
        vocab_file: String,
    },
}

// Define the training step
struct TrainingStep<B: AutodiffBackend> {
    model: TextAnalyzerModel<B>,
    optimizer: AdamConfig,
}

impl<B: AutodiffBackend> TrainStep<TextItem<B>, ClassificationOutput<B>> for TrainingStep<B> {
    fn step(&self, item: TextItem<B>) -> TrainOutput<ClassificationOutput<B>> {
        // Forward pass
        let logits = self.model.forward(item.tokens);
        
        // Calculate cross-entropy loss
        let loss = logits.cross_entropy_with_logits(&item.labels);
        
        // Calculate accuracy
        let predictions = logits.argmax(1);
        let targets = item.labels.argmax(1);
        let accuracy = predictions.equal(targets).to_dtype::<f32>().mean().into_scalar();
        
        // Backward pass and return
        let gradients = loss.backward();
        let optimizer = self.optimizer.init();
        let updated_params = optimizer.step(&self.model, &gradients);
        
        TrainOutput::new(
            self.model.clone_with(updated_params),
            ClassificationOutput::new(loss.into_scalar(), accuracy),
        )
    }
}

// Define the validation step
struct ValidationStep<B: Backend> {
    model: TextAnalyzerModel<B>,
}

impl<B: Backend> ValidStep<TextItem<B>, ClassificationOutput<B>> for ValidationStep<B> {
    fn step(&self, item: TextItem<B>) -> ClassificationOutput<B> {
        // Forward pass
        let logits = self.model.forward(item.tokens);
        
        // Calculate cross-entropy loss
        let loss = logits.cross_entropy_with_logits(&item.labels);
        
        // Calculate accuracy
        let predictions = logits.argmax(1);
        let targets = item.labels.argmax(1);
        let accuracy = predictions.equal(targets).to_dtype::<f32>().mean().into_scalar();
        
        ClassificationOutput::new(loss.into_scalar(), accuracy)
    }
}

// Main function
fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Run the appropriate command
    match cli.command {
        Commands::Train { data_dir, model_file, vocab_file, sample } => {
            train_model(data_dir, model_file, vocab_file, sample)?;
        }
        Commands::Evaluate { data_dir, model_file, vocab_file, sample } => {
            evaluate_model(data_dir, model_file, vocab_file, sample)?;
        }
        Commands::Analyze { text, model_file, vocab_file } => {
            analyze_text(text, model_file, vocab_file)?;
        }
        Commands::AnalyzeCsv { file, text_column, model_file, vocab_file } => {
            analyze_csv(file, text_column, model_file, vocab_file)?;
        }
    }
    
    Ok(())
}

// Train a new sentiment analysis model
fn train_model(data_dir: String, model_file: String, vocab_file: String, use_sample: bool) -> Result<()> {
    // CUSTOMIZE HERE: Modify the training process
    
    println!("Starting training process...");
    
    // Load dataset
    let dataset = if use_sample {
        println!("Using sample dataset");
        create_sample_dataset()
    } else {
        println!("Loading dataset from {}", data_dir);
        load_text_dataset(&data_dir)?
    };
    
    // Save vocabulary
    println!("Saving vocabulary to {}", vocab_file);
    dataset.vocabulary().save(&vocab_file)?;
    
    println!("Dataset loaded with {} examples", dataset.len());
    println!("Classes: {:?}", dataset.class_names());
    
    // Split into training and validation sets
    let (train_dataset, valid_dataset) = dataset.split_by_ratio([0.8, 0.2]);
    
    println!("Training set: {} examples", train_dataset.len());
    println!("Validation set: {} examples", valid_dataset.len());
    
    // Create data loaders
    let batcher = TextBatcher::<burn::backend::NdArray<f32>>::new(BATCH_SIZE);
    
    let train_loader = DataLoaderBuilder::new(batcher.clone())
        .batch_size(BATCH_SIZE)
        .shuffle(true)
        .build(train_dataset);
    
    let valid_loader = DataLoaderBuilder::new(batcher)
        .batch_size(BATCH_SIZE)
        .shuffle(false)
        .build(valid_dataset);
    
    // Create model
    let model = create_default_model::<burn::backend::NdArray<f32>>();
    
    // Create optimizer
    let optimizer = AdamConfig::new().with_learning_rate(LEARNING_RATE);
    
    // Create training and validation steps
    let training_step = TrainingStep { model: model.clone(), optimizer };
    let validation_step = ValidationStep { model };
    
    // Create recorder for saving model
    let recorder = DefaultRecorder::new();
    
    // Setup progress bar
    let progress_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap();
    
    // Training loop
    println!("Starting training for {} epochs", EPOCHS);
    
    let mut best_val_accuracy = 0.0;
    let mut patience_counter = 0;
    
    for epoch in 0..EPOCHS {
        println!("Epoch {}/{}", epoch + 1, EPOCHS);
        
        // Training phase
        let mut train_loss_sum = 0.0;
        let mut train_accuracy_sum = 0.0;
        let train_batches = train_loader.len();
        
        let progress_bar = ProgressBar::new(train_batches as u64);
        progress_bar.set_style(progress_style.clone());
        
        let mut current_model = training_step.model.clone();
        
        for (batch_idx, batch) in train_loader.enumerate() {
            let output = training_step.step(batch);
            current_model = output.model;
            
            train_loss_sum += output.context.loss;
            train_accuracy_sum += output.context.accuracy;
            
            progress_bar.set_position((batch_idx + 1) as u64);
        }
        
        progress_bar.finish_with_message("Training complete");
        
        let train_loss = train_loss_sum / train_batches as f32;
        let train_accuracy = train_accuracy_sum / train_batches as f32;
        
        // Validation phase
        let mut valid_loss_sum = 0.0;
        let mut valid_accuracy_sum = 0.0;
        let valid_batches = valid_loader.len();
        
        let progress_bar = ProgressBar::new(valid_batches as u64);
        progress_bar.set_style(progress_style.clone());
        
        let validation_step = ValidationStep { model: current_model.clone() };
        
        for (batch_idx, batch) in valid_loader.enumerate() {
            let output = validation_step.step(batch);
            
            valid_loss_sum += output.loss;
            valid_accuracy_sum += output.accuracy;
            
            progress_bar.set_position((batch_idx + 1) as u64);
        }
        
        progress_bar.finish_with_message("Validation complete");
        
        let valid_loss = valid_loss_sum / valid_batches as f32;
        let valid_accuracy = valid_accuracy_sum / valid_batches as f32;
        
        println!("Train Loss: {:.4}, Train Accuracy: {:.2}%", train_loss, train_accuracy * 100.0);
        println!("Valid Loss: {:.4}, Valid Accuracy: {:.2}%", valid_loss, valid_accuracy * 100.0);
        
        // Save best model
        if valid_accuracy > best_val_accuracy {
            println!("Validation accuracy improved from {:.2}% to {:.2}%, saving model", 
                    best_val_accuracy * 100.0, valid_accuracy * 100.0);
            
            best_val_accuracy = valid_accuracy;
            patience_counter = 0;
            
            // Save model
            recorder.record(current_model.clone(), &model_file)?;
        } else {
            patience_counter += 1;
            println!("Validation accuracy did not improve. Best: {:.2}%", best_val_accuracy * 100.0);
            
            // Early stopping
            if patience_counter >= EARLY_STOPPING_PATIENCE {
                println!("Early stopping triggered after {} epochs without improvement", 
                        patience_counter);
                break;
            }
        }
    }
    
    println!("Training complete. Best validation accuracy: {:.2}%", best_val_accuracy * 100.0);
    println!("Model saved to {}", model_file);
    
    Ok(())
}

// Evaluate model performance on a test dataset
fn evaluate_model(data_dir: String, model_file: String, vocab_file: String, use_sample: bool) -> Result<()> {
    // CUSTOMIZE HERE: Modify the evaluation process
    
    println!("Evaluating model...");
    
    // Check if model exists
    if !Path::new(&model_file).exists() {
        return Err(anyhow!("Model file not found: {}", model_file));
    }
    
    // Check if vocabulary exists
    if !Path::new(&vocab_file).exists() {
        return Err(anyhow!("Vocabulary file not found: {}", vocab_file));
    }
    
    // Load vocabulary
    let vocabulary = data::Vocabulary::load(&vocab_file)?;
    
    // Load dataset
    let dataset = if use_sample {
        println!("Using sample dataset");
        create_sample_dataset()
    } else {
        println!("Loading dataset from {}", data_dir);
        load_text_dataset(&data_dir)?
    };
    
    println!("Dataset loaded with {} examples", dataset.len());
    
    // Create data loader
    let batcher = TextBatcher::<burn::backend::NdArray<f32>>::new(BATCH_SIZE);
    
    let test_loader = DataLoaderBuilder::new(batcher)
        .batch_size(BATCH_SIZE)
        .shuffle(false)
        .build(dataset);
    
    // Load model
    let recorder = DefaultRecorder::new();
    let model: TextAnalyzerModel<burn::backend::NdArray<f32>> = recorder.load(&model_file)?;
    
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
    let mut test_accuracy_sum = 0.0;
    let test_batches = test_loader.len();
    
    // Confusion matrix (predicted x actual)
    let mut confusion_matrix = vec![vec![0; NUM_CLASSES]; NUM_CLASSES];
    
    for (batch_idx, batch) in test_loader.enumerate() {
        let output = validation_step.step(batch.clone());
        
        test_loss_sum += output.loss;
        test_accuracy_sum += output.accuracy;
        
        // Update confusion matrix
        let logits = validation_step.model.forward(batch.tokens);
        let predictions = logits.argmax(1).into_data().value;
        let targets = batch.labels.argmax(1).into_data().value;
        
        for (pred, target) in predictions.iter().zip(targets.iter()) {
            confusion_matrix[*pred][*target] += 1;
        }
        
        progress_bar.set_position((batch_idx + 1) as u64);
    }
    
    progress_bar.finish_with_message("Evaluation complete");
    
    let test_loss = test_loss_sum / test_batches as f32;
    let test_accuracy = test_accuracy_sum / test_batches as f32;
    
    println!("Test Loss: {:.4}, Test Accuracy: {:.2}%", test_loss, test_accuracy * 100.0);
    
    // Print confusion matrix
    println!("\nConfusion Matrix:");
    print!("      ");
    for i in 0..NUM_CLASSES {
        print!("{:<10} ", CLASS_NAMES[i]);
    }
    println!();
    
    for i in 0..NUM_CLASSES {
        print!("{:<6} ", CLASS_NAMES[i]);
        for j in 0..NUM_CLASSES {
            print!("{:<10} ", confusion_matrix[i][j]);
        }
        println!();
    }
    
    // Calculate precision, recall, and F1 score for each class
    println!("\nPer-class Metrics:");
    println!("Class      Precision   Recall      F1 Score");
    
    for i in 0..NUM_CLASSES {
        let true_positives = confusion_matrix[i][i];
        let false_positives: i32 = confusion_matrix[i].iter().enumerate()
            .filter(|&(j, _)| j != i)
            .map(|(_, &count)| count)
            .sum();
        let false_negatives: i32 = (0..NUM_CLASSES)
            .filter(|&j| j != i)
            .map(|j| confusion_matrix[j][i])
            .sum();
        
        let precision = if true_positives + false_positives > 0 {
            true_positives as f32 / (true_positives + false_positives) as f32
        } else {
            0.0
        };
        
        let recall = if true_positives + false_negatives > 0 {
            true_positives as f32 / (true_positives + false_negatives) as f32
        } else {
            0.0
        };
        
        let f1 = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };
        
        println!("{:<10} {:<11.2}% {:<11.2}% {:<.2}%", 
                CLASS_NAMES[i], precision * 100.0, recall * 100.0, f1 * 100.0);
    }
    
    Ok(())
}

// Analyze sentiment of a single text
fn analyze_text(text: String, model_file: String, vocab_file: String) -> Result<()> {
    // CUSTOMIZE HERE: Modify the text analysis process
    
    println!("Analyzing text: \"{}\"", text);
    
    // Check if model exists
    if !Path::new(&model_file).exists() {
        return Err(anyhow!("Model file not found: {}", model_file));
    }
    
    // Check if vocabulary exists
    if !Path::new(&vocab_file).exists() {
        return Err(anyhow!("Vocabulary file not found: {}", vocab_file));
    }
    
    // Load vocabulary
    let vocabulary = data::Vocabulary::load(&vocab_file)?;
    
    // Tokenize the text
    let tokens = data::tokenize(&text)
        .iter()
        .map(|token| vocabulary.token_to_id(token))
        .collect::<Vec<usize>>();
    
    // Create a tensor from the tokens
    let mut tokens_data = burn::tensor::Data::new(
        vec![0; MAX_SEQUENCE_LENGTH],
        [1, MAX_SEQUENCE_LENGTH],
    );
    
    // Copy tokens to the tensor
    for (i, &token) in tokens.iter().enumerate() {
        if i < MAX_SEQUENCE_LENGTH {
            tokens_data.value_mut()[i] = token;
        } else {
            break;
        }
    }
    
    let tokens_tensor = burn::tensor::Tensor::<burn::backend::NdArray<f32>, 2>::from_data(tokens_data);
    
    // Load model
    let recorder = DefaultRecorder::new();
    let model: TextAnalyzerModel<burn::backend::NdArray<f32>> = recorder.load(&model_file)?;
    
    // Forward pass
    let logits = model.forward(tokens_tensor);
    
    // Get predictions
    let probabilities = logits.softmax(1);
    let prediction = logits.argmax(1).into_scalar();
    
    // Print results
    println!("\nSentiment Analysis Results:");
    println!("Predicted sentiment: {} ({:.2}% confidence)", 
            CLASS_NAMES[prediction], probabilities.into_data().value[prediction] * 100.0);
    
    println!("\nProbabilities for each class:");
    for i in 0..NUM_CLASSES {
        println!("{}: {:.2}%", CLASS_NAMES[i], probabilities.into_data().value[i] * 100.0);
    }
    
    Ok(())
}

// Analyze sentiment of texts from a CSV file
fn analyze_csv(file: String, text_column: String, model_file: String, vocab_file: String) -> Result<()> {
    // CUSTOMIZE HERE: Modify the CSV analysis process
    
    println!("Analyzing texts from CSV file: {}", file);
    
    // Check if model exists
    if !Path::new(&model_file).exists() {
        return Err(anyhow!("Model file not found: {}", model_file));
    }
    
    // Check if vocabulary exists
    if !Path::new(&vocab_file).exists() {
        return Err(anyhow!("Vocabulary file not found: {}", vocab_file));
    }
    
    // Load vocabulary
    let vocabulary = data::Vocabulary::load(&vocab_file)?;
    
    // Load CSV file
    let file = fs::File::open(&file)?;
    let reader = io::BufReader::new(file);
    let mut csv_reader = csv::Reader::from_reader(reader);
    
    // Get headers
    let headers = csv_reader.headers()?.clone();
    let text_column_index = headers.iter().position(|h| h == text_column)
        .ok_or_else(|| anyhow!("Text column '{}' not found in CSV", text_column))?;
    
    // Load model
    let recorder = DefaultRecorder::new();
    let model: TextAnalyzerModel<burn::backend::NdArray<f32>> = recorder.load(&model_file)?;
    
    // Create output CSV
    let mut output_path = Path::new(&file).to_path_buf();
    output_path.set_file_name(format!("{}_analyzed.csv", 
                                    Path::new(&file).file_stem().unwrap().to_string_lossy()));
    
    let output_file = fs::File::create(output_path.clone())?;
    let mut writer = csv::Writer::from_writer(output_file);
    
    // Write headers
    let mut output_headers = headers.clone();
    for class in CLASS_NAMES.iter() {
        output_headers.push_field(format!("prob_{}", class));
    }
    output_headers.push_field("predicted_sentiment");
    writer.write_record(&output_headers)?;
    
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
        let text = record.get(text_column_index)
            .ok_or_else(|| anyhow!("Text column not found in record"))?;
        
        // Tokenize the text
        let tokens = data::tokenize(text)
            .iter()
            .map(|token| vocabulary.token_to_id(token))
            .collect::<Vec<usize>>();
        
        // Create a tensor from the tokens
        let mut tokens_data = burn::tensor::Data::new(
            vec![0; MAX_SEQUENCE_LENGTH],
            [1, MAX_SEQUENCE_LENGTH],
        );
        
        // Copy tokens to the tensor
        for (i, &token) in tokens.iter().enumerate() {
            if i < MAX_SEQUENCE_LENGTH {
                tokens_data.value_mut()[i] = token;
            } else {
                break;
            }
        }
        
        let tokens_tensor = burn::tensor::Tensor::<burn::backend::NdArray<f32>, 2>::from_data(tokens_data);
        
        // Forward pass
        let logits = model.forward(tokens_tensor);
        
        // Get predictions
        let probabilities = logits.softmax(1);
        let prediction = logits.argmax(1).into_scalar();
        
        // Write output record
        let mut output_record = record.clone();
        for i in 0..NUM_CLASSES {
            output_record.push_field(format!("{:.4}", probabilities.into_data().value[i]));
        }
        output_record.push_field(CLASS_NAMES[prediction]);
        writer.write_record(&output_record)?;
        
        progress_bar.set_position((i + 1) as u64);
    }
    
    progress_bar.finish_with_message("Analysis complete");
    
    println!("Analysis complete. Results saved to {}", output_path.display());
    
    Ok(())
}
