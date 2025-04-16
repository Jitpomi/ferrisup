use burn::{
    tensor::{backend::Backend, Tensor, Int},
    tensor::backend::AutodiffBackend,
    optim::{AdamConfig, GradientsParams, Optimizer},
    tensor::activation::log_softmax,
};
use crate::{
    model::Model
};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::ops::AddAssign;

/// Output for classification tasks
#[derive(Debug, Clone)]
pub struct ClassificationOutput<B: Backend> {
    pub loss: B::FloatElem,
    pub accuracy: B::FloatElem,
    pub targets: Tensor<B, 1, Int>,
    pub predictions: Tensor<B, 1, Int>,
}

/// Output for training steps
pub struct TrainOutput<T> {
    pub item: T,
}

// Note: TrainOutput cannot implement Clone because GradientsParams doesn't implement Clone
// If you need to clone this struct, you'll need to extract the item and handle gradients separately

impl<T: std::fmt::Debug> std::fmt::Debug for TrainOutput<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrainOutput")
            .field("item", &self.item)
            .finish()
    }
}

/// Training step for model
pub struct ModelTrainStep<B: Backend + AutodiffBackend, O> {
    pub model: Model<B>,
    pub optimizer: O,
    learning_rate: f64,
}

impl<B: Backend<FloatElem = f32> + AutodiffBackend, O: Optimizer<Model<B>, B>> ModelTrainStep<B, O> {
    /// Create a new training step
    pub fn new(model: Model<B>, optimizer: O, learning_rate: f64) -> Self {
        Self { model, optimizer, learning_rate }
    }
    
    /// Perform a single training step
    pub fn step(&mut self, batch: MnistBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        // Forward pass
        let output = self.model.forward(batch.images.clone());
        
        // Calculate loss
        let loss = cross_entropy_loss(&output, &batch.targets);
        
        // Calculate gradients
        let grads = loss.backward();
        
        // Convert gradients to GradientsParams
        let gradients_params = GradientsParams::from_grads(grads, &self.model);
        
        // Update parameters using the optimizer's step method with learning rate
        self.model = self.optimizer.step(self.learning_rate, self.model.clone(), gradients_params);
        
        // Calculate accuracy
        let predicted = output.argmax(1);
        // Flatten predicted and targets for comparison
        let predicted_flat = predicted.clone().flatten::<1>(0, 1);
        let targets_flat = batch.targets.clone().flatten::<1>(0, 1);
        let correct = predicted_flat.clone().equal(targets_flat.clone());
        let accuracy = correct.float().mean().into_scalar();
        
        // Return output
        TrainOutput {
            item: ClassificationOutput {
                loss: loss.into_scalar(),
                accuracy,
                targets: targets_flat.clone(),
                predictions: predicted_flat,
            },
        }
    }
}

/// Train the model
pub fn train<B: Backend<FloatElem = f32> + AutodiffBackend>(
    device: &B::Device,
    num_epochs: usize,
    batch_size: usize,
    learning_rate: f64,
    model_path: impl AsRef<std::path::Path> + std::fmt::Display,
) -> Model<B> 
where
    f32: AddAssign<<B as Backend>::FloatElem>,
{
    // Create model
    let model = Model::new(device);
    
    // Create dataloaders
    let (dataloader_train, mut dataloader_test) = create_dataloaders::<B>(device, batch_size);
    
    // Create optimizer with learning rate
    let optimizer_config = AdamConfig::new();
    let optimizer = optimizer_config.init::<B, Model<B>>();
    
    // Create training state
    let mut train_state = ModelTrainStep::new(model, optimizer, learning_rate);
    
    // Training loop
    for epoch in 0..num_epochs {
        println!("Epoch {}/{}", epoch + 1, num_epochs);
        
        // Training
        let mut train_loss = 0.0;
        let mut train_accuracy = 0.0;
        let mut num_batches = 0;
        
        let progress_bar = ProgressBar::new(60000 / batch_size as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        
        for batch in dataloader_train.iter() {
            // Forward pass and backprop
            let output = train_state.step(batch);
            
            // Update metrics
            train_loss += output.item.loss;
            train_accuracy += output.item.accuracy;
            num_batches += 1;
            
            progress_bar.inc(1);
        }
        
        progress_bar.finish_with_message("Training completed");
        
        // Calculate average metrics
        train_loss /= num_batches as f32;
        train_accuracy /= num_batches as f32;
        
        // Validation
        let (val_loss, val_accuracy) = evaluate(&train_state.model, &mut dataloader_test);
        
        // Print metrics
        println!(
            "Train Loss: {:.4}, Train Accuracy: {:.2}%, Val Loss: {:.4}, Val Accuracy: {:.2}%",
            train_loss,
            train_accuracy * 100.0,
            val_loss,
            val_accuracy * 100.0
        );
    }
    
    // Save model
    println!("Saving model to {}", model_path);
    
    // Save the model using the save method we implemented in Model
    train_state.model.save(model_path.as_ref()).expect("Failed to save model");

    train_state.model
}

/// Evaluate model on test set
pub fn evaluate<B: Backend<FloatElem = f32> + AutodiffBackend>(
    model: &Model<B>,
    test_dataloader: &mut Arc<dyn burn::data::dataloader::DataLoader<MnistBatch<B>>>,
) -> (f32, f32) {
    let mut total_loss = 0.0;
    let mut total_accuracy = 0.0;
    let mut num_batches = 0;
    
    let progress_bar = ProgressBar::new(10000 / 32);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    
    for batch in test_dataloader.iter() {
        // Forward pass
        let output = model.forward(batch.images.clone());
        
        // Calculate loss
        let loss = cross_entropy_loss(&output, &batch.targets);
        
        // Calculate accuracy
        let predicted = output.argmax(1);
        let predicted_flat = predicted.flatten::<1>(0, 1);
        let targets_flat = batch.targets.clone().flatten::<1>(0, 1);

        let correct = predicted_flat.clone().equal(targets_flat.clone());
        let accuracy = correct.float().mean().into_scalar();
        
        // Accumulate metrics
        total_loss += loss.into_scalar();
        total_accuracy += accuracy;
        num_batches += 1;
        
        progress_bar.inc(1);
    }
    
    progress_bar.finish_with_message("Evaluation completed");
    
    // Calculate average metrics
    let avg_loss = total_loss / num_batches as f32;
    let avg_accuracy = total_accuracy / num_batches as f32;
    
    (avg_loss, avg_accuracy)
}

/// Cross entropy loss function
pub fn cross_entropy_loss<B: Backend<FloatElem = f32> + AutodiffBackend>(
    output: &Tensor<B, 2>,
    target: &Tensor<B, 1, Int>,
) -> Tensor<B, 0> {
    // Apply log_softmax to the output
    let log_probs = log_softmax(output.clone(), 1);
    // Gather the log probabilities for the correct class for each sample
    let target = target.clone().unsqueeze(); // shape [batch, 1]
    let log_probs_for_targets = log_probs.gather(1, target).squeeze(1);
    // Negative mean log likelihood
    -log_probs_for_targets.mean_dim(0)
}

pub use crate::data::{create_dataloaders, MnistBatch};
