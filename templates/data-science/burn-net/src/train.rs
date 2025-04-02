use anyhow::Result;
use burn::{
    module::Module,
    optim::{AdamConfig, Optimizer, OptimizerConfig},
    record::{FullPrecisionSettings, Recorder},
    tensor::{backend::Backend, Tensor},
    train::{ClassificationOutput, LossOutput, TrainOutput, TrainStep, ValidStep},
};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};

use crate::dataset::{load_image, load_mnist, MnistItem};
use crate::model::ConvNet;

/// Train the model on MNIST dataset
pub fn train_mnist<B: Backend>(
    epochs: usize,
    batch_size: usize,
    learning_rate: f32,
    output_path: &Path,
) -> Result<()> {
    println!("🔄 Loading MNIST dataset...");
    let (dataloader_train, dataloader_test) = load_mnist::<B>(batch_size)?;
    
    // Initialize model
    println!("🏗️ Creating model...");
    let model = ConvNet::<B>::new(10);
    
    // Initialize optimizer
    let optimizer = AdamConfig::new()
        .with_learning_rate(learning_rate)
        .init();
    
    // Initialize training state
    let mut train_state = TrainingState::new(model, optimizer);
    
    // Train the model
    println!("🚀 Starting training...");
    let pb = ProgressBar::new(epochs as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} epochs ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    
    for epoch in 0..epochs {
        // Training
        let mut total_loss = 0.0;
        let mut total_samples = 0;
        let mut correct = 0;
        
        for batch in dataloader_train.iter() {
            let item_count = batch.len();
            total_samples += item_count;
            
            let (loss, accuracy) = train_step(&mut train_state, batch);
            total_loss += loss * item_count as f32;
            correct += (accuracy * item_count as f32) as usize;
        }
        
        let train_loss = total_loss / total_samples as f32;
        let train_accuracy = correct as f32 / total_samples as f32;
        
        // Validation
        let mut val_total_loss = 0.0;
        let mut val_total_samples = 0;
        let mut val_correct = 0;
        
        for batch in dataloader_test.iter() {
            let item_count = batch.len();
            val_total_samples += item_count;
            
            let (loss, accuracy) = valid_step(&train_state, batch);
            val_total_loss += loss * item_count as f32;
            val_correct += (accuracy * item_count as f32) as usize;
        }
        
        let val_loss = val_total_loss / val_total_samples as f32;
        let val_accuracy = val_correct as f32 / val_total_samples as f32;
        
        pb.set_message(format!(
            "Loss: {:.4} | Acc: {:.2}% | Val Loss: {:.4} | Val Acc: {:.2}%",
            train_loss,
            train_accuracy * 100.0,
            val_loss,
            val_accuracy * 100.0
        ));
        pb.inc(1);
    }
    pb.finish_with_message("Training complete!");
    
    // Save the model
    println!("💾 Saving model to {}", output_path.display());
    let record = train_state.model.into_record();
    let mut recorder = Recorder::new()
        .with_settings(FullPrecisionSettings::new())
        .init();
    recorder.record(output_path, &record)?;
    
    println!("✅ Model saved successfully!");
    
    Ok(())
}

/// Evaluate the model on MNIST test set
pub fn evaluate_mnist<B: Backend>(model_path: &Path) -> Result<()> {
    println!("🔄 Loading MNIST dataset...");
    let (_, dataloader_test) = load_mnist::<B>(32)?;
    
    // Load the model
    println!("📂 Loading model from {}", model_path.display());
    let model = ConvNet::<B>::new(10);
    let mut recorder = Recorder::new()
        .with_settings(FullPrecisionSettings::new())
        .init();
    let record = recorder.load(model_path)?;
    let model = model.load_record(record);
    
    // Evaluate
    println!("📊 Evaluating model...");
    let mut total_samples = 0;
    let mut correct = 0;
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    pb.set_message("Evaluating...");
    
    for batch in dataloader_test.iter() {
        let item_count = batch.len();
        total_samples += item_count;
        
        let accuracy = evaluate_batch(&model, batch);
        correct += (accuracy * item_count as f32) as usize;
        
        pb.set_message(format!(
            "Processed: {} | Accuracy: {:.2}%",
            total_samples,
            (correct as f32 / total_samples as f32) * 100.0
        ));
    }
    
    let accuracy = correct as f32 / total_samples as f32;
    pb.finish_with_message(format!("Final accuracy: {:.2}%", accuracy * 100.0));
    
    Ok(())
}

/// Evaluate the model on a single image
pub fn evaluate_single<B: Backend>(model_path: &Path, image_path: &Path) -> Result<()> {
    // Load the model
    println!("📂 Loading model from {}", model_path.display());
    let model = ConvNet::<B>::new(10);
    let mut recorder = Recorder::new()
        .with_settings(FullPrecisionSettings::new())
        .init();
    let record = recorder.load(model_path)?;
    let model = model.load_record(record);
    
    // Load the image
    println!("🖼️ Loading image from {}", image_path.display());
    let image = load_image::<B, _>(image_path)?;
    
    // Make prediction
    let output = model.forward(image);
    let prediction = output.argmax(1).into_data().value[0];
    
    println!("🔮 Prediction: {}", prediction);
    
    Ok(())
}

/// Make a prediction on a single image
pub fn predict<B: Backend>(model_path: &PathBuf, image_path: &PathBuf) -> Result<usize> {
    // Load the model
    let model = ConvNet::<B>::new(10);
    let mut recorder = Recorder::new()
        .with_settings(FullPrecisionSettings::new())
        .init();
    let record = recorder.load(model_path)?;
    let model = model.load_record(record);
    
    // Load the image
    let image = load_image::<B, _>(image_path)?;
    
    // Make prediction
    let output = model.forward(image);
    let prediction = output.argmax(1).into_data().value[0];
    
    Ok(prediction)
}

/// Training state
struct TrainingState<B: Backend> {
    model: ConvNet<B>,
    optimizer: AdamConfig<B>,
}

impl<B: Backend> TrainingState<B> {
    fn new(model: ConvNet<B>, optimizer: AdamConfig<B>) -> Self {
        Self { model, optimizer }
    }
}

/// Training step
fn train_step<B: Backend>(
    state: &mut TrainingState<B>,
    batch: Vec<MnistItem<B>>,
) -> (f32, f32) {
    // Forward pass
    let grads = B::GradientTape::new(|tape| {
        let outputs = batch
            .iter()
            .map(|item| {
                let output = state.model.forward_with_tape(item.image.clone(), tape);
                let target = item.label.clone();
                
                ClassificationOutput::new(output, target)
            })
            .collect::<Vec<_>>();
        
        let loss = outputs.iter().map(|o| o.loss()).sum::<Tensor<B, 1>>() / (outputs.len() as f32);
        
        let accuracy = outputs.iter().map(|o| o.accuracy()).sum::<f32>() / (outputs.len() as f32);
        
        TrainOutput::new(loss, accuracy)
    });
    
    // Backward pass and optimize
    state.optimizer.update(&mut state.model, grads);
    
    (grads.value.into_scalar(), grads.aux)
}

/// Validation step
fn valid_step<B: Backend>(
    state: &TrainingState<B>,
    batch: Vec<MnistItem<B>>,
) -> (f32, f32) {
    let outputs = batch
        .iter()
        .map(|item| {
            let output = state.model.forward(item.image.clone());
            let target = item.label.clone();
            
            ClassificationOutput::new(output, target)
        })
        .collect::<Vec<_>>();
    
    let loss = outputs.iter().map(|o| o.loss()).sum::<Tensor<B, 1>>() / (outputs.len() as f32);
    
    let accuracy = outputs.iter().map(|o| o.accuracy()).sum::<f32>() / (outputs.len() as f32);
    
    (loss.into_scalar(), accuracy)
}

/// Evaluate a batch
fn evaluate_batch<B: Backend>(
    model: &ConvNet<B>,
    batch: Vec<MnistItem<B>>,
) -> f32 {
    let outputs = batch
        .iter()
        .map(|item| {
            let output = model.forward(item.image.clone());
            let target = item.label.clone();
            
            ClassificationOutput::new(output, target)
        })
        .collect::<Vec<_>>();
    
    let accuracy = outputs.iter().map(|o| o.accuracy()).sum::<f32>() / (outputs.len() as f32);
    
    accuracy
}
