use anyhow::Result;
use linfa::metrics::ToConfusionMatrix;
use linfa::traits::Predict;
use ndarray::{Array1, Array2, ArrayBase, ArrayView1, ArrayView2, Data, Ix1};
use plotters::prelude::*;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::PathBuf;

/// Regression metrics
pub struct RegressionMetrics {
    pub mse: f64,
    pub rmse: f64,
    pub mae: f64,
    pub r2: f64,
}

/// Classification metrics
#[derive(Debug, Clone)]
pub struct ClassificationMetrics {
    pub accuracy: f64,
    pub precision_weighted: f64,
    pub recall_weighted: f64,
    pub f1_weighted: f64,
}

/// Clustering metrics
pub struct ClusteringMetrics {
    pub silhouette: f64,
    pub inertia: f64,
    pub n_clusters: usize,
}

/// Evaluate a regression model
pub fn evaluate_regression(
    model: &Box<dyn Predict<Array2<f64>, Array1<f64>>>,
    test_features: &ArrayView2<'_, f64>,
    test_targets: &ArrayView1<'_, f64>,
) -> Result<RegressionMetrics> {
    // Make predictions
    let predictions = model.predict(test_features.to_owned());
    
    // Calculate metrics
    let n = test_targets.len() as f64;
    
    // Calculate mean of actual values
    let mean_actual = test_targets.mean().unwrap();
    
    // Calculate R²
    let ss_total = test_targets.iter()
        .map(|&y| (y - mean_actual).powi(2))
        .sum::<f64>();
    
    let ss_residual = test_targets.iter()
        .zip(predictions.iter())
        .map(|(&y_true, &y_pred)| (y_true - y_pred).powi(2))
        .sum::<f64>();
    
    let r2 = 1.0 - (ss_residual / ss_total);
    
    // Calculate MAE
    let mae = test_targets.iter()
        .zip(predictions.iter())
        .map(|(&y_true, &y_pred)| (y_true - y_pred).abs())
        .sum::<f64>() / n;
    
    // Calculate MSE
    let mse = test_targets.iter()
        .zip(predictions.iter())
        .map(|(&y_true, &y_pred)| (y_true - y_pred).powi(2))
        .sum::<f64>() / n;
    
    // Calculate RMSE
    let rmse = mse.sqrt();
    
    Ok(RegressionMetrics { r2, mae, mse, rmse })
}

/// Evaluate a classification model
pub fn evaluate_classification<S>(
    targets: ArrayView1<'_, usize>,
    predictions: &ArrayBase<S, Ix1>,
    _test_features: ArrayView2<'_, f64>,
) -> Result<ClassificationMetrics> 
where
    S: Data<Elem = usize>,
{
    // Create confusion matrix
    let cm = ConfusionMatrix::new(targets, predictions.view())?;
    
    // Calculate metrics
    let accuracy = cm.accuracy();
    let precision_weighted = cm.precision_macro()?;
    let recall_weighted = cm.recall_macro()?;
    let f1_weighted = cm.f1_macro()?;
    
    Ok(ClassificationMetrics {
        accuracy,
        precision_weighted,
        recall_weighted,
        f1_weighted,
    })
}

/// Plot a confusion matrix
pub fn plot_confusion_matrix<'a, S>(
    targets: ArrayView1<'_, usize>,
    predictions: &ArrayBase<S, Ix1>,
    output_path: &str,
) -> Result<()> 
where
    S: Data<Elem = usize>,
{
    // Create confusion matrix
    let cm = ConfusionMatrix::new(targets, predictions.view())?;
    
    // Get the number of classes
    let n_classes = cm.classes();
    
    // Create a markdown file with the confusion matrix
    let mut output = File::create(output_path)?;
    
    // Add header
    output.write_fmt(format_args!("# Classification Report\n\n"))?;
    
    // Add confusion matrix
    output.write_fmt(format_args!("## Confusion Matrix\n\n"))?;
    output.write_fmt(format_args!("|   | "))?;
    
    // Add column headers (predicted classes)
    for i in 0..n_classes {
        output.write_fmt(format_args!("Pred {} | ", i))?;
    }
    output.write_fmt(format_args!("\n"))?;
    
    // Add separator
    output.write_fmt(format_args!("|---| "))?;
    for _ in 0..n_classes {
        output.write_fmt(format_args!("--------| "))?;
    }
    output.write_fmt(format_args!("\n"))?;
    
    // Add rows
    for i in 0..n_classes {
        output.write_fmt(format_args!("| True {} | ", i))?;
        
        for j in 0..n_classes {
            let count = cm.get(i, j);
            output.write_fmt(format_args!("{} | ", count))?;
        }
        output.write_fmt(format_args!("\n"))?;
    }
    
    // Add per-class metrics
    output.write_fmt(format_args!("\n## Per-Class Metrics\n"))?;
    output.write_fmt(format_args!("| Class | Precision | Recall | F1 Score | Support |\n"))?;
    output.write_fmt(format_args!("|-------|-----------|--------|----------|---------|\n"))?;
    
    for i in 0..n_classes {
        // In Linfa 0.7.1, we need to use the precision/recall methods without the _for_class suffix
        let precision = cm.precision(i).unwrap_or(0.0);
        let recall = cm.recall(i).unwrap_or(0.0);
        let f1 = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };
        
        // Calculate support (number of actual samples in this class)
        let mut support = 0;
        for j in 0..n_classes {
            // The confusion matrix is [actual, predicted]
            support += cm.get(i, j);
        }
        
        output.write_fmt(format_args!("| {} | {:.4} | {:.4} | {:.4} | {} |\n", 
                                     i, precision, recall, f1, support))?;
    }
    
    // Add overall metrics
    output.write_fmt(format_args!("\n## Overall Metrics\n"))?;
    output.write_fmt(format_args!("- Accuracy: {:.4}\n", cm.accuracy()))?;
    output.write_fmt(format_args!("- Precision (weighted): {:.4}\n", cm.precision_macro()?))?;
    output.write_fmt(format_args!("- Recall (weighted): {:.4}\n", cm.recall_macro()?))?;
    output.write_fmt(format_args!("- F1 Score (weighted): {:.4}\n", cm.f1_macro()?))?;
    
    Ok(())
}

/// Plot regression results
pub fn plot_regression_results(
    _test_features: &ArrayView2<'_, f64>,
    test_targets: &ArrayView1<'_, f64>,
    predictions: &Array1<f64>,
    metrics: &RegressionMetrics,
    output_path: &PathBuf,
) -> Result<()> {
    // For now, we'll just create a simple text file with the results
    let mut file = File::create(output_path)?;
    
    writeln!(file, "# Regression Results")?;
    writeln!(file, "")?;
    writeln!(file, "## Metrics")?;
    writeln!(file, "")?;
    writeln!(file, "| Metric | Value |")?;
    writeln!(file, "|--------|-------|")?;
    writeln!(file, "| R² Score | {:.4} |", metrics.r2)?;
    writeln!(file, "| Mean Absolute Error | {:.4} |", metrics.mae)?;
    writeln!(file, "| Mean Squared Error | {:.4} |", metrics.mse)?;
    writeln!(file, "| Root Mean Squared Error | {:.4} |", metrics.rmse)?;
    
    // Add a table of actual vs predicted values (first 10 samples)
    writeln!(file, "")?;
    writeln!(file, "## Sample Predictions (First 10)")?;
    writeln!(file, "")?;
    writeln!(file, "| Sample | Actual | Predicted | Error |")?;
    writeln!(file, "|--------|--------|-----------|-------|")?;
    
    let n_samples = test_targets.len().min(10);
    for i in 0..n_samples {
        let actual = test_targets[i];
        let predicted = predictions[i];
        let error = actual - predicted;
        writeln!(file, "| {} | {:.4} | {:.4} | {:.4} |", i + 1, actual, predicted, error)?;
    }
    
    // Add error distribution information
    writeln!(file, "")?;
    writeln!(file, "## Error Distribution")?;
    writeln!(file, "")?;
    
    let errors: Vec<f64> = test_targets.iter()
        .zip(predictions.iter())
        .map(|(&y_true, &y_pred)| y_true - y_pred)
        .collect();
    
    let min_error = errors.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_error = errors.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;
    
    writeln!(file, "| Statistic | Value |")?;
    writeln!(file, "|-----------|-------|")?;
    writeln!(file, "| Min Error | {:.4} |", min_error)?;
    writeln!(file, "| Max Error | {:.4} |", max_error)?;
    writeln!(file, "| Mean Error | {:.4} |", mean_error)?;
    
    Ok(())
}

/// Plot clustering results
pub fn plot_clustering(
    data: &Array2<f64>,
    clusters: &Array1<usize>,
    centroids: &Array2<f64>,
    output_path: &str,
) -> Result<()> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    // Use the first two dimensions for plotting
    let x_dim = 0;
    let y_dim = if data.ncols() > 1 { 1 } else { 0 };
    
    let x_min = data.column(x_dim).fold(f64::INFINITY, |a, &b| a.min(b));
    let x_max = data.column(x_dim).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let y_min = data.column(y_dim).fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = data.column(y_dim).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    let x_range = x_min..x_max;
    let y_range = y_min..y_max;
    
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Clustering Results ({} clusters)", centroids.nrows()),
            ("sans-serif", 20).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_range, y_range)?;
    
    chart.configure_mesh().draw()?;
    
    // Create a custom color palette
    let colors = [
        &RGBColor(200, 100, 100),
        &RGBColor(100, 200, 100),
        &RGBColor(100, 100, 200),
        &RGBColor(200, 200, 100),
        &RGBColor(100, 200, 200),
        &RGBColor(200, 100, 200),
        &RGBColor(150, 100, 100),
        &RGBColor(100, 150, 100),
        &RGBColor(100, 100, 150),
        &RGBColor(150, 150, 100),
        &RGBColor(100, 150, 150),
        &RGBColor(150, 100, 150),
    ];
    
    // Draw data points
    for cluster_id in 0..centroids.nrows() {
        let color = colors[cluster_id % colors.len()];
        
        chart.draw_series(
            data.rows()
                .into_iter()
                .zip(clusters.iter())
                .filter(|(_, &c)| c == cluster_id)
                .map(|(row, _)| {
                    Circle::new(
                        (row[x_dim], row[y_dim]),
                        3,
                        color.filled(),
                    )
                }),
        )?;
    }
    
    // Draw noise points (for DBSCAN)
    chart.draw_series(
        data.rows()
            .into_iter()
            .zip(clusters.iter())
            .filter(|(_, &c)| c == usize::MAX)
            .map(|(row, _)| {
                Circle::new(
                    (row[x_dim], row[y_dim]),
                    3,
                    BLACK.mix(0.3).filled(),
                )
            }),
    )?;
    
    // Draw centroids
    chart.draw_series(
        centroids.rows()
            .into_iter()
            .enumerate()
            .map(|(i, row)| {
                let color = colors[i % colors.len()];
                Cross::new(
                    (row[x_dim], row[y_dim]),
                    6,
                    color.filled(),
                )
            }),
    )?;
    
    root.present()?;
    
    Ok(())
}

/// Plot dimensionality reduction results
pub fn plot_reduction<T: std::fmt::Display>(
    data: &Array2<f64>,
    targets: &Array1<T>,
    output_path: &str,
) -> Result<()> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    // Use the first two dimensions for plotting
    let x_dim = 0;
    let y_dim = if data.ncols() > 1 { 1 } else { 0 };
    
    let x_min = data.column(x_dim).fold(f64::INFINITY, |a, &b| a.min(b));
    let x_max = data.column(x_dim).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let y_min = data.column(y_dim).fold(f64::INFINITY, |a, &b| a.min(b));
    let y_max = data.column(y_dim).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    let x_range = x_min..x_max;
    let y_range = y_min..y_max;
    
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Dimensionality Reduction Results",
            ("sans-serif", 20).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_range, y_range)?;
    
    chart.configure_mesh().draw()?;
    
    // Get unique targets
    let mut unique_targets = Vec::new();
    for target in targets.iter() {
        let target_str = format!("{}", target);
        if !unique_targets.contains(&target_str) {
            unique_targets.push(target_str);
        }
    }
    
    // Create a custom color palette
    let colors = [
        &RGBColor(200, 100, 100),
        &RGBColor(100, 200, 100),
        &RGBColor(100, 100, 200),
        &RGBColor(200, 200, 100),
        &RGBColor(100, 200, 200),
        &RGBColor(200, 100, 200),
        &RGBColor(150, 100, 100),
        &RGBColor(100, 150, 100),
        &RGBColor(100, 100, 150),
        &RGBColor(150, 150, 100),
        &RGBColor(100, 150, 150),
        &RGBColor(150, 100, 150),
    ];
    
    // Draw data points
    for (i, target_str) in unique_targets.iter().enumerate() {
        let color = colors[i % colors.len()];
        
        chart.draw_series(
            data.rows()
                .into_iter()
                .zip(targets.iter())
                .filter(|(_, target)| format!("{}", target) == *target_str)
                .map(|(row, _)| {
                    Circle::new(
                        (row[x_dim], row[y_dim]),
                        3,
                        color.filled(),
                    )
                }),
        )?
        .label(target_str)
        .legend(move |(x, y)| Circle::new((x, y), 3, color.filled()));
    }
    
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    
    root.present()?;
    
    Ok(())
}

/// Evaluate clustering results
pub fn evaluate_clustering(
    data: &Array2<f64>,
    clusters: &Array1<usize>,
    centroids: &Array2<f64>,
) -> Result<ClusteringMetrics> {
    // Calculate inertia (sum of squared distances to closest centroid)
    let mut inertia = 0.0;
    for (i, point) in data.rows().into_iter().enumerate() {
        let cluster = clusters[i];
        if cluster != usize::MAX { // Skip noise points (for DBSCAN)
            let centroid = centroids.row(cluster);
            let squared_dist = point
                .iter()
                .zip(centroid.iter())
                .map(|(&p, &c)| (p - c).powi(2))
                .sum::<f64>();
            inertia += squared_dist;
        }
    }
    
    // Calculate silhouette score
    // This is a simplified version that works for most cases
    let n_samples = data.nrows();
    let mut silhouette_sum = 0.0;
    let mut valid_samples = 0;
    
    for i in 0..n_samples {
        let cluster_i = clusters[i];
        if cluster_i == usize::MAX { // Skip noise points
            continue;
        }
        
        // Calculate a (mean distance to points in same cluster)
        let mut a_sum = 0.0;
        let mut a_count = 0;
        for j in 0..n_samples {
            if i != j && clusters[j] == cluster_i {
                let dist = data.row(i)
                    .iter()
                    .zip(data.row(j).iter())
                    .map(|(&p1, &p2)| (p1 - p2).powi(2))
                    .sum::<f64>()
                    .sqrt();
                a_sum += dist;
                a_count += 1;
            }
        }
        let a = if a_count > 0 { a_sum / a_count as f64 } else { 0.0 };
        
        // Calculate b (mean distance to points in nearest cluster)
        let mut b = f64::INFINITY;
        for c in 0..centroids.nrows() {
            if c != cluster_i {
                let mut b_sum = 0.0;
                let mut b_count = 0;
                for j in 0..n_samples {
                    if clusters[j] == c {
                        let dist = data.row(i)
                            .iter()
                            .zip(data.row(j).iter())
                            .map(|(&p1, &p2)| (p1 - p2).powi(2))
                            .sum::<f64>()
                            .sqrt();
                        b_sum += dist;
                        b_count += 1;
                    }
                }
                if b_count > 0 {
                    let cluster_b = b_sum / b_count as f64;
                    b = b.min(cluster_b);
                }
            }
        }
        
        // Calculate silhouette
        if a_count > 0 && b < f64::INFINITY {
            let s = (b - a) / a.max(b);
            silhouette_sum += s;
            valid_samples += 1;
        }
    }
    
    let silhouette = if valid_samples > 0 {
        silhouette_sum / valid_samples as f64
    } else {
        0.0
    };
    
    // Count unique clusters (excluding noise points)
    let n_clusters = clusters
        .iter()
        .filter(|&&c| c != usize::MAX)
        .collect::<std::collections::HashSet<_>>()
        .len();
    
    Ok(ClusteringMetrics {
        silhouette,
        inertia,
        n_clusters,
    })
}

struct DummyClassifier {}

impl Predict<Array2<f64>, Array1<usize>> for DummyClassifier {
    fn predict(&self, features: Array2<f64>) -> Array1<usize> {
        Array1::zeros(features.nrows())
    }
}
