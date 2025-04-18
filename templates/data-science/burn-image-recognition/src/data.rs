use burn::data::dataloader::{batcher::Batcher, DataLoader, DataLoaderBuilder};
use burn::data::dataset::vision::{MnistDataset, MnistItem};
use burn::prelude::*;
use std::sync::Arc;

/// Normalize a single MNIST pixel value (u8 or f32) to f32 with PyTorch stats.
pub fn normalize_mnist_pixel<T: Into<f32>>(pixel: T) -> f32 {
    ((pixel.into() / 255.0) - 0.1307) / 0.3081
}

/// A batch of MNIST images and labels.
#[derive(Clone, Debug)]
pub struct MnistBatch<B: Backend> {
    /// A batch of images as a 3D tensor [batch, 28, 28].
    pub images: Tensor<B, 3>,
    /// A batch of labels as a 1D int tensor [batch].
    pub targets: Tensor<B, 1, Int>,
}

/// A batcher for MNIST data.
#[derive(Clone, Debug)]
pub struct MnistBatcher<B: Backend> {
    /// The target device (CPU, CUDA, etc.) for the tensors.
    device: B::Device,
}

impl<B: Backend> MnistBatcher<B> {
    /// Create a new `MnistBatcher` instance.
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
}

impl<B: Backend> Batcher<MnistItem, MnistBatch<B>> for MnistBatcher<B> {
    /// Batch a list of `MnistItem`s into a `MnistBatch`.
    fn batch(&self, items: Vec<MnistItem>) -> MnistBatch<B> {
        let _batch_size = items.len();
        // Convert each image into a Tensor<B, 2> and stack them into Tensor<B, 3>
        let image_tensors: Vec<_> = items.iter().map(|item| {
            // Flatten [[f32; 28]; 28] to &[f32] for from_data
            let flat: Vec<f32> = item.image.iter().flat_map(|row| row.iter().copied()).collect();
            let data = TensorData::new(flat, Shape::new([28, 28]));
            Tensor::<B, 2>::from_data(data, &self.device)
        }).collect();
        let images = Tensor::stack(image_tensors, 0);
        let targets: Vec<i64> = items.iter().map(|item| item.label as i64).collect();
        let targets = Tensor::<B, 1, Int>::from_data(TensorData::from(targets.as_slice()), &self.device);
        MnistBatch { images, targets }
    }
}

/// Build a `DataLoader` for MNIST training or testing data.
pub fn mnist_dataloader<B: Backend + 'static>(
    train: bool,
    device: B::Device,
    batch_size: usize,
    shuffle: Option<u64>,
    num_workers: usize,
) -> Arc<dyn DataLoader<MnistBatch<B>>> {
    let dataset = if train {
        MnistDataset::train()
    } else {
        MnistDataset::test()
    };
    let batcher = MnistBatcher::<B>::new(device);
    let mut builder = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .num_workers(num_workers);
    if let Some(seed) = shuffle {
        builder = builder.shuffle(seed);
    }
    builder.build(dataset)
}
