use burn::module::Module;
use burn::nn::{
    conv::{Conv2d, Conv2dConfig},
    pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig},
    Linear, LinearConfig, ReLU,
};
use burn::tensor::{backend::Backend, Tensor};

/// A simple CNN model for image classification
#[derive(Module, Debug)]
pub struct ConvNet<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    pool: AdaptiveAvgPool2d,
    fc1: Linear<B>,
    fc2: Linear<B>,
    relu: ReLU,
}

impl<B: Backend> ConvNet<B> {
    pub fn new(num_classes: usize) -> Self {
        let conv1 = Conv2dConfig::new([1, 32], [3, 3])
            .with_padding([1, 1])
            .init();
        let conv2 = Conv2dConfig::new([32, 64], [3, 3])
            .with_padding([1, 1])
            .init();
        let pool = AdaptiveAvgPool2dConfig::new([1, 1]).init();
        let fc1 = LinearConfig::new(64, 128).init();
        let fc2 = LinearConfig::new(128, num_classes).init();
        let relu = ReLU::new();

        Self {
            conv1,
            conv2,
            pool,
            fc1,
            fc2,
            relu,
        }
    }

    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 2> {
        let x = self.relu.forward(self.conv1.forward(x));
        let x = self.relu.forward(self.conv2.forward(x));
        let x = self.pool.forward(x);
        let x = x.flatten(1);
        let x = self.relu.forward(self.fc1.forward(x));
        self.fc2.forward(x)
    }
}

/// A simple MLP model for tabular data
#[derive(Module, Debug)]
pub struct MLP<B: Backend> {
    fc1: Linear<B>,
    fc2: Linear<B>,
    fc3: Linear<B>,
    relu: ReLU,
}

impl<B: Backend> MLP<B> {
    pub fn new(input_size: usize, hidden_size: usize, num_classes: usize) -> Self {
        let fc1 = LinearConfig::new(input_size, hidden_size).init();
        let fc2 = LinearConfig::new(hidden_size, hidden_size).init();
        let fc3 = LinearConfig::new(hidden_size, num_classes).init();
        let relu = ReLU::new();

        Self {
            fc1,
            fc2,
            fc3,
            relu,
        }
    }

    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.relu.forward(self.fc1.forward(x));
        let x = self.relu.forward(self.fc2.forward(x));
        self.fc3.forward(x)
    }
}
