use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;
use std::fs;
use ferrisup_common::fs::create_directory;
use ferrisup_common::cargo;
use toml_edit::{DocumentMut};

use crate::commands::test_mode::is_test_mode;
use super::project_structure::{analyze_project_structure, map_component_to_template};
use super::utils::{store_transformation_metadata, store_component_type_in_cargo, make_shared_component_accessible, update_root_file_references, add_component_to_workspace};
use super::ui::{get_input_with_default, select_option};
use super::constants::{get_formatted_component_types, get_component_type_names};

// Function to add a component to a workspace
pub fn add_component(project_dir: &Path) -> Result<()> {
    // Get project structure - we don't use this directly but it validates the project
    analyze_project_structure(project_dir)?;

    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

    // Select component type
    let component_types = get_formatted_component_types();

    let component_idx = if is_test_mode() {
        0 // Default to first option in test mode
    } else {
        select_option("Select component type:", &component_types, 0)?
    };

    // Map index to component type
    let component_type = get_component_type_names()[component_idx];

    // Prompt for component name with default based on component type
    let mut component_name = get_input_with_default(
        &format!("Component name [{}]", component_type),
        component_type
    )?;
    
    // For shared components, check if the crate name is available on crates.io
    if component_type == "shared" {
        // Keep prompting until we get an available name
        let mut is_available = false;
        while !is_available {
            match cargo::is_crate_name_available(&component_name) {
                Ok(available) => {
                    if available {
                        is_available = true;
                        println!(
                            "{} {}",
                            "Success:".green().bold(),
                            format!("Crate name '{}' is available on crates.io", component_name).green()
                        );
                    } else {
                        println!(
                            "{} {}",
                            "Warning:".yellow().bold(),
                            format!("Crate name '{}' is already taken on crates.io", component_name).yellow()
                        );
                        
                        // Prompt for a different name
                        component_name = get_input_with_default(
                            "Please enter a different name for your shared component",
                            &format!("{}-common", project_dir.file_name().unwrap_or_default().to_string_lossy())
                        )?;
                    }
                },
                Err(e) => {
                    println!(
                        "{} {}",
                        "Warning:".yellow().bold(),
                        format!("Could not check crate name availability: {}", e).yellow()
                    );
                    // If we can't check, just proceed with the name
                    is_available = true;
                }
            }
        }
    }

    // Define component directory path (but don't create it yet)
    let component_dir = project_dir.join(&component_name);

    // Check if directory already exists
    if component_dir.exists() {
        println!(
            "{} {}",
            "Error:".red().bold(),
            format!("Component directory '{}' already exists", component_name).red()
        );
        return Ok(());
    }

    // Select framework if applicable
    let framework = select_framework_for_component_type(component_type)?;

    // Create the component by directly calling the new command
    println!(
        "{}",
        format!(
            "Creating {} component with name: {}",
            component_type, component_name
        )
        .blue()
    );

    // Map component type to template
    // For shared components, we need to explicitly use "library" as the template
    let template = if component_type == "shared" {
        "library"
    } else {
        map_component_to_template(component_type)
    };

    // Save current directory to return to it after component creation
    let current_dir = std::env::current_dir()?;

    // Change to project directory to create component at the right location
    std::env::set_current_dir(project_dir)?;

    // Call the new command to create the component
    let result = crate::commands::new::execute(
        Some(&component_name),
        Some(template),
        framework.as_deref(),
        None,
        None,
        false,
        false,
        false,
        None,
    );

    // Change back to original directory
    std::env::set_current_dir(current_dir)?;

    if let Err(e) = result {
        println!("{} {}", "Error creating component:".red().bold(), e);
        return Err(anyhow!("Failed to create component"));
    }

    // Display detected framework
    if let Some(framework_name) = &framework {
        println!("{} {}", "Using framework:".blue(), framework_name.cyan());
    }

    // Store transformation metadata
    store_transformation_metadata(project_dir, &component_name, template, framework.as_deref())?;

    // Store component type in component's Cargo.toml metadata
    store_component_type_in_cargo(&component_dir, template)?;

    // Add the component to workspace members
    add_component_to_workspace(project_dir, &component_name)?;

    // If this is a shared component, make it accessible to all workspace members
    if component_type == "shared" {
        make_shared_component_accessible(project_dir, &component_name)?;
    }

    // Create an empty list for files to keep at root since we're not moving files in this case
    let files_to_keep_at_root: Vec<String> = Vec::new();
    
    // Update references in files kept at the root
    update_root_file_references(project_dir, &component_name, &files_to_keep_at_root)?;

    Ok(())
}

// Function to add a component without converting to workspace
pub fn add_component_without_workspace(project_dir: &Path) -> Result<()> {
    println!(
        "{}",
        "Adding component to existing project structure...".blue()
    );

    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let project_name = &structure.project_name;

    // Select component type
    let component_types = get_formatted_component_types();

    let component_idx = if is_test_mode() {
        0 // Default to first option in test mode
    } else {
        select_option("Select component type to add:", &component_types, 0)?
    };

    // Map index to component type
    let component_type = get_component_type_names()[component_idx];

    // Select framework if applicable
    let framework = select_framework_for_component_type(component_type)?;

    // Map component type to template
    let template = map_component_to_template(component_type);

    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

    // Store transformation metadata
    store_transformation_metadata(project_dir, project_name, template, framework.as_deref())?;

    // Store component type in project's Cargo.toml metadata
    store_component_type_in_cargo(project_dir, template)?;

    // Add framework-specific dependencies to the project
    if let Some(framework_name) = &framework {
        add_framework_dependencies(project_dir, framework_name)?;
        println!("{} {}", "Added dependencies for framework:".blue(), framework_name.cyan());
    }

    println!(
        "{}",
        "Component functionality added to existing project!".green()
    );

    Ok(())
}

// Helper function to select framework based on component type
fn select_framework_for_component_type(component_type: &str) -> Result<Option<String>> {
    match component_type {
        "client" => {
            let frameworks = vec![
                "leptos - Reactive web framework with fine-grained reactivity".to_string(),
                "dioxus - Elegant React-like framework for desktop, web, and mobile".to_string(),
                "tauri - Build smaller, faster, and more secure desktop applications".to_string(),
            ];

            let framework_idx = if is_test_mode() {
                0
            } else {
                select_option("Select framework:", &frameworks, 0)?
            };

            match framework_idx {
                0 => Ok(Some("leptos".to_string())),
                1 => Ok(Some("dioxus".to_string())),
                2 => Ok(Some("tauri".to_string())),
                _ => Ok(None),
            }
        }
        "server" => {
            let frameworks = vec![
                "axum - Ergonomic and modular web framework by Tokio".to_string(),
                "actix - Powerful, pragmatic, and extremely fast web framework".to_string(),
                "poem - Full-featured and easy-to-use web framework".to_string(),
            ];

            let framework_idx = if is_test_mode() {
                0
            } else {
                select_option("Select framework:", &frameworks, 0)?
            };

            match framework_idx {
                0 => Ok(Some("axum".to_string())),
                1 => Ok(Some("actix".to_string())),
                2 => Ok(Some("poem".to_string())),
                _ => Ok(None),
            }
        }
        "edge" => {
            let providers = vec![
                "cloudflare - Cloudflare Workers".to_string(),
                "vercel - Vercel Edge Functions".to_string(),
                "fastly - Fastly Compute@Edge".to_string(),
                "aws - AWS Lambda@Edge".to_string(),
            ];

            let provider_idx = if is_test_mode() {
                0
            } else {
                select_option("Select provider:", &providers, 0)?
            };

            match provider_idx {
                0 => Ok(Some("cloudflare".to_string())),
                1 => Ok(Some("vercel".to_string())),
                2 => Ok(Some("fastly".to_string())),
                3 => Ok(Some("aws".to_string())),
                _ => Ok(None),
            }
        }
        "data-science" => {
            let frameworks = vec![
                "polars - Fast DataFrame library".to_string(),
                "linfa - Machine learning framework".to_string(),
            ];

            let framework_idx = if is_test_mode() {
                0
            } else {
                select_option("Select framework:", &frameworks, 0)?
            };

            match framework_idx {
                0 => Ok(Some("polars".to_string())),
                1 => Ok(Some("linfa".to_string())),
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

// Helper function to add framework dependencies to a project
fn add_framework_dependencies(project_dir: &Path, framework: &str) -> Result<()> {
    let cargo_path = project_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        return Ok(());
    }

    let cargo_content = fs::read_to_string(&cargo_path)?;
    let mut cargo_doc = cargo_content.parse::<DocumentMut>()?;

    // Add dependencies based on framework
    if let Some(deps) = cargo_doc.get_mut("dependencies") {
        if let Some(deps_table) = deps.as_table_mut() {
            match framework {
                "axum" => {
                    if deps_table.get("axum").is_none() {
                        deps_table.insert("axum", toml_edit::value("0.7.4"));
                        let mut tokio_table = toml_edit::Table::new();
                        tokio_table.insert("version", toml_edit::value("1.36.0"));
                        let mut features_array = toml_edit::Array::new();
                        features_array.push("full");
                        tokio_table.insert("features", toml_edit::value(features_array));
                        deps_table.insert("tokio", toml_edit::Item::Table(tokio_table));
                    }
                }
                "actix" => {
                    if deps_table.get("actix-web").is_none() {
                        deps_table.insert("actix-web", toml_edit::value("4.5.1"));
                    }
                }
                "poem" => {
                    if deps_table.get("poem").is_none() {
                        deps_table.insert("poem", toml_edit::value("2.0.0"));
                    }
                }
                "leptos" => {
                    if deps_table.get("leptos").is_none() {
                        deps_table.insert("leptos", toml_edit::value("0.6.5"));
                    }
                }
                "dioxus" => {
                    if deps_table.get("dioxus").is_none() {
                        deps_table.insert("dioxus", toml_edit::value("0.4.3"));
                        deps_table.insert("dioxus-web", toml_edit::value("0.4.3"));
                    }
                }
                "tauri" => {
                    if deps_table.get("tauri").is_none() {
                        let mut tauri_table = toml_edit::Table::new();
                        tauri_table.insert("version", toml_edit::value("1.6.0"));
                        let mut features_array = toml_edit::Array::new();
                        features_array.push("api-all");
                        tauri_table.insert("features", toml_edit::value(features_array));
                        deps_table.insert("tauri", toml_edit::Item::Table(tauri_table));
                    }
                }
                "polars" => {
                    if deps_table.get("polars").is_none() {
                        let mut polars_table = toml_edit::Table::new();
                        polars_table.insert("version", toml_edit::value("0.38.1"));
                        let mut features_array = toml_edit::Array::new();
                        features_array.push("lazy");
                        features_array.push("csv");
                        features_array.push("json");
                        polars_table.insert("features", toml_edit::value(features_array));
                        deps_table.insert("polars", toml_edit::Item::Table(polars_table));
                    }
                }
                "linfa" => {
                    if deps_table.get("linfa").is_none() {
                        deps_table.insert("linfa", toml_edit::value("0.7.0"));
                        deps_table.insert("ndarray", toml_edit::value("0.15.6"));
                    }
                }
                _ => {}
            }
        }
    }

    // Write updated Cargo.toml
    fs::write(cargo_path, cargo_doc.to_string())?;
    
    Ok(())
}
