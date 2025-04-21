use polars::prelude::*;
use std::fs::File;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating sample Parquet file...");
    
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

    // Ensure the target directory exists
    let target_dir = Path::new("../templates/data-science/polars-cli/data");
    std::fs::create_dir_all(target_dir)?;
    
    // Create a Parquet file
    let target_path = target_dir.join("example_data_parquet.parquet");
    let mut file = File::create(&target_path)?;
    ParquetWriter::new(&mut file).finish(&df)?;

    println!("Parquet file created successfully at {}", target_path.display());
    
    Ok(())
}
