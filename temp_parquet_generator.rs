use polars::prelude::*;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple DataFrame with sample data
    let df = df! [
        "id" => [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "name" => ["Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Hannah", "Ian", "Julia"],
        "department" => ["Engineering", "Sales", "Marketing", "Engineering", "HR", "Sales", "Marketing", "Engineering", "Finance", "HR"],
        "salary" => [75000, 65000, 60000, 78000, 55000, 68000, 62000, 79000, 72000, 56000],
        "age" => [28, 35, 42, 31, 29, 38, 33, 27, 45, 36]
    ]?;

    println!("DataFrame created successfully");
    println!("{}", df);

    // Create a Parquet file
    let mut file = File::create("templates/data-science/polars-cli/data/example_data_parquet.parquet")?;
    ParquetWriter::new(&mut file).finish(&df)?;

    println!("Parquet file created successfully");
    
    Ok(())
}
