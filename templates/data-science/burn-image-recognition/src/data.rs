use burn::{
    data::{
        dataloader::{batcher::Batcher, DataLoaderBuilder},
        dataset::Dataset,
    },
    tensor::{backend::Backend, Tensor, TensorData, Int},
};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
    sync::Arc,
};

/// MNIST dataset item
#[derive(Clone, Debug)]
pub struct MnistItem {
    pub image: Vec<f32>,
    pub label: usize,
}

/// MNIST batch
#[derive(Clone, Debug)]
pub struct MnistBatch<B: Backend> {
    /// Images
    pub images: Tensor<B, 4>,
    /// Targets
    pub targets: Tensor<B, 1, Int>,
}

impl<B: Backend> MnistBatch<B> {
    /// Move batch to device
    pub fn to_device(&self, device: &B::Device) -> Self {
        Self {
            images: self.images.clone().to_device(device),
            targets: self.targets.clone().to_device(device),
        }
    }
}

/// MNIST batcher
#[derive(Clone)]
pub struct MnistBatcher<B: Backend> {
    device: B::Device,
}

impl<B: Backend> MnistBatcher<B> {
    /// Create a new batcher
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

/// Create dataloaders for MNIST dataset
pub fn create_dataloaders<B: Backend>(
    device: &B::Device,
    batch_size: usize,
) -> (
    Arc<dyn burn::data::dataloader::DataLoader<MnistBatch<B>>>,
    Arc<dyn burn::data::dataloader::DataLoader<MnistBatch<B>>>,
) {
    // Create datasets
    let train_dataset = MnistDataset::from_path("data/mnist/train-images-idx3-ubyte", "data/mnist/train-labels-idx1-ubyte");
    let test_dataset = MnistDataset::from_path("data/mnist/t10k-images-idx3-ubyte", "data/mnist/t10k-labels-idx1-ubyte");
    
    // Create batcher
    let batcher = MnistBatcher::new(device.clone());
    
    // Create dataloaders
    let train_loader = DataLoaderBuilder::new(batcher.clone())
        .batch_size(batch_size)
        .shuffle(42) // Use a seed value instead of boolean
        .build(train_dataset);
    
    let test_loader = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .build(test_dataset);
    
    (train_loader, test_loader)
}

/// MNIST dataset
pub struct MnistDataset {
    images: Vec<Vec<u8>>,
    labels: Vec<u8>,
}

impl MnistDataset {
    /// Create a new dataset from path
    pub fn from_path(
        images_file_path: impl AsRef<Path>,
        labels_file_path: impl AsRef<Path>,
    ) -> Self {
        let images = read_idx_images(images_file_path);
        let labels = read_idx_labels(labels_file_path);
        
        Self { images, labels }
    }
}

impl Dataset<MnistItem> for MnistDataset {
    fn get(&self, index: usize) -> Option<MnistItem> {
        if index < self.len() {
            let image = self.images[index]
                .iter()
                .map(|&x| (x as f32) / 255.0)
                .collect();
            
            Some(MnistItem {
                image,
                label: self.labels[index] as usize,
            })
        } else {
            None
        }
    }
    
    fn len(&self) -> usize {
        self.labels.len()
    }
}

impl<B: Backend> Batcher<MnistItem, MnistBatch<B>> for MnistBatcher<B> {
    fn batch(&self, items: Vec<MnistItem>) -> MnistBatch<B> {
        // For each image: create [1, 1, 28, 28] tensor, normalize, then batch to [batch, 1, 28, 28]
        let images: Vec<_> = items.iter().map(|item| {
            let tensor = Tensor::<B, 1>::from_data(
                TensorData::from(item.image.as_slice()),
                &self.device,
            ).reshape([1, 28, 28]);
            let tensor = tensor.unsqueeze::<4>(); // Now [1, 1, 28, 28]
            ((tensor / 255.0) - 0.1307) / 0.3081
        }).collect();
        let images = Tensor::cat(images, 0); // [batch, 1, 28, 28]

        // Create targets as a single 1D tensor [batch] for classification (integer indices)
        let targets: Vec<_> = items.iter().map(|item| item.label as i64).collect();
        let targets = Tensor::<B, 1, Int>::from_data(
            TensorData::from(targets.as_slice()),
            &self.device,
        ); // [batch]

        MnistBatch { images, targets }
    }
}

/// Read IDX images file
fn read_idx_images(path: impl AsRef<Path>) -> Vec<Vec<u8>> {
    let file = File::open(path).expect("Cannot open file");
    let mut reader = BufReader::new(file);
    
    let mut buf = [0; 4];
    reader.read_exact(&mut buf).expect("Cannot read magic number");
    assert_eq!(u32::from_be_bytes(buf), 2051, "Invalid magic number");
    
    reader.read_exact(&mut buf).expect("Cannot read number of images");
    let num_images = u32::from_be_bytes(buf) as usize;
    
    reader.read_exact(&mut buf).expect("Cannot read number of rows");
    let num_rows = u32::from_be_bytes(buf) as usize;
    
    reader.read_exact(&mut buf).expect("Cannot read number of columns");
    let num_cols = u32::from_be_bytes(buf) as usize;
    
    let mut images = Vec::with_capacity(num_images);
    for _ in 0..num_images {
        let mut image = vec![0; num_rows * num_cols];
        reader.read_exact(&mut image).expect("Cannot read image");
        images.push(image);
    }
    
    images
}

/// Read IDX labels file
fn read_idx_labels(path: impl AsRef<Path>) -> Vec<u8> {
    let file = File::open(path).expect("Cannot open file");
    let mut reader = BufReader::new(file);
    
    let mut buf = [0; 4];
    reader.read_exact(&mut buf).expect("Cannot read magic number");
    assert_eq!(u32::from_be_bytes(buf), 2049, "Invalid magic number");
    
    reader.read_exact(&mut buf).expect("Cannot read number of items");
    let num_items = u32::from_be_bytes(buf) as usize;
    
    let mut labels = vec![0; num_items];
    reader.read_exact(&mut labels).expect("Cannot read labels");
    
    labels
}
