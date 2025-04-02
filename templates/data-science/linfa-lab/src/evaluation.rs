use anyhow::Result;
use linfa::metrics::{ConfusionMatrix, PredictionError};
use linfa::traits::Predict;
use ndarray::{Array1, Array2, ArrayBase, Data, Ix2};
use plotters::prelude::*;
use std::path::Path;

/// Regression metrics
pub struct RegressionMetrics {
    pub mse: f64,
    pub rmse: f64,
    pub mae: f64,
    pub r2: f64,
}

/// Classification metrics
pub struct ClassificationMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub confusion_matrix: ConfusionMatrix<usize>,
}

/// Clustering metrics
pub struct ClusteringMetrics {
    pub silhouette: f64,
    pub inertia: f64,
    pub n_clusters: usize,
}

/// Evaluate a regression model
pub fn evaluate_regression<'a, P>(
    model: P,
    test_dataset: linfa::Dataset<f64, f64>,
) -> Result<RegressionMetrics>
where
    P: Predict<Array1<f64>, Output = Array1<f64>> + 'a,
{
    let predictions = model.predict(&test_dataset.records);
    let targets = &test_dataset.targets;
    
    // Calculate metrics
    let n_samples = targets.len() as f64;
    
    // Mean Squared Error
    let mse = predictions
        .iter()
        .zip(targets.iter())
        .map(|(pred, actual)| (pred - actual).powi(2))
        .sum::<f64>() / n_samples;
    
    // Root Mean Squared Error
    let rmse = mse.sqrt();
    
    // Mean Absolute Error
    let mae = predictions
        .iter()
        .zip(targets.iter())
        .map(|(pred, actual)| (pred - actual).abs())
        .sum::<f64>() / n_samples;
    
    // R-squared
    let mean_target = targets.mean().unwrap();
    let total_sum_squares = targets
        .iter()
        .map(|&actual| (actual - mean_target).powi(2))
        .sum::<f64>();
    let residual_sum_squares = predictions
        .iter()
        .zip(targets.iter())
        .map(|(pred, actual)| (pred - actual).powi(2))
        .sum::<f64>();
    let r2 = 1.0 - (residual_sum_squares / total_sum_squares);
    
    Ok(RegressionMetrics {
        mse,
        rmse,
        mae,
        r2,
    })
}

/// Evaluate a classification model
pub fn evaluate_classification<'a, P>(
    model: P,
    test_dataset: linfa::Dataset<f64, usize>,
) -> Result<ClassificationMetrics>
where
    P: Predict<Array1<usize>, Output = Array1<usize>> + 'a,
{
    let predictions = model.predict(&test_dataset.records);
    let targets = &test_dataset.targets;
    
    // Calculate confusion matrix
    let cm = ConfusionMatrix::new(&predictions, targets)?;
    
    // Calculate metrics
    let accuracy = cm.accuracy();
    
    // Calculate precision, recall, and F1 for each class and average
    let mut total_precision = 0.0;
    let mut total_recall = 0.0;
    let mut total_f1 = 0.0;
    let n_classes = cm.classes().len();
    
    for class in cm.classes() {
        let precision = cm.precision_for_class(*class);
        let recall = cm.recall_for_class(*class);
        let f1 = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };
        
        total_precision += precision;
        total_recall += recall;
        total_f1 += f1;
    }
    
    let precision = total_precision / n_classes as f64;
    let recall = total_recall / n_classes as f64;
    let f1 = total_f1 / n_classes as f64;
    
    Ok(ClassificationMetrics {
        accuracy,
        precision,
        recall,
        f1,
        confusion_matrix: cm,
    })
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

/// Plot regression results
pub fn plot_regression_results(
    test_dataset: linfa::Dataset<f64, f64>,
    metrics: &RegressionMetrics,
    output_path: &str,
) -> Result<()> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let predictions = test_dataset.records.column(0).to_owned();
    let targets = &test_dataset.targets;
    
    let min_val = predictions
        .iter()
        .chain(targets.iter())
        .fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = predictions
        .iter()
        .chain(targets.iter())
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "Regression Results (R² = {:.4}, RMSE = {:.4})",
                metrics.r2, metrics.rmse
            ),
            ("sans-serif", 20).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            min_val..max_val,
            min_val..max_val,
        )?;
    
    chart.configure_mesh().draw()?;
    
    // Draw the perfect prediction line
    chart.draw_series(LineSeries::new(
        vec![(min_val, min_val), (max_val, max_val)],
        &BLACK.mix(0.5),
    ))?;
    
    // Draw the predictions
    chart.draw_series(
        predictions
            .iter()
            .zip(targets.iter())
            .map(|(&pred, &actual)| Circle::new((pred, actual), 3, &BLUE.mix(0.5))),
    )?;
    
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    
    root.present()?;
    
    Ok(())
}

/// Plot confusion matrix
pub fn plot_confusion_matrix(
    test_dataset: linfa::Dataset<f64, usize>,
    metrics: &ClassificationMetrics,
    output_path: &str,
) -> Result<()> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let cm = &metrics.confusion_matrix;
    let classes = cm.classes();
    let n_classes = classes.len();
    
    let max_count = cm.counts()
        .iter()
        .flatten()
        .fold(0, |a, &b| a.max(b));
    
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "Confusion Matrix (Accuracy = {:.2}%)",
                metrics.accuracy * 100.0
            ),
            ("sans-serif", 20).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            0..n_classes,
            0..n_classes,
        )?;
    
    chart.configure_mesh()
        .disable_mesh()
        .x_labels(n_classes)
        .y_labels(n_classes)
        .x_label_formatter(&|v| format!("{}", classes[*v]))
        .y_label_formatter(&|v| format!("{}", classes[*v]))
        .x_desc("Predicted")
        .y_desc("Actual")
        .draw()?;
    
    let cm_counts = cm.counts();
    
    // Draw the confusion matrix cells
    for i in 0..n_classes {
        for j in 0..n_classes {
            let count = cm_counts[i][j];
            let color = if i == j {
                RGBColor(100, 200, 100)
            } else {
                RGBColor(200, 100, 100)
            };
            
            let opacity = count as f64 / max_count as f64;
            
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(j, i), (j + 1, i + 1)],
                    color.mix(opacity).filled(),
                )
            ))?;
            
            chart.draw_series(std::iter::once(
                Text::new(
                    format!("{}", count),
                    (j + 0.5, i + 0.5),
                    ("sans-serif", 15).into_font(),
                )
            ))?;
        }
    }
    
    root.present()?;
    
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
    
    // Define colors for different clusters
    let colors = [
        &RED, &GREEN, &BLUE, &CYAN, &MAGENTA, &YELLOW,
        &RED.mix(0.5), &GREEN.mix(0.5), &BLUE.mix(0.5),
        &CYAN.mix(0.5), &MAGENTA.mix(0.5), &YELLOW.mix(0.5),
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
    
    // Define colors for different targets
    let colors = [
        &RED, &GREEN, &BLUE, &CYAN, &MAGENTA, &YELLOW,
        &RED.mix(0.5), &GREEN.mix(0.5), &BLUE.mix(0.5),
        &CYAN.mix(0.5), &MAGENTA.mix(0.5), &YELLOW.mix(0.5),
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
