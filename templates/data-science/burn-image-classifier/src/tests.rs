#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::backend::Backend;
    use burn_ndarray::NdArrayBackend;
    use std::path::Path;
    use image::DynamicImage;

    // Type alias for the backend we'll use in tests
    type B = NdArrayBackend<f32>;

    #[test]
    fn test_model_creation() {
        // Test that we can create a model with the default configuration
        let config = config::ImageClassifierConfig::default();
        let model = model::ImageClassifierModel::<B>::new(&config);
        
        // Check that the model has the correct number of output classes
        assert_eq!(model.num_classes(), config::NUM_CLASSES);
    }

    #[test]
    fn test_model_forward_pass() {
        // Create a model
        let model = model::ImageClassifierModel::<B>::default();
        
        // Create a dummy input tensor with the correct shape
        let batch_size = 2;
        let input = burn::tensor::Tensor::<B, 4>::zeros(
            [batch_size, config::NUM_CHANNELS, config::IMAGE_SIZE, config::IMAGE_SIZE]
        );
        
        // Run a forward pass
        let output = model.forward(input);
        
        // Check output shape
        assert_eq!(output.shape()[0], batch_size);
        assert_eq!(output.shape()[1], config::NUM_CLASSES);
    }

    #[test]
    fn test_sample_dataset_creation() {
        // Create a small sample dataset
        let samples_per_class = 5;
        let dataset = data::create_sample_dataset(samples_per_class).unwrap();
        
        // Check that we have the correct number of samples
        assert_eq!(dataset.len(), samples_per_class * config::NUM_CLASSES);
        
        // Check that we have the correct number of classes
        assert_eq!(dataset.num_classes(), config::NUM_CLASSES);
        
        // Check that the class names match
        for (i, name) in dataset.class_names().iter().enumerate() {
            assert_eq!(name, &config::CLASS_NAMES[i].to_string());
        }
    }

    #[test]
    fn test_dataset_splitting() {
        // Create a small sample dataset
        let samples_per_class = 10;
        let dataset = data::create_sample_dataset(samples_per_class).unwrap();
        
        // Split the dataset with 80% training, 20% validation
        let train_ratio = 0.8;
        let (train_dataset, val_dataset) = dataset.split_by_ratio(train_ratio);
        
        // Calculate expected sizes
        let total_samples = samples_per_class * config::NUM_CLASSES;
        let expected_train_size = (total_samples as f32 * train_ratio) as usize;
        let expected_val_size = total_samples - expected_train_size;
        
        // Check that the split sizes are correct
        assert_eq!(train_dataset.len(), expected_train_size);
        assert_eq!(val_dataset.len(), expected_val_size);
    }

    #[test]
    fn test_image_processing() {
        // Create a test image
        let width = config::IMAGE_SIZE as u32;
        let height = config::IMAGE_SIZE as u32;
        let img_buffer = image::RgbImage::new(width, height);
        let img = DynamicImage::ImageRgb8(img_buffer);
        
        // Convert to tensor data
        let tensor_data = data::image_to_tensor(&img);
        
        // Check tensor size
        assert_eq!(tensor_data.len(), config::IMAGE_SIZE * config::IMAGE_SIZE * config::NUM_CHANNELS);
    }

    #[test]
    fn test_training_history() {
        // Create a new training history
        let mut history = model::TrainingHistory::new();
        
        // Add some epochs
        history.add_epoch(1, 0.5, 0.4, 0.7);
        history.add_epoch(2, 0.3, 0.25, 0.8);
        
        // Check that the history contains the correct data
        assert_eq!(history.epochs, vec![1, 2]);
        assert_eq!(history.train_loss, vec![0.5, 0.3]);
        assert_eq!(history.val_loss, vec![0.4, 0.25]);
        assert_eq!(history.val_accuracy, vec![0.7, 0.8]);
        
        // Test saving and loading (if temp directory is available)
        if let Ok(temp_dir) = std::env::temp_dir().canonicalize() {
            let file_path = temp_dir.join("test_history.json");
            let file_path_str = file_path.to_str().unwrap();
            
            // Save the history
            history.save(file_path_str).unwrap();
            
            // Check that the file exists
            assert!(Path::new(file_path_str).exists());
            
            // Load the history
            let loaded_history = model::TrainingHistory::load(file_path_str).unwrap();
            
            // Check that the loaded history matches the original
            assert_eq!(loaded_history.epochs, history.epochs);
            assert_eq!(loaded_history.train_loss, history.train_loss);
            assert_eq!(loaded_history.val_loss, history.val_loss);
            assert_eq!(loaded_history.val_accuracy, history.val_accuracy);
            
            // Clean up
            std::fs::remove_file(file_path).unwrap_or(());
        }
    }
}
