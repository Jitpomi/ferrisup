use anyhow::Result;
use colored::Colorize;
use crate::project::templates::list_templates;

pub fn execute() -> Result<()> {
    let templates = list_templates()?;
    
    println!("\n{}", "Available templates:".green().bold());
    
    for (template_name, template_description) in templates {
        match template_name.as_str() {
            "minimal" => println!("  • {} - {}", template_name.cyan().bold(), template_description),
            "full-stack" => println!("  • {} - {}", template_name.magenta().bold(), template_description),
            "data-science" => println!("  • {} - {}", template_name.blue().bold(), template_description),
            "library" => println!("  • {} - {}", template_name.yellow().bold(), template_description),
            "web-server" => println!("  • {} - {}", template_name.green().bold(), template_description),
            "cli" => println!("  • {} - {}", template_name.red().bold(), template_description),
            "embedded-embassy" => println!("  • {} - {}", template_name.cyan().bold(), template_description),
            "desktop-dioxus" => println!("  • {} - {}", template_name.magenta().bold(), template_description),
            "wasm" => println!("  • {} - {}", template_name.yellow().bold(), template_description),
            "tauri" => println!("  • {} - {}", template_name.green().bold(), template_description),
            _ => println!("  • {} - {}", template_name.white().bold(), template_description)
        }
    }
    
    println!("\n{}", "For more information about a template, use:".blue());
    println!("  {}", "ferrisup preview <template-name>".cyan());
    
    Ok(())
}
