# Linfa Machine Learning Examples

This project contains working machine learning examples using the [Linfa](https://github.com/rust-ml/linfa) framework in Rust. These examples are specifically designed to work with Linfa 0.7.1 and demonstrate key machine learning workflows.

## Working Examples Included

This template includes the following tested and working machine learning examples:

1. **Classification with LogisticRegression**
   - Binary classification example using LogisticRegression
   - Creates synthetic data, trains a model, and evaluates with confusion matrix
   - Demonstrates the complete workflow from data to predictions

2. **Classification with DecisionTree**
   - Binary classification example using DecisionTree
   - Shows how to set model parameters and evaluate performance
   - Includes proper API usage for Linfa 0.7.1

3. **Regression with LinearRegression**
   - Simple linear regression example (y = mx + b)
   - Demonstrates parameter estimation and prediction on new data
   - Shows how to calculate metrics like MSE

4. **Clustering with DBSCAN**
   - Density-based clustering of synthetic data
   - Shows how to generate synthetic clustered data and apply DBSCAN
   - Includes proper cluster membership handling

## Running the Examples

To run the examples, use the following commands:

```bash
# Run all examples
cargo run -- all

# Run specific examples
cargo run -- classify  # LogisticRegression classification
cargo run -- tree      # DecisionTree classification
cargo run -- regress   # LinearRegression
cargo run -- cluster   # DBSCAN clustering

# Get help
cargo run -- help
```

## Project Structure

- `src/main.rs` - Main entry point with command-line interface
- `src/classification.rs` - LogisticRegression classification example
- `src/decision_tree.rs` - DecisionTree classification example
- `src/regression.rs` - LinearRegression example
- `src/clustering.rs` - DBSCAN clustering example

## Real-World Applications

Each example can be expanded to solve real-world problems:

### LogisticRegression Classification
- **Fraud Detection**: Expand to classify transactions as fraudulent or legitimate using features like transaction amount, time, location, and user history.
- **Medical Diagnosis**: Adapt to predict disease presence based on patient symptoms and test results.
- **Customer Churn Prediction**: Modify to predict which customers are likely to cancel a subscription based on usage patterns.

### DecisionTree Classification
- **Credit Risk Assessment**: Expand to evaluate loan applications by predicting default risk based on applicant data.
- **Email Spam Filtering**: Adapt to classify emails as spam or legitimate based on content features.
- **Plant Disease Diagnosis**: Modify to identify plant diseases from image-derived features of leaves or stems.

### LinearRegression
- **House Price Prediction**: Expand to predict real estate prices based on features like square footage, location, and number of rooms.
- **Sales Forecasting**: Adapt to predict future sales based on historical data and seasonal factors.
- **Crop Yield Prediction**: Modify to estimate agricultural yields based on weather conditions, soil quality, and farming practices.

### DBSCAN Clustering
- **Customer Segmentation**: Expand to group customers based on purchasing behavior for targeted marketing.
- **Anomaly Detection**: Adapt to identify unusual patterns in network traffic that might indicate security threats.
- **Geographic Analysis**: Modify to identify population centers or points of interest from geospatial data.

## Implementation Steps for Real-World Applications

To adapt these examples to real-world problems:

1. **Data Loading and Preprocessing**:
   ```rust
   // Add code to load data from CSV or other sources
   let data = csv::Reader::from_path("your_data.csv")?
       .deserialize()
       .collect::<Result<Vec<YourDataType>, _>>()?;
       
   // Add preprocessing steps like normalization
   let normalized_features = preprocess_features(features);
   ```

2. **Feature Engineering**:
   ```rust
   // Create new features from existing ones
   let engineered_features = engineer_features(raw_features);
   
   // Select most important features
   let selected_features = select_features(engineered_features);
   ```

3. **Cross-Validation**:
   ```rust
   // Implement k-fold cross-validation
   let k_folds = 5;
   let mut metrics = Vec::new();
   
   for fold in 0..k_folds {
       let (train, test) = create_fold(dataset, fold, k_folds);
       let model = train_model(&train);
       let fold_metrics = evaluate_model(&model, &test);
       metrics.push(fold_metrics);
   }
   ```

4. **Hyperparameter Tuning**:
   ```rust
   // Grid search for optimal parameters
   let c_values = vec![0.1, 1.0, 10.0];
   let max_iter_values = vec![100, 500, 1000];
   
   let mut best_params = None;
   let mut best_score = f64::NEG_INFINITY;
   
   for &c in &c_values {
       for &max_iter in &max_iter_values {
           let model = LogisticRegression::default()
               .c(c)
               .max_iterations(max_iter)
               .fit(&train_dataset)?;
           
           let score = evaluate_model(&model, &validation_dataset);
           if score > best_score {
               best_score = score;
               best_params = Some((c, max_iter));
           }
       }
   }
   ```

5. **Model Persistence**:
   ```rust
   // Save trained model to file
   let model_bytes = bincode::serialize(&model)?;
   std::fs::write("model.bin", model_bytes)?;
   
   // Load model for inference
   let model_bytes = std::fs::read("model.bin")?;
   let model: YourModelType = bincode::deserialize(&model_bytes)?;
   ```

## Multi-Format Data Support

This project supports loading and saving datasets in CSV, JSON, and Parquet formats.

### Generating Sample Data

You can generate synthetic data for testing in CSV, JSON, and Parquet formats:

```bash
# Generate classification data in all formats
cargo run -- generate classification all

# Generate regression data in CSV format only
cargo run -- generate regression csv

# Generate clustering data in JSON format only
cargo run -- generate clustering json

# Generate clustering data in Parquet format only
cargo run -- generate clustering parquet
```

### Data Format Priority

When running examples, the data is loaded in the following priority order:

1. CSV files (e.g., `data/sample_classification.csv`)
2. JSON files (e.g., `data/sample_classification.json`)
3. Parquet files (e.g., `data/sample_classification.parquet`)
4. If no data files are found, synthetic data is generated in memory

To force the use of a specific format, make sure only that format's file exists in the data directory.

## Data Format Specifications

### CSV Format

- Classification/Regression: `x,y,target` columns
- Clustering: `x,y` columns

### JSON Format

- Classification/Regression:
  ```json
  {
    "data": [
      {"x": 1.0, "y": 2.0, "target": 0.0},
      {"x": 3.0, "y": 4.0, "target": 1.0}
    ]
  }
  ```

- Clustering:
  ```json
  {
    "data": [
      {"x": 1.0, "y": 2.0},
      {"x": 3.0, "y": 4.0}
    ]
  }
  ```

### Parquet Format

- Classification/Regression: Contains columns `x`, `y`, and `target`
- Clustering: Contains columns `x` and `y`

## Dependencies

This project uses Linfa 0.7.1 and its various components:

- `linfa` - Core framework
- `linfa-linear` - Linear regression
- `linfa-logistic` - Logistic regression
- `linfa-trees` - Decision trees
- `linfa-clustering` - Clustering algorithms including DBSCAN
- `ndarray` - N-dimensional arrays for data manipulation
- `rand_xoshiro` (version 0.6.0) - Required for compatibility with Linfa 0.7.1

## Extending the Examples

You can extend these examples by:

1. Adding more complex datasets (e.g., from CSV files)
2. Implementing cross-validation
3. Adding hyperparameter tuning
4. Combining multiple models in an ensemble
5. Adding visualization of results

## Troubleshooting

If you encounter any issues:

1. Make sure you're using Linfa 0.7.1 as specified in Cargo.toml
2. Verify that rand_xoshiro is version 0.6.0 (critical for compatibility)
3. Check array dimensions in your data (Linfa expects specific shapes)

## Resources

- [Linfa Documentation](https://docs.rs/linfa/)
- [Linfa GitHub Repository](https://github.com/rust-ml/linfa)
- [ndarray Documentation](https://docs.rs/ndarray/)
