use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use std::path::PathBuf;
use tracing::{info, Level};

mod analysis;

/// FerrisUp Data Science Template CLI
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input data file path (CSV, JSON, or Parquet)
    #[clap(short, long, value_parser)]
    input: PathBuf,
    
    /// Output file for results
    #[clap(short, long, value_parser)]
    output: Option<PathBuf>,
    
    /// Type of analysis to perform
    #[clap(short, long, default_value = "summary")]
    analysis: String,
    
    /// Columns to include in analysis (comma-separated)
    #[clap(short, long)]
    columns: Option<String>,
    
    /// Generate visualizations
    #[clap(short, long)]
    visualize: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    // Parse command line arguments
    let args = Args::parse();
    
    info!("Starting data analysis on file: {:?}", args.input);
    
    // Load dataset 
    let df = load_dataset(&args.input)?;
    info!("Dataset loaded with shape: {} rows × {} columns", df.height(), df.width());
    
    // Parse columns to include
    let columns = args.columns.as_ref()
        .map(|cols| cols.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_else(Vec::new);
    
    // Perform analysis
    let result = match args.analysis.as_str() {
        "summary" => analysis::summarize(&df, &columns)?,
        "correlation" => analysis::calculate_correlations(&df, &columns)?,
        "cluster" => analysis::cluster_data(&df, &columns)?,
        "timeseries" => analysis::analyze_timeseries(&df, &columns)?,
        _ => return Err(anyhow::anyhow!("Unknown analysis type: {}", args.analysis)),
    };
    
    // Print results to console
    println!("{}", result);
    
    // Save results if output is specified
    if let Some(output_path) = args.output {
        info!("Saving analysis results to {:?}", output_path);
        std::fs::write(&output_path, result)?;
    }
    
    // Generate visualizations if requested
    if args.visualize {
        info!("Generating visualizations");
        let viz_path = args.output
            .map(|p| p.with_extension("png"))
            .unwrap_or_else(|| PathBuf::from("visualization.png"));
            
        analysis::visualize(&df, &columns, &viz_path)?;
        info!("Visualization saved to {:?}", viz_path);
    }
    
    info!("Analysis completed successfully");
    Ok(())
}

/// Load a dataset from various file formats
fn load_dataset(path: &PathBuf) -> Result<DataFrame> {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    let df = match extension.to_lowercase().as_str() {
        "csv" => {
            let file = std::fs::File::open(path)?;
            CsvReader::new(file)
                .infer_schema(Some(100))
                .has_header(true)
                .finish()?
        },
        "json" => {
            let file = std::fs::File::open(path)?;
            JsonReader::new(file)
                .finish()?
        },
        "parquet" => {
            let file = std::fs::File::open(path)?;
            ParquetReader::new(file)
                .finish()?
        },
        _ => return Err(anyhow::anyhow!("Unsupported file format: {}", extension)),
    };
    
    Ok(df)
}
