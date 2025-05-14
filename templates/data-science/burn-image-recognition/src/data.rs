use burn::data::dataset::Dataset;
use burn::data::dataloader::{DataLoader, DataLoaderBuilder, batcher::Batcher};
use burn::tensor::{backend::Backend, Int, Tensor, Data, Shape};
use burn::prelude::*;
use std::path::Path;
use std::sync::Arc;

/// Normalize a single MNIST pixel value (u8 or f32) to f32 with PyTorch stats.
pub fn normalize_mnist_pixel<T: Into<f32>>(pixel: T) -> f32 {
    ((pixel.into() / 255.0) - 0.1307) / 0.3081
}

#[derive(Debug, Clone)]
pub struct MnistItem {
    pub image: Vec<f32>,
    pub label: usize,
}

#[derive(Debug, Clone)]
pub struct MnistBatch<B: Backend> {
    pub images: Tensor<B, 3>,
    pub targets: Tensor<B, 1, Int>,
}

#[derive(Debug, Clone)]
pub struct MnistBatcher;

impl MnistBatcher {
    pub fn new() -> Self {
        Self
    }
}

impl<B: Backend> Batcher<B, MnistItem, MnistBatch<B>> for MnistBatcher {
    fn batch(&self, items: Vec<MnistItem>, device: &B::Device) -> MnistBatch<B> {
        let batch_size = items.len();
        
        // Create a flat vector of all pixel values
        let mut image_data = Vec::with_capacity(batch_size * 28 * 28);
        for item in &items {
            image_data.extend_from_slice(&item.image);
        }
        
        // Create the images tensor
        let images = Tensor::<B, 3>::from_data(
            Data::new(image_data, Shape::new([batch_size, 28, 28])),
            device
        );

        // Create the targets tensor
        let targets = Tensor::<B, 1, Int>::from_data(
            Data::new(items.iter().map(|item| item.label as i64).collect::<Vec<_>>(), Shape::new([batch_size])),
            device
        );

        MnistBatch { images, targets }
    }
}

pub struct MnistDataset {
    images: Vec<MnistItem>,
}

impl MnistDataset {
    pub fn new(images: Vec<MnistItem>) -> Self {
        Self { images }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let images = std::fs::read(path.join("train-images-idx3-ubyte")).unwrap();
        let labels = std::fs::read(path.join("train-labels-idx1-ubyte")).unwrap();

        // Skip the header bytes: 16 for images, 8 for labels
        let images = &images[16..];
        let labels = &labels[8..];

        let images = images
            .chunks(28 * 28)
            .zip(labels.iter())
            .map(|(chunk, &label)| {
                let values = chunk
                    .iter()
                    .map(|&b| normalize_mnist_pixel(b))
                    .collect::<Vec<_>>();
                
                MnistItem {
                    image: values,
                    label: label as usize,
                }
            })
            .collect::<Vec<_>>();

        Self { images }
    }
    
    pub fn train() -> Self {
        Self::from_path("data/mnist")
    }
    
    pub fn test() -> Self {
        Self::from_path("data/mnist")
    }
}

impl Dataset<MnistItem> for MnistDataset {
    fn get(&self, index: usize) -> Option<MnistItem> {
        self.images.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.images.len()
    }
}

/// Build a `DataLoader` for MNIST training or testing data.
pub fn mnist_dataloader<B: Backend + 'static>(
    train: bool,
    device: &B::Device,
    batch_size: usize,
    shuffle: Option<u64>,
    num_workers: usize,
) -> Arc<dyn DataLoader<MnistBatch<B>>> {
    let dataset = if train {
        MnistDataset::train()
    } else {
        MnistDataset::test()
    };
    let batcher = MnistBatcher::new();
    let mut builder = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .num_workers(num_workers)
        .set_device(device.clone());
    
    if let Some(seed) = shuffle {
        builder = builder.shuffle(seed);
    }
    
    builder.build(dataset)
}
