# Linfa Machine Learning Examples

This project contains a collection of machine learning examples using the [Linfa](https://github.com/rust-ml/linfa) framework in Rust. Linfa is a Rust machine learning framework that provides functionality similar to scikit-learn in Python.

## Examples Included

This template includes the following machine learning examples:

1. **Classification with LogisticRegression**
   - Binary classification example using LogisticRegression
   - Demonstrates training, prediction, and evaluation with confusion matrix

2. **Classification with DecisionTree**
   - Binary classification example using DecisionTree
   - Shows how to set model parameters and evaluate performance

3. **Regression with LinearRegression**
   - Simple linear regression example (y = mx + b)
   - Demonstrates parameter estimation and prediction on new data

4. **Clustering with DBSCAN**
   - Density-based clustering of synthetic data
   - Shows how to generate synthetic clustered data and apply DBSCAN

## Getting Started

To run the examples, use the following commands:

```bash
# Run all examples
cargo run -- all

# Run specific examples
cargo run -- classify  # LogisticRegression classification
cargo run -- tree      # DecisionTree classification
cargo run -- regress   # LinearRegression
cargo run -- cluster   # DBSCAN clustering

# Show help information
cargo run -- help
```

## Project Structure

- `src/main.rs` - Main entry point with command-line interface
- `src/datasets.rs` - Dataset loading and processing utilities
- `src/models.rs` - Model creation and training utilities
- `src/evaluation.rs` - Evaluation metrics and visualization

## Dependencies

This project uses Linfa 0.7.1 and its various components:

- `linfa` - Core framework
- `linfa-linear` - Linear regression
- `linfa-logistic` - Logistic regression
- `linfa-trees` - Decision trees
- `linfa-clustering` - Clustering algorithms including DBSCAN
- `ndarray` - N-dimensional arrays for data manipulation

## Extending the Examples

You can extend these examples by:

1. Adding more complex datasets (e.g., from CSV files)
2. Implementing cross-validation
3. Adding hyperparameter tuning
4. Combining multiple models in an ensemble
5. Adding visualization of results

## Resources

- [Linfa Documentation](https://docs.rs/linfa/)
- [Linfa GitHub Repository](https://github.com/rust-ml/linfa)
- [ndarray Documentation](https://docs.rs/ndarray/)
