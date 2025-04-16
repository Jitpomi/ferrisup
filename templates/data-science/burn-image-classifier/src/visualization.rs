// Visualization utilities for the image classifier
// This file contains functions for visualizing model predictions and training history

use plotters::prelude::*;
use image::DynamicImage;
use crate::error::Result;

/// Plot training history (loss and accuracy)
pub fn plot_training_history(
    train_losses: &[f64],
    valid_losses: &[f64],
    train_accuracies: &[f64],
    valid_accuracies: &[f64],
    output_path: &str,
) -> Result<()> {
    // Create a drawing area
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    // Split into two panels
    let (upper, lower) = root.split_vertically(300);
    
    // Plot loss
    {
        let max_loss: f64 = train_losses.iter()
            .chain(valid_losses.iter())
            .fold(0.0f64, |a, &b| a.max(b));
            
        let min_loss: f64 = train_losses.iter()
            .chain(valid_losses.iter())
            .fold(f64::MAX, |a, &b| a.min(b));
            
        let mut chart = ChartBuilder::on(&upper)
            .caption("Training Loss", ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(
                0.0f64..(train_losses.len() as f64),
                (min_loss * 0.9f64)..(max_loss * 1.1f64),
            )?;
            
        chart.configure_mesh()
            .x_desc("Epoch")
            .y_desc("Loss")
            .draw()?;
            
        // Draw training loss
        chart.draw_series(LineSeries::new(
            train_losses.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            &RED,
        ))?
        .label("Training")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
        
        // Draw validation loss
        chart.draw_series(LineSeries::new(
            valid_losses.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            &BLUE,
        ))?
        .label("Validation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        
        // Add a legend
        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
            
        // Add text labels
        chart.draw_series(PointSeries::of_element(
            vec![(0.0f64, train_losses[0])],
            5,
            &RED,
            &|coord, _size, _style| {
                EmptyElement::at(coord)
                    + Text::new(
                        "Train".to_string(),
                        (10, 0),
                        ("sans-serif", 15).into_font().color(&RED),
                    )
            },
        ))?;
        
        chart.draw_series(PointSeries::of_element(
            vec![(0.0f64, valid_losses[0])],
            5,
            &BLUE,
            &|coord, _size, _style| {
                EmptyElement::at(coord)
                    + Text::new(
                        "Valid".to_string(),
                        (10, 15),
                        ("sans-serif", 15).into_font().color(&BLUE),
                    )
            },
        ))?;
    }
    
    // Plot accuracy
    {
        let max_acc: f64 = train_accuracies.iter()
            .chain(valid_accuracies.iter())
            .fold(0.0f64, |a, &b| a.max(b));
            
        let min_acc: f64 = train_accuracies.iter()
            .chain(valid_accuracies.iter())
            .fold(1.0f64, |a, &b| a.min(b));
            
        let mut chart = ChartBuilder::on(&lower)
            .caption("Training Accuracy", ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(
                0.0f64..(train_accuracies.len() as f64),
                (min_acc * 0.9f64)..(max_acc * 1.1f64),
            )?;
            
        chart.configure_mesh()
            .x_desc("Epoch")
            .y_desc("Accuracy")
            .draw()?;
            
        // Draw training accuracy
        chart.draw_series(LineSeries::new(
            train_accuracies.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            &RED,
        ))?
        .label("Training")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
        
        // Draw validation accuracy
        chart.draw_series(LineSeries::new(
            valid_accuracies.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            &BLUE,
        ))?
        .label("Validation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));
        
        // Add a legend
        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
            
        // Add text labels
        chart.draw_series(PointSeries::of_element(
            vec![(0.0f64, train_accuracies[0])],
            5,
            &RED,
            &|coord, _size, _style| {
                EmptyElement::at(coord)
                    + Text::new(
                        "Train".to_string(),
                        (10, 0),
                        ("sans-serif", 15).into_font().color(&RED),
                    )
            },
        ))?;
        
        chart.draw_series(PointSeries::of_element(
            vec![(0.0f64, valid_accuracies[0])],
            5,
            &BLUE,
            &|coord, _size, _style| {
                EmptyElement::at(coord)
                    + Text::new(
                        "Valid".to_string(),
                        (10, 15),
                        ("sans-serif", 15).into_font().color(&BLUE),
                    )
            },
        ))?;
    }
    
    Ok(())
}

/// Plot model predictions for an image
pub fn plot_predictions(
    img: &DynamicImage,
    class_indices: &[usize],
    probabilities: &[f32],
    output_path: &str,
) -> Result<()> {
    // Create a drawing area
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    // Split into two panels
    let (left, right) = root.split_horizontally(400);
    
    // Draw the image on the left panel
    {
        let img_resized = img.resize_exact(380, 380, image::imageops::FilterType::Lanczos3);
        let (width, height) = (img_resized.width() as usize, img_resized.height() as usize);
        
        let mut chart = ChartBuilder::on(&left)
            .caption("Input Image", ("sans-serif", 20).into_font())
            .margin(10)
            .x_label_area_size(10)
            .y_label_area_size(10)
            .build_cartesian_2d(0f64..(width as f64), 0f64..(height as f64))?;
            
        let img_rgb = img_resized.to_rgb8();
        
        chart.draw_series(
            (0..height).flat_map(move |y| {
                let img_rgb = img_rgb.clone(); // Clone for each row
                (0..width).map(move |x| {
                    let pixel = img_rgb.get_pixel(x as u32, y as u32);
                    let color = RGBColor(pixel[0], pixel[1], pixel[2]);
                    Rectangle::new([(x as f64, y as f64), ((x+1) as f64, (y+1) as f64)], color.filled())
                })
            })
        )?;
    }
    
    // Draw the predictions on the right panel
    {
        let max_classes = class_indices.len().min(5); // Show at most 5 classes
        
        let mut chart = ChartBuilder::on(&right)
            .caption("Predictions", ("sans-serif", 20).into_font())
            .margin(30)
            .x_label_area_size(30)
            .y_label_area_size(100)
            .build_cartesian_2d(0f32..1f32, 0f64..(max_classes as f64))?;
            
        chart.configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .y_desc("Class")
            .x_desc("Probability")
            .draw()?;
            
        // Draw bars for each prediction
        for (i, (&class_idx, &prob)) in class_indices.iter().zip(probabilities.iter()).take(max_classes).enumerate() {
            let class_name = format!("Class {}", class_idx);
            let color = HSLColor(0.3 * i as f64 / max_classes as f64, 0.7, 0.5);
            
            // Draw the bar
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(0f32, i as f64), (prob, (i + 1) as f64)],
                    color.filled(),
                )
            ))?
            .label(&class_name)
            .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled()));
            
            // Add text label with probability percentage
            chart.draw_series(std::iter::once(
                Text::new(
                    format!("{:.1}%", prob * 100.0),
                    (prob + 0.02f32, i as f64 + 0.5),
                    ("sans-serif", 15).into_font(),
                )
            ))?;
        }
        
        // Add a legend
        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .position(SeriesLabelPosition::UpperRight)
            .draw()?;
    }
    
    Ok(())
}

/// Plot a confusion matrix
#[allow(dead_code)]
pub fn plot_confusion_matrix(
    matrix: &[Vec<usize>],
    class_names: &[String],
    output_path: &str,
) -> Result<()> {
    let num_classes = matrix.len();
    
    // Create a drawing area
    let root = BitMapBackend::new(output_path, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;
    
    // Find the maximum value in the matrix for color scaling
    let max_value = matrix.iter()
        .flat_map(|row| row.iter())
        .fold(0, |a, &b| a.max(b));
    
    // Create a chart
    let mut chart = ChartBuilder::on(&root)
        .caption("Confusion Matrix", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..(num_classes as f64), 0f64..(num_classes as f64))?;
    
    chart.configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Predicted Class")
        .y_desc("True Class")
        .x_labels(num_classes)
        .y_labels(num_classes)
        .x_label_formatter(&|x| {
            let idx = *x as usize;
            if idx < class_names.len() {
                class_names[idx].clone()
            } else {
                format!("Class {}", idx)
            }
        })
        .y_label_formatter(&|y| {
            let idx = *y as usize;
            if idx < class_names.len() {
                class_names[idx].clone()
            } else {
                format!("Class {}", idx)
            }
        })
        .draw()?;
    
    // Draw the matrix cells
    for (i, row) in matrix.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            // Calculate color intensity based on value
            let intensity: f64 = if max_value > 0 {
                value as f64 / max_value as f64
            } else {
                0.0
            };
            
            // Use a color gradient from white to blue
            let color = RGBColor(
                (255.0 * (1.0 - intensity)) as u8,
                (255.0 * (1.0 - intensity)) as u8,
                255,
            );
            
            // Draw the cell
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(j as f64, i as f64), ((j + 1) as f64, (i + 1) as f64)],
                    color.filled(),
                )
            ))?;
            
            // Add the value as text
            chart.draw_series(std::iter::once(
                Text::new(
                    format!("{}", value),
                    (j as f64 + 0.5, i as f64 + 0.5),
                    ("sans-serif", 20).into_font().color(if intensity > 0.5 { &WHITE } else { &BLACK }),
                )
            ))?;
        }
    }
    
    Ok(())
}

/// Accuracy metric for tracking model performance
pub struct Accuracy<B> {
    correct: usize,
    total: usize,
    _phantom: std::marker::PhantomData<B>,
}

impl<B> Accuracy<B> {
    /// Create a new accuracy metric
    pub fn new() -> Self {
        Self {
            correct: 0,
            total: 0,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Reset the metric
    pub fn reset(&mut self) {
        self.correct = 0;
        self.total = 0;
    }
    
    /// Add a correct prediction
    pub fn add_correct(&mut self) {
        self.correct += 1;
        self.total += 1;
    }
    
    /// Add an incorrect prediction
    pub fn add_incorrect(&mut self) {
        self.total += 1;
    }
    
    /// Compute the accuracy
    pub fn compute(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.correct as f64 / self.total as f64
        }
    }
}
