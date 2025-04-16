// Custom error types for the image classifier
// This provides better error handling and more informative error messages

use thiserror::Error;
use std::io;
use std::path::PathBuf;
use image::ImageError;
use plotters::drawing::DrawingAreaErrorKind;
use std::error::Error as StdError;

#[derive(Error, Debug)]
pub enum ImageClassifierError {
    // Currently used error variants
    #[error("Failed to read directory {path:?}: {source}")]
    DirectoryReadError {
        path: PathBuf,
        source: io::Error,
    },

    #[error("No images found in directory {0}")]
    EmptyDirectoryError(PathBuf),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Image error: {0}")]
    ImageError(#[from] ImageError),
    
    #[error("Plotting error: {0}")]
    PlottingError(String),

    // Commented out unused error variants - kept for future use
    /*
    #[error("Failed to load image from {path:?}: {source}")]
    ImageLoadError {
        path: PathBuf,
        source: image::ImageError,
    },

    #[error("No class directories found in {0}")]
    NoClassDirectoriesError(PathBuf),

    #[error("Failed to save model: {0}")]
    ModelSaveError(String),

    #[error("Failed to load model: {0}")]
    ModelLoadError(String),

    #[error("Invalid image dimensions: expected {expected:?}, got {actual:?}")]
    InvalidImageDimensions {
        expected: (usize, usize),
        actual: (usize, usize),
    },

    #[error("Failed to create dataset: {0}")]
    DatasetCreationError(String),

    #[error("Failed to process image: {0}")]
    ImageProcessingError(String),

    #[error("Failed to save visualization: {0}")]
    VisualizationError(String),
    
    #[error("Dataset error: {0}")]
    DatasetError(String),
    
    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),
    
    #[error("No class directories found in {0}")]
    NoClassDirectories(PathBuf),
    
    #[error("No images found in {0}")]
    NoImagesFound(PathBuf),
    */
}

// Implement From trait for plotters error types
impl<E: StdError + std::fmt::Debug + std::marker::Send + std::marker::Sync + 'static> From<DrawingAreaErrorKind<E>> for ImageClassifierError {
    fn from(_: DrawingAreaErrorKind<E>) -> Self {
        ImageClassifierError::PlottingError("Drawing error".to_string())
    }
}

/// Result type for the image classifier
pub type Result<T> = std::result::Result<T, ImageClassifierError>;
