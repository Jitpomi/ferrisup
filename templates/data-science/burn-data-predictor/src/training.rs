// Training functionality for numerical data prediction models

use burn::module::Module;
use burn::tensor::backend::Backend;
use burn::train::{RegressionOutput, TrainOutput, TrainStep, ValidStep};
use burn::optim::Adam;

use crate::model::DataPredictorModel;
use crate::data::DataItem;

// Training step handler - manages one step of training
#[derive(Clone)]
pub struct TrainingStepHandler<B: Backend> {
    model: DataPredictorModel<B>,
    optimizer: Adam<B>,
}

impl<B: Backend> TrainingStepHandler<B> {
    pub fn new(model: DataPredictorModel<B>, optimizer: Adam<B>) -> Self {
        Self { model, optimizer }
    }
}

// Implement the TrainStep trait for our training handler
impl<B: Backend> TrainStep<DataItem<B>, RegressionOutput> for TrainingStepHandler<B> {
    fn step(&mut self, batch: &DataItem<B>) -> TrainOutput<RegressionOutput> {
        // Forward pass
        let output = self.model.forward(batch.features.clone());
        
        // Calculate loss (Mean Squared Error)
        let loss = output.mse_loss(batch.targets.clone());
        
        // Backward pass and optimization
        let grads = loss.backward();
        self.optimizer.update(&mut self.model, grads);
        
        // Calculate metrics (Mean Absolute Error)
        let mae = output.mae(batch.targets.clone());
        
        TrainOutput::new(
            loss.into_scalar(), 
            RegressionOutput::new(mae.into_scalar())
        )
    }
}

// Validation step handler - manages one step of validation
#[derive(Clone)]
pub struct ValidationStepHandler<B: Backend> {
    model: DataPredictorModel<B>,
}

impl<B: Backend> ValidationStepHandler<B> {
    pub fn new(model: DataPredictorModel<B>) -> Self {
        Self { model }
    }
}

// Implement the ValidStep trait for our validation handler
impl<B: Backend> ValidStep<DataItem<B>, RegressionOutput> for ValidationStepHandler<B> {
    fn step(&mut self, batch: &DataItem<B>) -> RegressionOutput {
        // Forward pass
        let output = self.model.forward(batch.features.clone());
        
        // Calculate metrics (Mean Absolute Error)
        let mae = output.mae(batch.targets.clone());
        
        RegressionOutput::new(mae.into_scalar())
    }
}
