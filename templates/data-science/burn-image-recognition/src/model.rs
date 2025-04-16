use burn::{
    module::Module,
    nn::{conv::{Conv2dConfig}, BatchNorm, BatchNormConfig, Linear, LinearConfig},
    tensor::{backend::Backend, Tensor},
    record::{NamedMpkFileRecorder, FullPrecisionSettings, Recorder},
    tensor::activation::relu,
};

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    conv1: ConvBlock<B>,
    conv2: ConvBlock<B>,
    fc1: Linear<B>,
    
    fc2: Linear<B>,
}

impl<B: Backend> Model<B> {
    /// Create a new model
    pub fn new(device: &B::Device) -> Self {
        let conv1 = ConvBlock::new(
            Conv2dConfig::new([1, 32], [3, 3]),
            device,
        );
        
        let conv2 = ConvBlock::new(
            Conv2dConfig::new([32, 64], [3, 3]),
            device,
        );
        
        let fc1 = LinearConfig::new(1600, 128)
            .init(device);
        
        let fc2 = LinearConfig::new(128, 10)
            .init(device);
        
        Self {
            conv1,
            conv2,
            fc1,
            fc2,
        }
    }
    
    /// Load a model from a file
    pub fn load<P: Into<std::path::PathBuf>>(path: P) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
        let device = <B as Backend>::Device::default();
        let pathbuf: std::path::PathBuf = path.into();
        let record = recorder.load(pathbuf, &device)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)?;
        let mut model = Self::new(&device);
        model = model.load_record(record);
        Ok(model)
    }
    
    /// Save the model to a file
    pub fn save<P: Into<std::path::PathBuf> + Clone>(&self, path: P) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
        let pathbuf: std::path::PathBuf = path.clone().into();
        self.clone().save_file(pathbuf, &recorder)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)
    }
    
    /// Forward pass
    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 2> {
        // First conv block
        let x = self.conv1.forward(input);
        
        // Max pooling (Burn 0.16+)
        let x = burn::tensor::module::max_pool2d(x, [2, 2], [2, 2], [0, 0], [1, 1]);
        
        // Second conv block
        let x = self.conv2.forward(x);
        
        // Max pooling (Burn 0.16+)
        let x = burn::tensor::module::max_pool2d(x, [2, 2], [2, 2], [0, 0], [1, 1]);
        
        // Flatten
        let [batch_size, channels, height, width] = x.dims();
        let x = x.reshape([batch_size, channels * height * width]);
        
        // Fully connected layers
        let x = relu(self.fc1.forward(x));
        let x = self.fc2.forward(x);
        
        x
    }
}

#[derive(Module, Debug)]
pub struct ConvBlock<B: Backend> {
    conv: burn::nn::conv::Conv2d<B>,
    norm: BatchNorm<B, 2>,
}

impl<B: Backend> ConvBlock<B> {
    pub fn new(config: Conv2dConfig, device: &B::Device) -> Self {
        let conv = config.init(device);
        let norm = BatchNormConfig::new(config.channels[1])
            .init(device);
            
        Self { conv, norm }
    }
    
    pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
        // Do NOT reshape before batch norm. BatchNorm2D expects 4D: [batch, channels, height, width]
        let x = self.conv.forward(input);
        let x = self.norm.forward(x);
        relu(x)
    }
}

impl<B: Backend> Default for Model<B> {
    fn default() -> Self {
        let device = B::Device::default();
        Self::new(&device)
    }
}
