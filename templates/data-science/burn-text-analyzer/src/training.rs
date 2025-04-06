// Training functionality for text sentiment analysis models

use burn::module::Module;
use burn::tensor::backend::Backend;
use burn::train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep};
use burn::optim::Adam;

use crate::model::TextSentimentModel;
use crate::data::TextItem;

// Training step handler - manages one step of training
#[derive(Clone)]
pub struct TrainingStepHandler<B: Backend> {
    model: TextSentimentModel<B>,
    optimizer: Adam<B>,
}

impl<B: Backend> TrainingStepHandler<B> {
    pub fn new(model: TextSentimentModel<B>, optimizer: Adam<B>) -> Self {
        Self { model, optimizer }
    }
}

// Implement the TrainStep trait for our training handler
impl<B: Backend> TrainStep<TextItem<B>, ClassificationOutput> for TrainingStepHandler<B> {
    fn step(&mut self, batch: &TextItem<B>) -> TrainOutput<ClassificationOutput> {
        // Forward pass
        let output = self.model.forward(batch.tokens.clone(), batch.lengths.clone());
        
        // Calculate loss
        let loss = output.loss(batch.targets.clone());
        
        // Backward pass and optimization
        let grads = loss.backward();
        self.optimizer.update(&mut self.model, grads);
        
        // Calculate accuracy
        let accuracy = output.accuracy(batch.targets.clone());
        
        TrainOutput::new(
            loss.into_scalar(), 
            ClassificationOutput::new(accuracy.into_scalar())
        )
    }
}

// Validation step handler - manages one step of validation
#[derive(Clone)]
pub struct ValidationStepHandler<B: Backend> {
    model: TextSentimentModel<B>,
}

impl<B: Backend> ValidationStepHandler<B> {
    pub fn new(model: TextSentimentModel<B>) -> Self {
        Self { model }
    }
}

// Implement the ValidStep trait for our validation handler
impl<B: Backend> ValidStep<TextItem<B>, ClassificationOutput> for ValidationStepHandler<B> {
    fn step(&mut self, batch: &TextItem<B>) -> ClassificationOutput {
        // Forward pass
        let output = self.model.forward(batch.tokens.clone(), batch.lengths.clone());
        
        // Calculate accuracy
        let accuracy = output.accuracy(batch.targets.clone());
        
        ClassificationOutput::new(accuracy.into_scalar())
    }
}
