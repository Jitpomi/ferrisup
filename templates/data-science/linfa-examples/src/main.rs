use anyhow::Result;
use std::env;

mod clustering;
mod regression;
mod classification;
mod decision_tree;

fn print_usage() {
    println!("Linfa 0.7.1 Examples");
    println!("Usage: cargo run -- [example]");
    println!("Available examples:");
    println!("  classify    - Run classification example with LogisticRegression");
    println!("  cluster     - Run clustering example with DBSCAN");
    println!("  regress     - Run regression example with LinearRegression");
    println!("  tree        - Run classification example with Decision Tree");
    println!("  all         - Run all examples sequentially");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "classify" => classification::run_logistic_regression_example()?,
        "cluster" => clustering::run_dbscan_example()?,
        "regress" => regression::run_regression_example()?,
        "tree" => decision_tree::run_decision_tree_example()?,
        "all" => {
            println!("\n=== Running LogisticRegression Classification Example ===\n");
            classification::run_logistic_regression_example()?;
            
            println!("\n=== Running Decision Tree Classification Example ===\n");
            decision_tree::run_decision_tree_example()?;
            
            println!("\n=== Running Linear Regression Example ===\n");
            regression::run_regression_example()?;
            
            println!("\n=== Running DBSCAN Clustering Example ===\n");
            clustering::run_dbscan_example()?;
            
            println!("\n=== All examples completed successfully ===\n");
        },
        _ => {
            println!("Unknown example: {}", args[1]);
            print_usage();
        }
    }
    
    Ok(())
}
