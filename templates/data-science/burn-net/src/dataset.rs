use anyhow::Result;
use burn::data::dataset::vision::MnistDataset;
use burn::data::{dataloader::DataLoaderBuilder, dataset::Dataset};
use burn::tensor::{backend::Backend, Data, Tensor};
use image::{DynamicImage, GenericImageView, Pixel};
use std::path::Path;

/// MNIST item with image and label
#[derive(Clone)]
pub struct MnistItem<B: Backend> {
    pub image: Tensor<B, 4>,
    pub label: Tensor<B, 1, usize>,
}

/// Load the MNIST dataset
pub fn load_mnist<B: Backend>(
    batch_size: usize,
) -> Result<(
    burn::data::dataloader::DataLoader<MnistItem<B>, Vec<MnistItem<B>>>,
    burn::data::dataloader::DataLoader<MnistItem<B>, Vec<MnistItem<B>>>,
)> {
    // Download and load the dataset
    let dataset_train = MnistDataset::train()?;
    let dataset_test = MnistDataset::test()?;

    // Create dataloaders
    let dataloader_train = DataLoaderBuilder::new(dataset_train)
        .batch_size(batch_size)
        .shuffle(true)
        .build_for_iter(|items| {
            let batch_images = items
                .iter()
                .map(|item| {
                    let image = Tensor::<B, 3>::from_data(Data::from(item.image.clone()))
                        .reshape([1, 28, 28])
                        .divide_scalar(255.0);
                    image
                })
                .collect::<Vec<_>>();

            let batch_labels = items
                .iter()
                .map(|item| Tensor::<B, 1, usize>::from_data(Data::from([item.label as usize])))
                .collect::<Vec<_>>();

            batch_images
                .into_iter()
                .zip(batch_labels)
                .map(|(image, label)| MnistItem { image, label })
                .collect()
        });

    let dataloader_test = DataLoaderBuilder::new(dataset_test)
        .batch_size(batch_size)
        .shuffle(false)
        .build_for_iter(|items| {
            let batch_images = items
                .iter()
                .map(|item| {
                    let image = Tensor::<B, 3>::from_data(Data::from(item.image.clone()))
                        .reshape([1, 28, 28])
                        .divide_scalar(255.0);
                    image
                })
                .collect::<Vec<_>>();

            let batch_labels = items
                .iter()
                .map(|item| Tensor::<B, 1, usize>::from_data(Data::from([item.label as usize])))
                .collect::<Vec<_>>();

            batch_images
                .into_iter()
                .zip(batch_labels)
                .map(|(image, label)| MnistItem { image, label })
                .collect()
        });

    Ok((dataloader_train, dataloader_test))
}

/// Load a single image for prediction
pub fn load_image<B: Backend, P: AsRef<Path>>(path: P) -> Result<Tensor<B, 4>> {
    let img = image::open(path)?;
    
    // Convert to grayscale and resize to 28x28
    let img = img.resize_exact(28, 28, image::imageops::FilterType::Lanczos3);
    let img = img.grayscale();
    
    // Convert to tensor
    let mut data = vec![0.0; 28 * 28];
    for (i, pixel) in img.pixels().enumerate() {
        let [r, _, _, _] = pixel.2.0;
        data[i] = r as f32 / 255.0;
    }
    
    let tensor = Tensor::<B, 3>::from_data(Data::from(data))
        .reshape([1, 28, 28]);
    
    // Add batch dimension
    let tensor = tensor.unsqueeze::<4>(0);
    
    Ok(tensor)
}
