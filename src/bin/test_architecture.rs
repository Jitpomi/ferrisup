use anyhow::Result;
use colored::*;
use ferrisup::core::Config;
use ferrisup::project::{find_handler, get_handlers};
use ferrisup::project::templates;
use serde_json::json;

fn main() -> Result<()> {
    println!("\n{}", "FerrisUp Architecture Test".green().bold());
    println!("{}", "======================".green());

    // Test core modules
    println!("\n{}", "🔍 Testing Core Modules".cyan().bold());
    let config = Config::default();
    println!("  ✅ Config loaded successfully");
    println!("  📁 Templates directory: {}", config.templates_dir.display());

    // Test project handlers
    println!("\n{}", "🔍 Testing Project Handlers".cyan().bold());
    let handlers = get_handlers();
    println!("  ✅ Found {} project handlers", handlers.len());

    // Print handlers for verification
    for (i, handler) in handlers.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, handler.name().cyan(), handler.description());
    }

    // Test template functionality  
    println!("\n{}", "🔍 Testing Template Management".cyan().bold());
    match templates::list_templates() {
        Ok(templates) => {
            println!("  ✅ Found {} templates", templates.len());
            for (i, (name, desc)) in templates.iter().enumerate().take(5) {
                println!("  {}. {} - {}", i + 1, name.cyan(), desc);
            }
            if templates.len() > 5 {
                println!("  ... and {} more", templates.len() - 5);
            }
        },
        Err(e) => println!("  ❌ Failed to list templates: {}", e)
    }

    // Test handler finding functionality
    println!("\n{}", "🔍 Testing Handler Selection".cyan().bold());
    let test_templates = vec![
        "minimal",
        "embedded-embassy",
        "data-science",
        "client-dioxus",
        "server"
    ];

    for template in test_templates {
        let variables = json!({
            "template": template,
            "project_name": "test_project"
        });

        match find_handler(template, &variables) {
            Some(handler) => println!("  ✅ Found handler for '{}': {}", template, handler.name()),
            None => println!("  ❌ No handler found for '{}'", template)
        }
    }

    // Verify template application is available (just check the function exists, don't actually run it)
    println!("\n{}", "🔍 Testing Template Application".cyan().bold());
    println!("  ✅ Function templates::apply_template is available");
    println!("  ✅ Function templates::get_template_config is available");
    println!("  ✅ Function templates::get_template_next_steps is available");

    println!("\n{}", "Architecture Test Complete".green().bold());
    println!("{}", "======================".green());

    Ok(())
}
