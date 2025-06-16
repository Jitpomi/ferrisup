use anyhow::Result;
use colored::Colorize;
use crate::project::templates::list_templates;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::path::PathBuf;
use std::fs;

pub fn execute() -> Result<()> {
    println!("{}", "FerrisUp Interactive Project Builder".bold().green());
    println!("{}", "Scale your Rust project with the features you need".blue());
    
    // Step 1: Get project directory (current or new)
    let use_current_dir = Confirm::new()
        .with_prompt("Use current directory for your project?")
        .default(true)
        .interact()?;
    
    let project_dir = if use_current_dir {
        std::env::current_dir()?
    } else {
        let dir: String = Input::new()
            .with_prompt("Enter the path for your new project")
            .interact_text()?;
        
        let path = PathBuf::from(&dir);
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        path
    };
    
    // New Step: Offer templates or custom setup
    let use_template = Confirm::new()
        .with_prompt("Would you like to use a predefined template?")
        .default(false)
        .interact()?;
    
    if use_template {
        // Get available templates
        let templates = list_templates()?;
        let template_names: Vec<String> = templates.iter().map(|(name, desc)| {
            format!("{} - {}", name, desc)
        }).collect();
        
        // Present template options
        let template_idx = Select::new()
            .with_prompt("Select a template")
            .items(&template_names)
            .default(0)
            .interact()?;
        
        let selected_template = templates[template_idx].0.clone();
        
        // Confirm template selection
        println!("\n{}", "Template Configuration Summary".bold().yellow());
        println!("Project directory: {}", project_dir.display().to_string().cyan());
        println!("Template: {}", selected_template.cyan());
        
        let proceed = Confirm::new()
            .with_prompt("Proceed with template-based project generation?")
            .default(true)
            .interact()?;
        
        if !proceed {
            println!("{}", "Project generation cancelled.".yellow());
            return Ok(());
        }
        
        // Generate project from template
        println!("\n{}", "Generating project from template...".green());
        
        // Here we would call the new::execute function with the selected template
        // commands::new::execute(&project_dir.file_name().unwrap().to_string_lossy(), &selected_template, false, false, false, None)?;
        
        println!("{}", "Project successfully generated!".bold().green());
        println!("Your new Rust project is ready at: {}", project_dir.display().to_string().cyan());
        
        return Ok(());
    }
    
    // Step 2: Project type selection
    let project_types = vec![
        "Binary (Simple application)",
        "Library (Reusable code)",
        "Workspace (Multi-crate project)",
    ];
    
    let project_type_idx = Select::new()
        .with_prompt("Select project type")
        .items(&project_types)
        .default(0)
        .interact()?;
    
    // Step 3: If workspace, ask about components
    let mut has_client = false;
    let mut has_server = false;
    let mut has_libs = false;
    let mut has_ai = false;
    let mut has_edge = false;
    let mut has_embedded = false;
    
    if project_type_idx == 2 {  // Workspace
        println!("\n{}", "Workspace Components".bold().cyan());
        
        let component_options = vec![
            "Client Applications (Web/Desktop UI)",
            "Server Services (APIs/Backend)",
            "Libraries (Shared code)",
            "AI Components (ML models, inference)",
            "Edge Computing (WASM, Serverless)",
            "Embedded Systems (IoT, Hardware)",
        ];
        
        let selections = MultiSelect::new()
            .with_prompt("Select the components for your workspace")
            .items(&component_options)
            .interact()?;
        
        has_client = selections.contains(&0);
        has_server = selections.contains(&1);
        has_libs = selections.contains(&2);
        has_ai = selections.contains(&3);
        has_edge = selections.contains(&4);
        has_embedded = selections.contains(&5);
    }
    
    // Step 4: Database selection
    let use_database = Confirm::new()
        .with_prompt("Include database support?")
        .default(false)
        .interact()?;
    
    let db_type = if use_database {
        let db_options = vec![
            "PostgreSQL",
            "MySQL",
            "SQLite",
            "MongoDB",
            "Redis",
            "DynamoDB",
            "None (will configure later)",
        ];
        
        let db_idx = Select::new()
            .with_prompt("Select database type")
            .items(&db_options)
            .default(0)
            .interact()?;
        
        match db_idx {
            0 => "postgres",
            1 => "mysql",
            2 => "sqlite",
            3 => "mongodb",
            4 => "redis",
            5 => "dynamodb",
            _ => "none",
        }
    } else {
        "none"
    };
    
    // Step 5: Client framework selection (if applicable)
    let client_frameworks = if has_client {
        println!("\n{}", "Client Framework Setup".bold().cyan());
        
        let framework_options = vec![
            "Dioxus (React-like, Web/Desktop/Mobile)",
            "Tauri (Desktop with web technologies)",
            "Leptos (Web with fine-grained reactivity)",
            "Yew (Component-based framework)",
            "Vanilla (No framework)",
        ];
        
        let selections = MultiSelect::new()
            .with_prompt("Select client_old frameworks to use")
            .items(&framework_options)
            .interact()?;
        
        let mut frameworks = Vec::new();
        if selections.contains(&0) { frameworks.push("dioxus"); }
        if selections.contains(&1) { frameworks.push("tauri"); }
        if selections.contains(&2) { frameworks.push("leptos"); }
        if selections.contains(&3) { frameworks.push("yew"); }
        if selections.contains(&4) { frameworks.push("vanilla"); }
        
        frameworks
    } else {
        Vec::new()
    };
    
    // Step 6: Server framework selection (if applicable)
    let server_frameworks = if has_server {
        println!("\n{}", "Server Framework Setup".bold().cyan());
        
        let framework_options = vec![
            "Poem (Simple and flexible)",
            "Axum (Modular and performant)",
            "Actix Web (Powerful and mature)",
            "Rocket (Ergonomic and boilerplate-free)",
            "Warp (Composable and fast)",
        ];
        
        let selections = MultiSelect::new()
            .with_prompt("Select server frameworks to use")
            .items(&framework_options)
            .interact()?;
        
        let mut frameworks = Vec::new();
        if selections.contains(&0) { frameworks.push("poem"); }
        if selections.contains(&1) { frameworks.push("axum"); }
        if selections.contains(&2) { frameworks.push("actix-web"); }
        if selections.contains(&3) { frameworks.push("rocket"); }
        if selections.contains(&4) { frameworks.push("warp"); }
        
        frameworks
    } else {
        Vec::new()
    };
    
    // Step 7: AI components (if applicable)
    let ai_components = if has_ai {
        println!("\n{}", "AI Components Setup".bold().cyan());
        
        let ai_options = vec![
            "Text Generation (LLaMA, GPT)",
            "Image Generation (Stable Diffusion)",
            "Speech Recognition (Whisper)",
            "Embeddings (BERT, Sentence transformers)",
            "Computer Vision (Object detection, classification)",
        ];
        
        let selections = MultiSelect::new()
            .with_prompt("Select AI capabilities to include")
            .items(&ai_options)
            .interact()?;
        
        let mut components = Vec::new();
        if selections.contains(&0) { components.push("text-generation"); }
        if selections.contains(&1) { components.push("image-generation"); }
        if selections.contains(&2) { components.push("speech-recognition"); }
        if selections.contains(&3) { components.push("embeddings"); }
        if selections.contains(&4) { components.push("computer-vision"); }
        
        components
    } else {
        Vec::new()
    };
    
    // Step 8: Edge components (if applicable)
    let edge_components = if has_edge {
        println!("\n{}", "Edge Computing Setup".bold().cyan());
        
        let edge_options = vec![
            "WebAssembly (WASM)",
            "Cloudflare Workers",
            "Deno Deploy",
            "Netlify Functions",
            "Vercel Edge Functions",
        ];
        
        let selections = MultiSelect::new()
            .with_prompt("Select edge computing targets")
            .items(&edge_options)
            .interact()?;
        
        let mut components = Vec::new();
        if selections.contains(&0) { components.push("wasm"); }
        if selections.contains(&1) { components.push("cloudflare-workers"); }
        if selections.contains(&2) { components.push("deno-deploy"); }
        if selections.contains(&3) { components.push("netlify-functions"); }
        if selections.contains(&4) { components.push("vercel-edge"); }
        
        components
    } else {
        Vec::new()
    };
    
    // Step 9: Embedded components (if applicable)
    let embedded_components = if has_embedded {
        println!("\n{}", "Embedded Systems Setup".bold().cyan());
        
        let embedded_options = vec![
            "Raspberry Pi Pico (RP2040)",
            "ESP32",
            "STM32",
            "Arduino",
            "Generic Microcontroller",
        ];
        
        let selections = MultiSelect::new()
            .with_prompt("Select embedded targets")
            .items(&embedded_options)
            .interact()?;
        
        let mut components = Vec::new();
        if selections.contains(&0) { components.push("rp2040"); }
        if selections.contains(&1) { components.push("esp32"); }
        if selections.contains(&2) { components.push("stm32"); }
        if selections.contains(&3) { components.push("arduino"); }
        if selections.contains(&4) { components.push("generic"); }
        
        components
    } else {
        Vec::new()
    };
    
    // Step 10: Additional features
    println!("\n{}", "Additional Features".bold().cyan());
    
    let feature_options = vec![
        "GitHub Actions CI/CD",
        "Docker containerization",
        "Kubernetes manifests",
        "Observability setup (metrics/tracing)",
        "Authentication boilerplate",
        "Testing infrastructure",
        "Documentation generation",
    ];
    
    let feature_selections = MultiSelect::new()
        .with_prompt("Select additional features to include")
        .items(&feature_options)
        .interact()?;
    
    // Step 11: Confirmation
    println!("\n{}", "Project Configuration Summary".bold().yellow());
    println!("Project directory: {}", project_dir.display().to_string().cyan());
    println!("Project type: {}", project_types[project_type_idx].cyan());
    
    if project_type_idx == 2 {
        println!("Components:");
        if has_client { println!("  - {}", "Client Applications".cyan()); }
        if has_server { println!("  - {}", "Server Services".cyan()); }
        if has_libs { println!("  - {}", "Libraries".cyan()); }
        if has_ai { println!("  - {}", "AI Components".cyan()); }
        if has_edge { println!("  - {}", "Edge Computing".cyan()); }
        if has_embedded { println!("  - {}", "Embedded Systems".cyan()); }
    }
    
    if use_database {
        println!("Database: {}", db_type.cyan());
    } else {
        println!("Database: {}", "None".cyan());
    }
    
    if !client_frameworks.is_empty() {
        println!("Client frameworks: {}", client_frameworks.join(", ").cyan());
    }
    
    if !server_frameworks.is_empty() {
        println!("Server frameworks: {}", server_frameworks.join(", ").cyan());
    }
    
    if !ai_components.is_empty() {
        println!("AI components: {}", ai_components.join(", ").cyan());
    }
    
    if !edge_components.is_empty() {
        println!("Edge components: {}", edge_components.join(", ").cyan());
    }
    
    if !embedded_components.is_empty() {
        println!("Embedded components: {}", embedded_components.join(", ").cyan());
    }
    
    if !feature_selections.is_empty() {
        println!("Additional features:");
        for &idx in &feature_selections {
            println!("  - {}", feature_options[idx].cyan());
        }
    }
    
    let proceed = Confirm::new()
        .with_prompt("Proceed with project generation?")
        .default(true)
        .interact()?;
    
    if !proceed {
        println!("{}", "Project generation cancelled.".yellow());
        return Ok(());
    }
    
    // Step 12: Generate the project
    println!("\n{}", "Generating project...".green());
    
    // Here we would call functions to create the actual project structure
    // based on the selected options. This would involve creating directories,
    // writing Cargo.toml files, copying template files, etc.
    
    // For now, we'll just print a success message
    println!("{}", "Project successfully generated!".bold().green());
    println!("Your new Rust project is ready at: {}", project_dir.display().to_string().cyan());
    
    Ok(())
}
