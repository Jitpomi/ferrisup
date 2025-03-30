use anyhow::Result;
use colored::*;

use crate::template_manager::get_all_templates;

pub fn execute() -> Result<()> {
    let templates = get_all_templates()?;
    
    println!("\n{}", "Available templates:".green().bold());
    
    for template in templates {
        match template.as_str() {
            "minimal" => {
                println!("  {} - Simple binary with a single main.rs file", 
                    "minimal".cyan());
            },
            "library" => {
                println!("  {} - Rust library crate with a lib.rs file", 
                    "library".cyan());
            },
            "full-stack" => {
                println!("  {} - Complete application with client, server, and shared libraries", 
                    "full-stack".cyan());
            },
            "gen-ai" => {
                println!("  {} - AI-focused project with inference and model components", 
                    "gen-ai".cyan());
            },
            "edge-app" => {
                println!("  {} - WebAssembly-based application for edge computing", 
                    "edge-app".cyan());
            },
            "embedded" => {
                println!("  {} - Embedded systems firmware for microcontrollers", 
                    "embedded".cyan());
            },
            "serverless" => {
                println!("  {} - Serverless functions for cloud deployment", 
                    "serverless".cyan());
            },
            "iot-device" => {
                println!("  {} - IoT device firmware with connectivity features", 
                    "iot-device".cyan());
            },
            "ml-pipeline" => {
                println!("  {} - Machine learning data processing pipeline", 
                    "ml-pipeline".cyan());
            },
            "data-science" => {
                println!("  {} - Data science project with analysis tools", 
                    "data-science".cyan());
            },
            _ => {
                println!("  {} - Custom template", 
                    template.cyan());
            }
        }
    }
    
    println!("\n{} ferrisup new <PROJECT_NAME> --template=<TEMPLATE>\n", 
        "Usage:".yellow().bold());
    
    Ok(())
}
