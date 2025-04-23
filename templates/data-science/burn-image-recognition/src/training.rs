use burn::{
    tensor::{backend::Backend, Tensor, Int},
    tensor::backend::AutodiffBackend,
    optim::{AdamConfig, Optimizer, GradientsParams},
    nn::loss::CrossEntropyLossConfig,
};
use crate::{
    model::Model,
    data::{mnist_dataloader, MnistBatch},
};
use indicatif::{ProgressBar, ProgressStyle};
use std::ops::AddAssign;

pub fn compute_accuracy<B: Backend<FloatElem = f32>>(predicted: &Tensor<B, 1, Int>, targets: &Tensor<B, 1, Int>) -> f32 {
    let correct = predicted.clone().equal(targets.clone());
    correct.float().mean().into_scalar()
}

pub fn create_progress_bar(len: u64, _label: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(ProgressStyle::default_bar().progress_chars("#>-") );
    pb
}

pub struct ModelTrainStep<B: Backend + AutodiffBackend, O> {
    pub model: Model<B>,
    pub optimizer: O,
    pub learning_rate: f64,
}

impl<B: Backend<FloatElem = f32> + AutodiffBackend, O: Optimizer<Model<B>, B>> ModelTrainStep<B, O> {
    pub fn new(model: Model<B>, optimizer: O, learning_rate: f64) -> Self {
        Self { model, optimizer, learning_rate }
    }
    pub fn step(&mut self, batch: &MnistBatch<B>) -> (f32, f32) {
        let output = self.model.forward(&batch.images);
        let loss = CrossEntropyLossConfig::new().init(&output.device()).forward(output.clone(), batch.targets.clone());
        let grads = loss.backward();
        let gradients_params = GradientsParams::from_grads(grads, &self.model);
        self.model = self.optimizer.step(self.learning_rate, self.model.clone(), gradients_params);
        let predicted = output.argmax(1).squeeze(1); // shape [batch_size]
        let accuracy = compute_accuracy(&predicted, &batch.targets); // both shape [batch_size]
        (loss.into_scalar(), accuracy)
    }
}

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
    let model = Model::new(device);
    // When calling dataloader, use:
    let dataloader_train = mnist_dataloader::<B>(true, device.clone(), batch_size, Some(42), 4);
    // When passing to evaluate, use:
    let dataloader_test = mnist_dataloader::<B>(false, device.clone(), batch_size, None, 2);
    let optimizer_config = AdamConfig::new();
    let optimizer = optimizer_config.init::<B, Model<B>>();
    let mut train_state = ModelTrainStep::new(model, optimizer, learning_rate);
    for epoch in 0..num_epochs {
        println!("Epoch {}/{}", epoch + 1, num_epochs);
        let mut train_loss: f32 = 0.0;
        let mut train_accuracy: f32 = 0.0;
        let mut num_batches = 0;
        let progress_bar = create_progress_bar(60000 / batch_size as u64, "Training");
        for batch in dataloader_train.iter() {
            let (loss, accuracy) = train_state.step(batch);
            train_loss += loss;
            train_accuracy += accuracy;
            num_batches += 1;
            progress_bar.inc(1);
        }
        progress_bar.finish_with_message("Training completed");
        if num_batches > 0 {
            train_loss /= num_batches as f32;
            train_accuracy /= num_batches as f32;
        }
        let (val_loss, val_accuracy) = evaluate(&train_state.model, dataloader_test.as_ref());
        println!(
            "Train Loss: {:.4}, Train Accuracy: {:.2}%, Val Loss: {:.4}, Val Accuracy: {:.2}%",
            train_loss,
            train_accuracy * 100.0,
            val_loss,
            val_accuracy * 100.0
        );
    }
    println!("Saving model to {}", model_path);
    train_state.model.save(model_path.as_ref()).expect("Failed to save model");
    train_state.model
}

pub fn evaluate<B: Backend<FloatElem = f32>>(
    model: &Model<B>,
    dataloader: &dyn burn::data::dataloader::DataLoader<MnistBatch<B>>,
) -> (f32, f32) {
    // When passing dataloader, use .as_ref()
    let mut total_loss: f32 = 0.0;
    let mut total_accuracy: f32 = 0.0;
    let mut num_batches = 0;
    let progress_bar = create_progress_bar(10000 / 32, "Evaluation");
    for batch in dataloader.iter() {
        let output = model.forward(&batch.images);
        let loss = CrossEntropyLossConfig::new().init(&output.device()).forward(output.clone(), batch.targets.clone());
        let predicted = output.argmax(1).squeeze(1); // shape [batch_size]
        let accuracy = compute_accuracy(&predicted, &batch.targets); // both shape [batch_size]
        total_loss += loss.into_scalar();
        total_accuracy += accuracy;
        num_batches += 1;
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Evaluation completed");
    if num_batches > 0 {
        (total_loss / num_batches as f32, total_accuracy / num_batches as f32)
    } else {
        (0.0, 0.0)
    }
}
