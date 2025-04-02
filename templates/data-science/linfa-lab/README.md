# Linfa-Lab - Machine Learning in Rust

This project provides a foundation for machine learning in Rust using [Linfa](https://github.com/rust-ml/linfa), a Rust ML framework similar to scikit-learn in Python.

## Features

- **Classification**: Logistic regression, decision trees, random forests, SVM, and k-NN
- **Regression**: Linear regression, ridge, lasso, elastic net, decision trees, and random forests
- **Clustering**: K-means, DBSCAN, and Gaussian mixture models
- **Dimensionality Reduction**: PCA and t-SNE
- **Dataset Handling**: Load built-in datasets (Iris, Diabetes, Wine Quality) or custom CSV files
- **Evaluation**: Calculate metrics and generate plots for model performance
- **Data Generation**: Create synthetic datasets for experimentation

## Getting Started

### Classification

```bash
# Train a logistic regression model on the Iris dataset
cargo run -- classification -d iris -m logistic

# Train a random forest on a custom dataset
cargo run -- classification -d custom -f data.csv -t target_column -m random_forest
```

### Regression

```bash
# Train a linear regression model on the Diabetes dataset
cargo run -- regression -d diabetes -m linear

# Train an elastic net model with custom test size and seed
cargo run -- regression -d winequality -m elasticnet -t 0.3 -s 123
```

### Clustering

```bash
# Perform K-means clustering on the Iris dataset
cargo run -- clustering -d iris -a kmeans -n 3

# Use DBSCAN on a custom dataset
cargo run -- clustering -d custom -f data.csv -a dbscan
```

### Dimensionality Reduction

```bash
# Perform PCA on the Wine Quality dataset
cargo run -- reduction -d winequality -a pca -n 2

# Use t-SNE on a custom dataset
cargo run -- reduction -d custom -f data.csv -a tsne -n 3
```

### Data Generation

```bash
# Generate a classification dataset
cargo run -- generate -t classification -n 1000 -f 10 -c 3 -o data.csv

# Generate a regression dataset
cargo run -- generate -t regression -n 500 -f 5 -o regression_data.csv
```

## Model Comparison

| Task | Model | Description |
|------|-------|-------------|
| Classification | `logistic` | Logistic regression (similar to scikit-learn's LogisticRegression) |
| Classification | `decision_tree` | Decision tree classifier (similar to scikit-learn's DecisionTreeClassifier) |
| Classification | `random_forest` | Random forest classifier (similar to scikit-learn's RandomForestClassifier) |
| Classification | `svm` | Support vector machine (similar to scikit-learn's SVC) |
| Classification | `knn` | K-nearest neighbors (similar to scikit-learn's KNeighborsClassifier) |
| Regression | `linear` | Linear regression (similar to scikit-learn's LinearRegression) |
| Regression | `ridge` | Ridge regression (similar to scikit-learn's Ridge) |
| Regression | `lasso` | Lasso regression (similar to scikit-learn's Lasso) |
| Regression | `elasticnet` | Elastic net regression (similar to scikit-learn's ElasticNet) |
| Regression | `decision_tree` | Decision tree regressor (similar to scikit-learn's DecisionTreeRegressor) |
| Regression | `random_forest` | Random forest regressor (similar to scikit-learn's RandomForestRegressor) |
| Regression | `knn` | K-nearest neighbors regressor (similar to scikit-learn's KNeighborsRegressor) |
| Clustering | `kmeans` | K-means clustering (similar to scikit-learn's KMeans) |
| Clustering | `dbscan` | DBSCAN clustering (similar to scikit-learn's DBSCAN) |
| Clustering | `gaussian_mixture` | Gaussian mixture model (similar to scikit-learn's GaussianMixture) |
| Reduction | `pca` | Principal component analysis (similar to scikit-learn's PCA) |
| Reduction | `tsne` | t-SNE (similar to scikit-learn's TSNE) |

## Extending

This template provides a foundation for machine learning in Rust. To extend it:

1. **Add new models**: Implement additional Linfa algorithms in `src/models.rs`
2. **Work with different datasets**: Add more dataset loaders in `src/datasets.rs`
3. **Improve visualizations**: Enhance the plotting capabilities in `src/evaluation.rs`
4. **Add hyperparameter tuning**: Implement grid search or random search for model selection

## Resources

- [Linfa Documentation](https://github.com/rust-ml/linfa)
- [Rust ML Working Group](https://github.com/rust-ml)
- [ndarray Documentation](https://docs.rs/ndarray/)
