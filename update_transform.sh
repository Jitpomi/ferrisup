#!/bin/bash
set -e

# This script updates the transform.rs file to use the new direct_component approach

# Step 1: Add the import for direct_component
sed -i '' '/use crate::utils::{create_directory, update_workspace_members};/a\\
use crate::commands::direct_component::add_component_direct;' src/commands/transform.rs

# Step 2: Create a backup of the original transform.rs file
cp src/commands/transform.rs src/commands/transform.rs.bak

# Step 3: Create a new version of the add_component function
cat > add_component_new.rs << 'EOF'
// Function to add a component to a workspace
fn add_component(project_dir: &Path) -> Result<()> {
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let project_name = &structure.project_name;
    
    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;
    
    // Select component type
    let component_types = vec![
        "client - Frontend web application (Leptos, Yew, or Dioxus)",
        "server - Web server with API endpoints (Axum, Actix, or Poem)",
        "shared - Shared code between client and server",
        "edge - Edge computing applications (Cloudflare, Vercel, Fastly)",
        "data-science - Data science and machine learning projects",
        "embedded - Embedded systems firmware",
    ];
    
    let component_idx = Select::new()
        .with_prompt("Select component type:")
        .items(&component_types)
        .default(0)
        .interact()?;
    
    // Map index to component type
    let component_type = match component_idx {
        0 => "client",
        1 => "server",
        2 => "shared",
        3 => "edge",
        4 => "data-science",
        5 => "embedded",
        _ => "client", // Default to client
    };
    
    // Prompt for component name with default based on component type
    let component_name = Input::<String>::new()
        .with_prompt(format!("Component name [{}]", component_type))
        .default(component_type.to_string())
        .interact_text()?;
    
    // Define component directory path
    let component_dir = project_dir.join(&component_name);
    
    // Check if directory already exists
    if component_dir.exists() {
        println!("{} {}", 
            "Error:".red().bold(), 
            format!("Component directory '{}' already exists", component_name).red());
        return Ok(());
    }
    
    // Select framework if applicable
    let framework = match component_type {
        "client" => {
            let frameworks = vec![
                "leptos - Reactive web framework with fine-grained reactivity",
                "dioxus - Elegant React-like framework for desktop, web, and mobile",
                "tauri - Build smaller, faster, and more secure desktop applications",
            ];
            
            let framework_idx = Select::new()
                .with_prompt("Select framework:")
                .items(&frameworks)
                .default(0)
                .interact()?;
            
            match framework_idx {
                0 => Some("leptos"),
                1 => Some("dioxus"),
                2 => Some("tauri"),
                _ => None,
            }
        },
        "server" => {
            let frameworks = vec![
                "axum - Ergonomic and modular web framework by Tokio",
                "actix - Powerful, pragmatic, and extremely fast web framework",
                "poem - Full-featured and easy-to-use web framework",
            ];
            
            let framework_idx = Select::new()
                .with_prompt("Select framework:")
                .items(&frameworks)
                .default(0)
                .interact()?;
            
            match framework_idx {
                0 => Some("axum"),
                1 => Some("actix"),
                2 => Some("poem"),
                _ => None,
            }
        },
        "edge" => {
            let providers = vec![
                "cloudflare - Cloudflare Workers",
                "vercel - Vercel Edge Functions",
                "fastly - Fastly Compute@Edge",
                "aws - AWS Lambda@Edge",
            ];
            
            let provider_idx = Select::new()
                .with_prompt("Select provider:")
                .items(&providers)
                .default(0)
                .interact()?;
            
            match provider_idx {
                0 => Some("cloudflare"),
                1 => Some("vercel"),
                2 => Some("fastly"),
                3 => Some("aws"),
                _ => None,
            }
        },
        "data-science" => {
            let frameworks = vec![
                "polars - Fast DataFrame library",
                "linfa - Machine learning framework",
            ];
            
            let framework_idx = Select::new()
                .with_prompt("Select framework:")
                .items(&frameworks)
                .default(0)
                .interact()?;
            
            match framework_idx {
                0 => Some("polars"),
                1 => Some("linfa"),
                _ => None,
            }
        },
        _ => None,
    };
    
    // Map component type to template
    let template = map_component_to_template(component_type);
    
    // Use the direct component approach to create the component and fix imports
    add_component_direct(project_dir, &component_name, template, framework.as_deref())?;
    
    // Store transformation metadata
    store_transformation_metadata(project_dir, &component_name, template, framework.as_deref())?;
    
    println!("{}", format!("Component '{}' successfully added to workspace!", component_name).green());
    
    Ok(())
}
EOF

echo "Created new add_component function in add_component_new.rs"
echo "Please manually replace the add_component function in src/commands/transform.rs with the contents of add_component_new.rs"
echo "Then install FerrisUp with 'cargo install --path .'"
