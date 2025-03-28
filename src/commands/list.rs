use anyhow::Result;
use colored::*;

use crate::templates::get_all_templates;

pub fn execute() -> Result<()> {
    let templates = get_all_templates()?;
    
    println!("\n{}", "Available templates:".green().bold());
    
    for template in templates {
        match template.as_str() {
            "minimal" => {
                println!("  {} - {}", 
                    "minimal".cyan(),
                    "Simple binary with a single main.rs file");
            },
            "library" => {
                println!("  {} - {}", 
                    "library".cyan(),
                    "Rust library crate with a lib.rs file");
            },
            "full-stack" => {
                println!("  {} - {}", 
                    "full-stack".cyan(),
                    "Complete application with client, server, and shared libraries");
            },
            "gen-ai" => {
                println!("  {} - {}", 
                    "gen-ai".cyan(),
                    "AI-focused project with inference and model components");
            },
            "edge-app" => {
                println!("  {} - {}", 
                    "edge-app".cyan(),
                    "WebAssembly-based application for edge computing");
            },
            "embedded" => {
                println!("  {} - {}", 
                    "embedded".cyan(),
                    "Embedded systems firmware for microcontrollers");
            },
            "serverless" => {
                println!("  {} - {}", 
                    "serverless".cyan(),
                    "Serverless functions for cloud deployment");
            },
            "iot-device" => {
                println!("  {} - {}", 
                    "iot-device".cyan(),
                    "IoT device firmware with connectivity features");
            },
            "ml-pipeline" => {
                println!("  {} - {}", 
                    "ml-pipeline".cyan(),
                    "Machine learning data processing pipeline");
            },
            "data-science" => {
                println!("  {} - {}", 
                    "data-science".cyan(),
                    "Data science project with analysis tools");
            },
            _ => {
                println!("  {} - {}", 
                    template.cyan(),
                    "Custom template");
            }
        }
    }
    
    println!("\n{} {}", 
        "Usage:".yellow().bold(),
        "ferrisup new <PROJECT_NAME> --template=<TEMPLATE>\n");
    
    Ok(())
}
