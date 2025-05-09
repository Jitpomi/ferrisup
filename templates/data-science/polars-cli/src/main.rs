use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use comfy_table::{Cell, ContentArrangement, Table};
use polars::prelude::*;
use std::path::PathBuf;
use std::fs::File;
use rand::Rng;
{{#if (eq visualization "yes")}}
use plotters::prelude::*;
{{/if}}

/// A Rust CLI for data analysis using Polars (similar to pandas in Python)
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load and analyze a data file
    Analyze {
        /// Path to the data file (CSV, JSON, or Parquet)
        #[arg(short, long)]
        file: PathBuf,
        
        /// File format (csv, json, parquet)
        #[arg(short = 't', long, default_value = "{{#if (eq data_source "CSV files")}}csv{{else}}{{#if (eq data_source "Parquet files")}}parquet{{else}}{{#if (eq data_source "JSON data")}}json{{else}}csv{{/if}}{{/if}}{{/if}}")]
        format: String,
        
        {{#if (eq data_source "JSON data")}}
        /// JSON format (records, lines) - only used for JSON files
        #[arg(long, default_value = "records")]
        json_format: String,
        {{/if}}
        
        /// Optional column to filter on
        #[arg(short = 'c', long)]
        filter_column: Option<String>,
        
        /// Optional value to filter for
        #[arg(short = 'v', long)]
        filter_value: Option<String>,
        
        /// Optional column to group by
        #[arg(short = 'g', long)]
        group_by: Option<String>,
        
        /// Optional column to aggregate
        #[arg(short = 'a', long)]
        aggregate: Option<String>,
        
        /// Aggregation function (sum, mean, min, max, count)
        #[arg(short = 'u', long, default_value = "count")]
        agg_func: String,
        
        /// Perform statistical analysis
        #[arg(short, long)]
        stats: bool,
        
        /// Confidence level for statistical tests (0.90, 0.95, 0.99)
        #[arg(long, default_value = "0.95")]
        confidence: f64,
    },
    
    {{#if (eq visualization "yes")}}
    /// Create visualizations from data
    Visualize {
        /// Path to the data file (CSV, JSON, or Parquet)
        #[arg(short, long)]
        file: PathBuf,
        
        /// File format (csv, json, parquet)
        #[arg(short = 't', long, default_value = "{{#if (eq data_source "CSV files")}}csv{{else}}{{#if (eq data_source "Parquet files")}}parquet{{else}}{{#if (eq data_source "JSON data")}}json{{else}}csv{{/if}}{{/if}}{{/if}}")]
        format: String,
        
        {{#if (eq data_source "JSON data")}}
        /// JSON format (records, lines) - only used for JSON files
        #[arg(long, default_value = "records")]
        json_format: String,
        {{/if}}
        
        /// Column to visualize (must be numeric)
        #[arg(short = 'c', long)]
        column: String,
        
        /// Output file path (defaults to column_name_histogram.png)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    {{/if}}
    
    /// Generate a sample dataset
    Generate {
        /// Number of rows to generate
        #[arg(short, long, default_value_t = 100)]
        rows: usize,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Output format (csv, json, parquet)
        #[arg(short = 't', long, default_value = "{{#if (eq data_source "CSV files")}}csv{{else}}{{#if (eq data_source "Parquet files")}}parquet{{else}}{{#if (eq data_source "JSON data")}}json{{else}}csv{{/if}}{{/if}}{{/if}}")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyze {
            file,
            format,
            {{#if (eq data_source "JSON data")}}
            json_format,
            {{/if}}
            filter_column,
            filter_value,
            group_by,
            aggregate,
            agg_func,
            stats,
            confidence,
        } => {
            println!("📊 Loading data from {}: {}", format, file.display());
            
            // Read the data file based on format
            let df = match format.to_lowercase().as_str() {
                {{#if (eq data_source "JSON data")}}
                "json" => {
                    println!("Using JSON format: {}", json_format);
                    let json_fmt = match json_format.to_lowercase().as_str() {
                        "lines" => JsonFormat::JsonLines,
                        _ => JsonFormat::Json, // Use Json instead of JsonRecords in Polars 0.46.0
                    };
                    
                    let file = File::open(file)
                        .with_context(|| format!("Failed to open JSON file: {}", file.display()))?;
                    
                    JsonReader::new(file)
                        .with_json_format(json_fmt)
                        .finish()
                        .with_context(|| format!("Failed to read JSON file"))?
                },
                {{/if}}
                {{#if (eq data_source "Parquet files")}}
                "parquet" => {
                    let file = File::open(file)
                        .with_context(|| format!("Failed to open Parquet file: {}", file.display()))?;
                    
                    ParquetReader::new(file)
                        .finish()
                        .with_context(|| format!("Failed to read Parquet file"))?
                },
                {{/if}}
                _ => {
                    // Default to CSV
                    let file = File::open(file)
                        .with_context(|| format!("Failed to open CSV file: {}", file.display()))?;
                    
                    CsvReader::new(file)
                        .finish()
                        .with_context(|| "Failed to parse CSV data")?
                }
            };
            
            // Show basic info
            println!("\n📋 Data Overview:");
            println!("Rows: {}", df.height());
            println!("Columns: {}", df.width());
            println!("Column names: {:?}", df.get_column_names());
            
            // Apply filter if specified
            let filtered_df = if let (Some(col_name), Some(val)) = (filter_column, filter_value) {
                println!("\n🔍 Filtering where {} = {}", col_name, val);
                // In Polars 0.46.0, we need to use the correct filter syntax
                let filter_expr = col(col_name).eq(lit(val.as_str()));
                df.lazy().filter(filter_expr).collect()?
            } else {
                df.clone()
            };
            
            // Apply grouping and aggregation if specified
            let result_df = if let Some(group_col) = group_by {
                println!("\n📊 Grouping by: {}", group_col);
                
                let agg_col = aggregate.as_ref().unwrap_or(&group_col);
                println!("📊 Aggregating: {} using {}", agg_col, agg_func);
                
                let gb = filtered_df.lazy().group_by([col(group_col)]);
                
                match agg_func.as_str() {
                    "sum" => gb.agg([col(agg_col).sum()]).collect()?,
                    "mean" => gb.agg([col(agg_col).mean()]).collect()?,
                    "min" => gb.agg([col(agg_col).min()]).collect()?,
                    "max" => gb.agg([col(agg_col).max()]).collect()?,
                    _ => gb.agg([col(agg_col).count()]).collect()?,
                }
            } else {
                filtered_df
            };
            
            // Sort using the correct API for Polars 0.46.0
            let options = SortMultipleOptions {
                descending: vec![false],
                nulls_last: vec![true],
                maintain_order: false,
                multithreaded: true,
                limit: None,
            };
            let result_df = if let Some(group_col) = group_by {
                result_df.sort([group_col.to_string()], options)?
            } else {
                result_df
            };
            
            // Perform statistical analysis if requested
            if *stats {
                println!("\n📈 Statistical Analysis (Confidence Level: {}%):", confidence * 100.0);
                
                for col_name in result_df.get_column_names() {
                    if let Ok(col) = result_df.column(col_name) {
                        // Check if the column is numeric
                        if !matches!(col.dtype(), 
                            DataType::Int8 | DataType::Int16 | DataType::Int32 | 
                            DataType::Int64 | DataType::UInt8 | DataType::UInt16 | 
                            DataType::UInt32 | DataType::UInt64 | DataType::Float32 | 
                            DataType::Float64) {
                            
                            continue;
                        }
                        
                        println!("\nColumn: {}", col_name);
                        
                        // Calculate basic statistics manually
                        let series = match col {
                            Column::Series(s) => s,
                            _ => continue,
                        };
                        
                        // Get count
                        let count = series.len() as f64;
                        println!("  Count: {}", count);
                        
                        // Get min and max
                        if let Ok(Some(min_val)) = series.min::<f64>() {
                            println!("  Min: {}", min_val);
                        }
                        
                        if let Ok(Some(max_val)) = series.max::<f64>() {
                            println!("  Max: {}", max_val);
                        }
                        
                        // Get mean
                        let mean = if let Some(mean_val) = series.mean() {
                            println!("  Mean: {:.4}", mean_val);
                            mean_val
                        } else {
                            0.0
                        };
                        
                        // Get standard deviation
                        let std_dev = if let Some(std_val) = series.std(1) {
                            println!("  Std Dev: {:.4}", std_val);
                            std_val
                        } else {
                            0.0
                        };
                        
                        // Confidence interval
                        let z_score = match *confidence {
                            0.90 => 1.645,
                            0.99 => 2.576,
                            _ => 1.96, // 0.95 is default
                        };
                        
                        let margin_error = z_score * std_dev / count.sqrt();
                        println!("  {}% Confidence Interval: {:.4} ± {:.4}", 
                                 confidence * 100.0, mean, margin_error);
                                     
                        {{#if (eq visualization "yes")}}
                        // Generate a simple histogram for this column
                        if let Some(numeric_col) = series.cast(&DataType::Float64).ok() {
                            if let Ok(values) = numeric_col.f64() {
                                let output_path = file.with_file_name(format!(
                                    "{}_histogram_{}.png",
                                    file.file_stem().unwrap().to_string_lossy(),
                                    col_name
                                ));
                                
                                if let Err(e) = plot_histogram(&values, col_name, &output_path) {
                                    println!("  Warning: Could not generate histogram: {}", e);
                                } else {
                                    println!("  Histogram saved to: {}", output_path.display());
                                }
                            }
                        }
                        {{/if}}
                    }
                }
            }
            
            // Display results in a nice table
            print_dataframe(&result_df)?;
            
            // Save the result
            let output_path = file.with_file_name(format!(
                "{}_analyzed.{}",
                file.file_stem().unwrap().to_string_lossy(),
                match format.to_lowercase().as_str() {
                    {{#if (eq data_source "JSON data")}}
                    "json" => "json",
                    {{/if}}
                    {{#if (eq data_source "Parquet files")}}
                    "parquet" => "parquet",
                    {{/if}}
                    _ => "csv",
                }
            ));
            
            let mut output_file = File::create(&output_path)?;
            let mut result_df_mut = result_df.clone();
            
            match format.to_lowercase().as_str() {
                {{#if (eq data_source "JSON data")}}
                "json" => {
                    let json_fmt = match json_format.to_lowercase().as_str() {
                        "lines" => JsonFormat::JsonLines,
                        _ => JsonFormat::Json,
                    };
                    let mut json_writer = JsonWriter::new(&mut output_file)
                        .with_json_format(json_fmt);
                    json_writer.finish(&mut result_df_mut)?;
                },
                {{/if}}
                {{#if (eq data_source "Parquet files")}}
                "parquet" => {
                    let mut parquet_writer = ParquetWriter::new(&mut output_file);
                    parquet_writer.finish(&mut result_df_mut)?;
                },
                {{/if}}
                _ => {
                    // Default to CSV
                    let mut csv_writer = CsvWriter::new(&mut output_file);
                    csv_writer.finish(&mut result_df_mut)?;
                }
            }
            
            println!("\n💾 Results saved to: {}", output_path.display());
        }
        
        {{#if (eq visualization "yes")}}
        Commands::Visualize { file, format, {{#if (eq data_source "JSON data")}} json_format, {{/if}} column, output } => {
            println!("📊 Loading data from {}: {}", format, file.display());
            
            // Read the data file based on format
            let df = match format.to_lowercase().as_str() {
                {{#if (eq data_source "JSON data")}}
                "json" => {
                    println!("Using JSON format: {}", json_format);
                    let json_fmt = match json_format.to_lowercase().as_str() {
                        "lines" => JsonFormat::JsonLines,
                        _ => JsonFormat::Json, // Use Json instead of JsonRecords in Polars 0.46.0
                    };
                    
                    let file = File::open(file)
                        .with_context(|| format!("Failed to open JSON file: {}", file.display()))?;
                    
                    JsonReader::new(file)
                        .with_json_format(json_fmt)
                        .finish()
                        .with_context(|| format!("Failed to read JSON file"))?
                },
                {{/if}}
                {{#if (eq data_source "Parquet files")}}
                "parquet" => {
                    let file = File::open(file)
                        .with_context(|| format!("Failed to open Parquet file: {}", file.display()))?;
                    
                    ParquetReader::new(file)
                        .finish()
                        .with_context(|| format!("Failed to read Parquet file"))?
                },
                {{/if}}
                _ => {
                    // Default to CSV
                    let file = File::open(file)
                        .with_context(|| format!("Failed to open CSV file: {}", file.display()))?;
                    
                    CsvReader::new(file)
                        .finish()
                        .with_context(|| "Failed to parse CSV data")?
                }
            };
            
            // Check if the column exists
            let column_exists = df.get_column_names().iter().any(|c| c.as_str() == column);
            if !column_exists {
                println!("Error: Column '{}' does not exist in the data", column);
                return Ok(());
            }
            
            // Get the column
            let col = df.column(column).unwrap();
            
            // Check if the column is numeric
            if !matches!(col.dtype(), 
                DataType::Int8 | DataType::Int16 | DataType::Int32 | 
                DataType::Int64 | DataType::UInt8 | DataType::UInt16 | 
                DataType::UInt32 | DataType::UInt64 | DataType::Float32 | 
                DataType::Float64) {
                
                println!("Error: Column '{}' is not numeric", column);
                return Ok(());
            }
            
            // Get the output path
            let output_path = output.clone().unwrap_or_else(|| {
                file.with_file_name(format!("{}_histogram.png", column))
            });
            
            // Generate the histogram
            if let Some(numeric_col) = col.cast(&DataType::Float64).ok() {
                if let Ok(values) = numeric_col.f64() {
                    if let Err(e) = plot_histogram(&values, column, &output_path) {
                        println!("Error: Could not generate histogram: {}", e);
                    } else {
                        println!("Histogram saved to: {}", output_path.display());
                    }
                }
            }
        },
        {{/if}}
        
        Commands::Generate { rows, output, format } => {
            println!("🔄 Generating sample dataset with {} rows", rows);
            
            let mut rng = rand::thread_rng();
            let mut df = df! [
                "id" => (0..*rows as i32).collect::<Vec<_>>(),
                "name" => (0..*rows).map(|i| format!("Person_{}", i)).collect::<Vec<_>>(),
                "age" => (0..*rows).map(|_| rng.gen_range(20..100)).collect::<Vec<_>>(),
                "salary" => (0..*rows).map(|_| (rng.gen::<f32>() * 100000.0).round()).collect::<Vec<_>>(),
                "department" => (0..*rows).map(|i| {
                    match i % 5 {
                        0 => "Engineering",
                        1 => "Marketing",
                        2 => "Sales",
                        3 => "HR",
                        _ => "Finance",
                    }.to_string()
                }).collect::<Vec<_>>(),
                "date" => (0..*rows).map(|i| {
                    let days = i as i64 % 365;
                    chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                        .unwrap()
                        .checked_add_days(chrono::Days::new(days as u64))
                        .unwrap()
                        .format("%Y-%m-%d")
                        .to_string()
                }).collect::<Vec<_>>()
            ]?;
            
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            let mut file = File::create(output)?;
            
            match format.to_lowercase().as_str() {
                {{#if (eq data_source "JSON data")}}
                "json" => {
                    let mut json_writer = JsonWriter::new(&mut file)
                        .with_json_format(JsonFormat::Json);
                    json_writer.finish(&mut df)?;
                },
                {{/if}}
                {{#if (eq data_source "Parquet files")}}
                "parquet" => {
                    let mut parquet_writer = ParquetWriter::new(&mut file);
                    parquet_writer.finish(&mut df)?;
                },
                {{/if}}
                _ => {
                    // Default to CSV
                    let mut csv_writer = CsvWriter::new(&mut file);
                    csv_writer.finish(&mut df)?;
                }
            }
            
            println!("✅ Sample data saved to: {}", output.display());
            println!("\nTry analyzing it with:");
            println!("cargo run -- analyze -f {}", output.display());
        }
    }

    Ok(())
}

{{#if (eq visualization "yes")}}
fn plot_histogram(values: &ChunkedArray<Float64Type>, column_name: &str, output_path: &PathBuf) -> Result<()> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    // Find min and max for the x-axis range
    let min_val = values.min().unwrap_or(0.0);
    let max_val = values.max().unwrap_or(100.0);
    
    // Create bins for the histogram
    let bin_count = 10;
    let bin_width = (max_val - min_val) / bin_count as f64;
    
    let mut bins = vec![0; bin_count];
    let mut bin_labels = vec![];
    
    // Create bin labels
    for i in 0..bin_count {
        let lower = min_val + i as f64 * bin_width;
        let upper = min_val + (i + 1) as f64 * bin_width;
        bin_labels.push(format!("{:.1}-{:.1}", lower, upper));
    }
    
    // Count values in each bin
    for val in values.into_iter().flatten() {
        let bin_idx = ((val - min_val) / bin_width).floor() as usize;
        if bin_idx < bin_count {
            bins[bin_idx] += 1;
        }
    }
    
    // Find the maximum bin count for y-axis scaling
    let max_count = *bins.iter().max().unwrap_or(&1) as f64;
    
    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Histogram of {}", column_name), ("sans-serif", 22))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(
            0..bin_count,
            0.0..max_count * 1.1,
        )?;
    
    chart.configure_mesh()
        .x_labels(bin_count)
        .x_label_formatter(&|x| {
            let idx = *x as usize;
            if idx < bin_labels.len() {
                bin_labels[idx].clone()
            } else {
                "".to_string()
            }
        })
        .x_desc("Value")
        .y_desc("Count")
        .draw()?;
    
    chart.draw_series(
        Histogram::vertical(&chart)
            .style(BLUE.filled())
            .margin(0)
            .data(bins.iter().enumerate().map(|(i, c)| (i, *c as f64))),
    )?;
    
    root.present()?;
    
    Ok(())
}
{{/if}}

fn print_dataframe(df: &DataFrame) -> Result<()> {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    
    let headers: Vec<Cell> = df.get_column_names()
        .iter()
        .map(|name| Cell::new(name))
        .collect();
    table.set_header(headers);
    
    let max_rows = std::cmp::min(df.height(), 20);
    for row_idx in 0..max_rows {
        let row_cells: Vec<Cell> = df.get_column_names()
            .iter()
            .map(|col_name| {
                let col = df.column(col_name).unwrap();
                let cell_value = match col.dtype() {
                    DataType::Float32 | DataType::Float64 => {
                        format!("{:.2}", col.get(row_idx).unwrap())
                    },
                    _ => format!("{}", col.get(row_idx).unwrap()),
                };
                Cell::new(cell_value)
            })
            .collect();
        table.add_row(row_cells);
    }
    
    println!("\n📊 Results:");
    println!("{table}");
    
    if df.height() > max_rows {
        println!("(Showing {} of {} rows)", max_rows, df.height());
    }
    
    Ok(())
}
