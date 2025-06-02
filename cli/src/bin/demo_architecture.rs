use anyhow::Result;
use ferrisup::project;
use serde_json::json;
use std::path::PathBuf;
use colored::Colorize;
use std::fs;

fn main() -> Result<()> {
    println!("{}",
        "FerrisUp Project Handlers Architecture Demo"
        .green().bold()
    );
    println!("=======================================\n");
    
    // This simply prints the summary of all available handlers
    // It doesn't actually create any projects
    println!("Available Project Handlers:");
    println!("---------------------------");
    
    for (i, handler) in project::get_handlers().iter().enumerate() {
        println!("{}. {}: {}",
            (i + 1).to_string().cyan(),
            handler.name().green().bold(),
            handler.description()
        );
        
        // Get the templates this handler can handle
        let mut handles_templates = Vec::new();
        let test_templates = ["embedded-embassy", "data-science", "server", "client-dioxus", 
                             "tauri", "minimal", "library", "edge"];
        
        for template in test_templates.iter() {
            if handler.can_handle(template, &json!({})) {
                handles_templates.push(*template);
            }
        }
        
        if !handles_templates.is_empty() {
            println!("   Handles templates: {}", 
                handles_templates
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .yellow()
            );
        }
        
        println!();
    }
    
    // Check if a demo output directory exists and clean it
    let demo_dir = PathBuf::from("demo_output");
    if demo_dir.exists() {
        println!("Cleaning previous demo output...");
        fs::remove_dir_all(&demo_dir)?;
    }
    fs::create_dir_all(&demo_dir)?;
    
    // Print summary of what makes this architecture great
    println!("\nArchitecture Benefits:");
    println!("--------------------");
    println!("✅ {}", "Clean Separation of Concerns".green().bold());
    println!("   • CLI handlers and template handlers don't interfere with each other");
    println!("   • Changes to one handler type don't affect others");
    
    println!("✅ {}", "Easy Extension".green().bold());
    println!("   • New CLI tools can be added without affecting templates");
    println!("   • New templates can be added without breaking CLI tool support");
    
    println!("✅ {}", "Consistent Interface".green().bold());
    println!("   • All handlers expose the same interface for project creation");
    println!("   • Unified next steps generation makes user experience consistent");
    
    println!("✅ {}", "Parquet File Support".green().bold());
    println!("   • Data science template properly handles CSV, JSON, and Parquet formats");
    println!("   • Specific fixes ensure correct file extensions in next steps");
    
    println!("✅ {}", "Flexible Variable Handling".green().bold());
    println!("   • Templates can derive complex variables with Handlebars expressions");
    println!("   • CLI handlers map FerrisUp variables to CLI tool arguments");
    
    println!("\nTo try the actual implementation, run:");
    println!("cargo run -- new my-project --template [template-name]");
    
    Ok(())
}
