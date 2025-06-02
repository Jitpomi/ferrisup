# FerrisUp Data Science Templates

This directory contains various templates for data science and machine learning projects in Rust.

## Available Templates

### Data Analysis with Polars
- Located in `polars-cli/`
- Fast data processing and analysis using the Polars DataFrame library
- Support for CSV, JSON, and Parquet files
- Includes visualizations with plotters

### Machine Learning with Linfa
- Located in `linfa-examples/` and `linfa-lab/`
- Statistical analysis and machine learning with Rust's Linfa ecosystem
- Classification, clustering, and regression models
- Includes datasets and examples

### Deep Learning with Burn
- Located in `burn-image-recognition/` and `burn-image-classifier/`
- Neural network training and inference using Burn
- Support for image processing, text processing, and numerical data
- Includes models for MNIST and CIFAR-10 datasets

## Getting Started

Select a template using FerrisUp:

```
ferrisup new my-project --template data-science
```

Then follow the prompts to select a specific data science approach and configuration options.

## Dependencies

Most templates require:
- Rust stable (1.68.0+)
- wasm32-unknown-unknown target (for web visualizations)

Specific templates may have additional dependencies, which will be installed automatically or prompted during setup.
