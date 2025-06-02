use polars::prelude::*;
use rand::Rng;
use chrono::{NaiveDate, Duration};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a random number generator
    let mut rng = rand::thread_rng();
    
    // Generate sample data
    let n_rows = 100;
    
    // Create id column
    let id_values: Vec<i32> = (1..=n_rows).collect();
    let id_series = Series::new("id", id_values);
    
    // Create name column
    let name_values: Vec<String> = (1..=n_rows).map(|i| format!("Item {}", i)).collect();
    let name_series = Series::new("name", name_values);
    
    // Create value column (random float values)
    let value_values: Vec<f64> = (0..n_rows).map(|_| rng.gen::<f64>() * 1000.0).collect();
    let value_series = Series::new("value", value_values);
    
    // Create category column
    let categories = ["A", "B", "C", "D"];
    let category_values: Vec<&str> = (0..n_rows)
        .map(|_| categories[rng.gen_range(0..categories.len())])
        .collect();
    let category_series = Series::new("category", category_values);
    
    // Create date column
    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let date_values: Vec<i64> = (0..n_rows)
        .map(|i| {
            let date = start_date + Duration::days(i as i64);
            date.and_hms_opt(0, 0, 0).unwrap().timestamp()
        })
        .collect();
    let date_series = Series::new("date", date_values);
    
    // Create is_active column
    let is_active_values: Vec<bool> = (0..n_rows)
        .map(|_| rng.gen_bool(0.5))
        .collect();
    let is_active_series = Series::new("is_active", is_active_values);
    
    // Create DataFrame
    let df = DataFrame::new(vec![
        id_series, name_series, value_series, category_series, date_series, is_active_series
    ])?;
    
    // Save as Parquet file
    let file = File::create("example_data_parquet.parquet")?;
    ParquetWriter::new(file).finish(&df)?;
    
    println!("Sample Parquet file created successfully!");
    
    Ok(())
}
