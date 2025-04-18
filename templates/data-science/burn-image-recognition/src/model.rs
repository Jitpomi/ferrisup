use crate::data::MnistBatch;
use burn::{
    nn::{loss::CrossEntropyLossConfig, BatchNorm, PaddingConfig2d},
    prelude::*,
    tensor::backend::AutodiffBackend,
    train::{ClassificationOutput, TrainOutput, TrainStep, ValidStep},
};
use burn::record::{Recorder, CompactRecorder};

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    conv1: ConvBlock<B>,
    conv2: ConvBlock<B>,
    conv3: ConvBlock<B>,
    dropout: nn::Dropout,
    fc1: nn::Linear<B>,
    fc2: nn::Linear<B>,
    activation: nn::Gelu,
}

const NUM_CLASSES: usize = 10;

impl<B: Backend> Model<B> {
    pub fn new(device: &B::Device) -> Self {
        let conv1 = ConvBlock::new([1, 8], [3, 3], device); // out: [Batch,8,26,26]
        let conv2 = ConvBlock::new([8, 16], [3, 3], device); // out: [Batch,16,24x24]
        let conv3 = ConvBlock::new([16, 24], [3, 3], device); // out: [Batch,24,22x22]
        let hidden_size = 24 * 22 * 22;
        let fc1 = nn::LinearConfig::new(hidden_size, 32)
            .with_bias(false)
            .init(device);
        let fc2 = nn::LinearConfig::new(32, NUM_CLASSES)
            .with_bias(false)
            .init(device);

        let dropout = nn::DropoutConfig::new(0.5).init();

        Self {
            conv1,
            conv2,
            conv3,
            dropout,
            fc1,
            fc2,
            activation: nn::Gelu::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 3>) -> Tensor<B, 2> {
        let [batch_size, height, width] = input.dims();
        println!("Input dims: [{}, {}, {}]", batch_size, height, width);
        
        let x = input.reshape([batch_size, 1, height, width]).detach();
        println!("After reshape to 4D: {:?}", x.dims());
        
        let x = self.conv1.forward(x);
        println!("After conv1: {:?}", x.dims());
        
        let x = self.conv2.forward(x);
        println!("After conv2: {:?}", x.dims());
        
        let x = self.conv3.forward(x);
        println!("After conv3: {:?}", x.dims());
        
        let [batch_size, channels, height, width] = x.dims();
        println!("Before reshape to 2D: batch_size={}, channels={}, height={}, width={}", batch_size, channels, height, width);
        
        let x = x.reshape([batch_size, channels * height * width]);
        println!("After reshape to 2D: {:?}", x.dims());
        
        let x = self.dropout.forward(x);
        println!("After dropout: {:?}", x.dims());
        
        let x = self.fc1.forward(x);
        println!("After fc1: {:?}", x.dims());
        
        let x = self.activation.forward(x);
        println!("After activation: {:?}", x.dims());
        
        let x = self.fc2.forward(x);
        println!("After fc2: {:?}", x.dims());
        
        x
    }

    pub fn forward_classification(&self, item: MnistBatch<B>) -> ClassificationOutput<B> {
        let targets = item.targets;
        println!("Targets dims: {:?}", targets.dims());
        let output = self.forward(item.images);
        println!("Output dims: {:?}", output.dims());
        let loss = CrossEntropyLossConfig::new()
            .init(&output.device())
            .forward(output.clone(), targets.clone());
        println!("Loss computed, shape should be scalar");

        ClassificationOutput {
            loss,
            output,
            targets,
        }
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let record = self.clone().into_record();
        CompactRecorder::new().record(record, path.to_path_buf())?;
        Ok(())
    }
}

#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    conv: nn::conv::Conv2d<B>,
    norm: BatchNorm<B, 2>,
    activation: nn::Gelu,
}

impl<B: Backend> ConvBlock<B> {
    pub fn new(channels: [usize; 2], kernel_size: [usize; 2], device: &B::Device) -> Self {
        let conv = nn::conv::Conv2dConfig::new(channels, kernel_size)
            .with_padding(PaddingConfig2d::Valid)
            .init(device);
        let norm = nn::BatchNormConfig::new(channels[1]).init(device);

        Self {
            conv,
            norm,
            activation: nn::Gelu::new(),
        }
    }

    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(input);
        let x = self.norm.forward(x);

        self.activation.forward(x)
    }
}

impl<B: AutodiffBackend> TrainStep<MnistBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, item: MnistBatch<B>) -> TrainOutput<ClassificationOutput<B>> {
        let item = self.forward_classification(item);

        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> ValidStep<MnistBatch<B>, ClassificationOutput<B>> for Model<B> {
    fn step(&self, item: MnistBatch<B>) -> ClassificationOutput<B> {
        self.forward_classification(item)
    }
}
