use crate::data::{MnistBatch, mnist_dataloader};
use crate::model::Model;
use burn::{
    tensor::backend::Backend,
    train::{
        ClassificationOutput, LearningRate, Optimizer, OptimizerConfig, StepOutput, TrainStep, ValidStep,
    },
    record::{Recorder, CompactRecorder},
    prelude::*,
};
use std::sync::Arc;

pub fn train<B: burn::tensor::backend::AutodiffBackend>(
    device: &B::Device,
    num_epochs: usize,
    batch_size: usize,
    learning_rate: f64,
    model_path: String,
) {
    // Create the model and optimizer
    let model = Model::new(device);
    let mut optimizer = burn::optim::AdamConfig::new()
        .with_learning_rate(learning_rate)
        .with_weight_decay(1e-5)
        .init();

    // Create the training and validation data loaders
    let train_loader = mnist_dataloader::<B>(true, device, batch_size, Some(42), 2);
    let valid_loader = mnist_dataloader::<B>(false, device, batch_size, None, 2);

    // Initialize the recorder
    let mut recorder = CompactRecorder::new();

    // Training loop
    for epoch in 0..num_epochs {
        let mut train_loss = 0.0;
        let mut train_acc = 0.0;
        let mut train_batches = 0;

        // Training
        for batch in train_loader.iter() {
            let output = model.step(batch);
            let batch_loss = output.loss.clone().into_scalar();
            let batch_accuracy = accuracy(output.item);

            train_loss += batch_loss;
            train_acc += batch_accuracy;
            train_batches += 1;

            // Update the model
            optimizer.update(&mut *output.model, output.gradients);

            // Print progress
            if train_batches % 100 == 0 {
                println!(
                    "Epoch: {}/{}, Batch: {}, Loss: {:.4}, Accuracy: {:.2}%",
                    epoch + 1,
                    num_epochs,
                    train_batches,
                    train_loss / train_batches as f64,
                    train_acc * 100.0 / train_batches as f64
                );
            }
        }

        // Calculate average training metrics
        train_loss /= train_batches as f64;
        train_acc /= train_batches as f64;

        // Validation
        let (val_loss, val_acc) = evaluate::<B>(&model, valid_loader.as_ref());

        println!(
            "Epoch: {}/{}, Train Loss: {:.4}, Train Acc: {:.2}%, Val Loss: {:.4}, Val Acc: {:.2}%",
            epoch + 1,
            num_epochs,
            train_loss,
            train_acc * 100.0,
            val_loss,
            val_acc * 100.0
        );
    }

    // Save the model
    recorder.record(&model);
    recorder.save(model_path).expect("Failed to save model");
}

pub fn evaluate<B: Backend>(
    model: &Model<B>,
    loader: &dyn burn::data::dataloader::DataLoader<MnistBatch<B>>,
) -> (f64, f64) {
    let mut total_loss = 0.0;
    let mut total_acc = 0.0;
    let mut num_batches = 0;

    for batch in loader.iter() {
        let output = model.step(batch);
        let batch_loss = output.loss.into_scalar();
        let batch_accuracy = accuracy(output);

        total_loss += batch_loss;
        total_acc += batch_accuracy;
        num_batches += 1;
    }

    (total_loss / num_batches as f64, total_acc / num_batches as f64)
}

fn accuracy<B: Backend>(output: ClassificationOutput<B>) -> f64 {
    let predictions = output.output.argmax(1);
    let targets = output.targets;
    
    let pred_data = predictions.to_data();
    let target_data = targets.to_data();
    
    let pred = pred_data.as_slice::<i64>().unwrap();
    let target = target_data.as_slice::<i64>().unwrap();
    
    let correct = pred.iter().zip(target.iter()).filter(|&(a, b)| a == b).count();
    correct as f64 / pred.len() as f64
}
