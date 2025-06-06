use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
// Command import removed (no longer needed)
use crate::commands::import_fixer::fix_component_imports;
use crate::utils::{
    copy_dir_contents, create_directory, extract_dependencies, update_cargo_with_dependencies,
    update_workspace_members,
};
use dialoguer::{Confirm, Input, Select};
use toml_edit::{value, Document, Item, Table};

pub fn execute(project_path: Option<&str>, template_name: Option<&str>) -> Result<()> {
    println!(
        "{}",
        "FerrisUp Interactive Project Transformer".bold().green()
    );
    println!(
        "{}",
        "Transform your existing Rust project with new features".blue()
    );

    // Check if we're in test mode
    let is_test_mode = std::env::var("FERRISUP_TEST_MODE").is_ok();

    // Interactive mode if project path is not provided
    let path_str = match project_path {
        Some(p) => p.to_string(),
        None => {
            // Default to current directory
            let current_dir = std::env::current_dir()?;
            let use_current_dir = if is_test_mode {
                true
            } else {
                Confirm::new()
                    .with_prompt("Use current directory for transformation?")
                    .default(true)
                    .interact()?
            };

            if use_current_dir {
                current_dir.to_string_lossy().to_string()
            } else {
                // Prompt for project path
                Input::new()
                    .with_prompt("Enter the path to your project")
                    .interact_text()?
            }
        }
    };

    let project_dir = Path::new(&path_str);

    // Check if directory exists
    if !project_dir.exists() {
        println!(
            "{} {} {}",
            "Error:".red().bold(),
            "Directory".red(),
            format!("'{}' does not exist", path_str).red()
        );

        // Ask if user wants to specify a different path
        if Confirm::new()
            .with_prompt("Would you like to specify a different path?")
            .default(true)
            .interact()?
        {
            return execute(None, template_name);
        } else {
            return Ok(());
        }
    }

    // Check if it's a valid Rust project (has Cargo.toml)
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        println!(
            "{} {}",
            "Error:".red().bold(),
            format!(
                "'{}' is not a valid Rust project (Cargo.toml not found)",
                path_str
            )
            .red()
        );

        // Ask if user wants to specify a different path
        if Confirm::new()
            .with_prompt("Would you like to specify a different path?")
            .default(true)
            .interact()?
        {
            return execute(None, template_name);
        } else {
            return Ok(());
        }
    }

    // Analyze project structure
    println!("{}", "Analyzing project structure...".blue());
    let structure = analyze_project_structure(project_dir)?;

    // Print detected project type
    let project_type = if structure.is_workspace {
        "Workspace"
    } else if structure.is_binary {
        "Binary"
    } else {
        "Library"
    };

    println!(
        "{} {}",
        "Detected project type:".blue(),
        project_type.cyan()
    );

    // Main transformation loop
    let mut is_workspace = structure.is_workspace;
    loop {
        // Show different options based on whether it's a workspace or not
        let options = if is_workspace {
            vec!["Add a component", "Exit"]
        } else {
            vec![
                "Convert project to workspace",
                "Use current structure",
                "Exit",
            ]
        };

        let option_idx = if is_test_mode {
            0
        } else {
            Select::new()
                .with_prompt("What would you like to do?")
                .items(&options)
                .default(0)
                .interact()?
        };

        if is_workspace {
            match option_idx {
                0 => {
                    // Add a component
                    add_component(project_dir)?;
                }
                1 => {
                    // Exit
                    println!("{}", "Exiting transformation.".blue());
                    print_final_next_steps(project_dir)?;
                    break;
                }
                _ => unreachable!(),
            }
        } else {
            match option_idx {
                0 => {
                    // Convert to workspace
                    convert_to_workspace(project_dir)?;
                    is_workspace = true;
                    // Continue to the next iteration with workspace options
                    continue;
                }
                1 => {
                    // Use current structure
                    println!("{}", "Using current structure.".blue());
                    add_component_without_workspace(project_dir)?;
                }
                2 => {
                    // Exit
                    println!("{}", "Exiting transformation.".blue());
                    break;
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

// Structure to hold project analysis
#[derive(Debug)]
struct ProjectStructure {
    is_workspace: bool,
    is_binary: bool,
    project_name: String,
}

// Function to analyze project structure
fn analyze_project_structure(project_dir: &Path) -> Result<ProjectStructure> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;

    // Parse Cargo.toml
    let cargo_doc = cargo_toml_content
        .parse::<Document>()
        .context("Failed to parse Cargo.toml as TOML")?;

    // Check if it's a workspace
    let is_workspace = cargo_doc.get("workspace").is_some();

    // Check if it's a binary or library
    let is_binary = if let Some(_lib) = cargo_doc.get("lib") {
        false
    } else {
        // Check for bin target or assume binary if no lib section
        cargo_doc.get("bin").is_some() || !cargo_doc.get("package").is_none()
    };

    // Get project name
    let project_name = if let Some(package) = cargo_doc.get("package") {
        if let Some(name) = package.get("name") {
            name.as_str().unwrap_or("unknown").to_string()
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    Ok(ProjectStructure {
        is_workspace,
        is_binary,
        project_name,
    })
}

// Function to convert a project to a workspace
fn convert_to_workspace(project_dir: &Path) -> Result<()> {
    println!("{}", "Converting project to workspace...".blue());

    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let project_name = &structure.project_name;

    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

    // Detect component type from project structure
    let component_type = detect_component_type(project_dir)?;

    // Map component type to a more user-friendly name for the prompt
    let default_name = match component_type {
        "minimal" => "minimal",
        "client" => "client",
        "server" => "server",
        "shared" => "shared",
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        _ => component_type,
    };

    // Prompt for component name with default based on component type
    let component_name = Input::<String>::new()
        .with_prompt(format!(
            "What would you like to name the first component? [{}]",
            default_name
        ))
        .default(default_name.to_string())
        .interact_text()?;

    // Create component directory and src subdirectory
    let component_dir = project_dir.join(&component_name);
    create_directory(&component_dir)?;
    create_directory(&component_dir.join("src"))?;

    // Move all project files to component directory except workspace files
    let entries = fs::read_dir(project_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        // Skip workspace files and the component directory itself
        if file_name == "Cargo.toml"
            || file_name == "Cargo.lock"
            || file_name == ".git"
            || file_name == ".ferrisup"
            || file_name == component_name
        {
            continue;
        }

        // Move file or directory to component
        let target_path = component_dir.join(&file_name);

        if path.is_dir() {
            // Copy directory recursively
            copy_dir_all(&path, &target_path)?;
            // Remove original after successful copy
            fs::remove_dir_all(&path)?;
        } else {
            // Copy file
            fs::copy(&path, &target_path)?;
            // Remove original after successful copy
            fs::remove_file(&path)?;
        }
    }

    // Just copy the original Cargo.toml to the component directory
    let original_cargo_path = project_dir.join("Cargo.toml");
    let component_cargo_path = component_dir.join("Cargo.toml");

    // Copy the original Cargo.toml to the component directory
    fs::copy(&original_cargo_path, &component_cargo_path)?;

    // Update the component Cargo.toml package name
    let component_cargo_content = fs::read_to_string(&component_cargo_path)?;
    let mut component_cargo_doc = component_cargo_content
        .parse::<Document>()
        .context("Failed to parse component Cargo.toml")?;

    // Update the package name using the project_name from structure
    if let Some(package) = component_cargo_doc.get_mut("package") {
        if let Some(table) = package.as_table_mut() {
            table.insert(
                "name",
                value(format!(
                    "{0}_{1}",
                    project_name.to_lowercase(),
                    component_name.to_lowercase()
                )),
            );
        }
    }

    // Write updated component Cargo.toml
    fs::write(component_cargo_path, component_cargo_doc.to_string())?;

    // Update imports in source files to use the new package name
    update_source_imports(
        &component_dir,
        &project_name.to_lowercase(),
        &component_name.to_lowercase(),
    )?;

    // Create new Cargo.toml for workspace
    let workspace_cargo_toml = format!(
        r#"[workspace]
members = [
    "{}"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
"#,
        component_name
    );

    fs::write(project_dir.join("Cargo.toml"), workspace_cargo_toml)?;

    // Update component's Cargo.toml to use project-prefixed package name
    let component_cargo_path = component_dir.join("Cargo.toml");
    if component_cargo_path.exists() {
        let component_cargo_content = fs::read_to_string(&component_cargo_path)?;
        let mut component_doc = component_cargo_content
            .parse::<Document>()
            .context("Failed to parse component Cargo.toml")?;

        // Update package name to use project_component format with underscores
        if let Some(package) = component_doc.get_mut("package") {
            if let Some(name) = package.get_mut("name") {
                *name = toml_edit::value(format!("{}_{}", project_name, component_name));
            }
        }

        // Write updated Cargo.toml
        fs::write(component_cargo_path, component_doc.to_string())?;
    } else {
        // Create new Cargo.toml for component if it doesn't exist
        let component_cargo_toml = format!(
            r#"[package]
name = "{}_{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
            project_name, component_name
        );

        fs::write(component_cargo_path, component_cargo_toml)?;
    }

    // Detect framework from the original project (for metadata only)
    let mut detected_framework = None;
    let src_main_path = component_dir.join("src/main.rs");
    let src_lib_path = component_dir.join("src/lib.rs");

    // Check both main.rs and lib.rs for framework detection
    for src_path in &[&src_main_path, &src_lib_path] {
        if src_path.exists() {
            let content = fs::read_to_string(src_path)?;

            // Try to detect the framework from imports (for metadata only)
            if content.contains("use poem") {
                detected_framework = Some("poem");
                break;
            } else if content.contains("use axum") {
                detected_framework = Some("axum");
                break;
            } else if content.contains("use actix_web") {
                detected_framework = Some("actix");
                break;
            } else if content.contains("use leptos") {
                detected_framework = Some("leptos");
                break;
            } else if content.contains("use dioxus") {
                detected_framework = Some("dioxus");
                break;
            }
        }
    }

    // We're not adding dependencies manually anymore since we've preserved the original ones
    if let Some(framework) = detected_framework {
        println!("{} {}", "Detected framework:".blue(), framework.cyan());
    }

    // Determine the component type based on the component name and detected framework
    let template = match component_name.as_str() {
        "client" => "client",
        "server" => "server",
        "shared" => "shared",
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        _ => "server", // Default to server if unknown
    };

    // Store transformation metadata
    store_transformation_metadata(project_dir, &component_name, template, detected_framework)?;

    // Store component type in component's Cargo.toml metadata
    store_component_type_in_cargo(&component_dir, template)?;

    // Fix workspace resolver
    let workspace_cargo_path = project_dir.join("Cargo.toml");
    let workspace_cargo_content = fs::read_to_string(&workspace_cargo_path)?;
    let mut workspace_doc = workspace_cargo_content
        .parse::<Document>()
        .context("Failed to parse workspace Cargo.toml")?;

    // Add resolver = "2" to workspace
    if let Some(workspace) = workspace_doc.get_mut("workspace") {
        if let Some(table) = workspace.as_table_mut() {
            if table.get("resolver").is_none() {
                table.insert("resolver", value("2"));
            }
        }
    }

    // Write updated workspace Cargo.toml
    fs::write(workspace_cargo_path, workspace_doc.to_string())?;

    // Print success message
    println!("{}", "Project successfully converted to workspace!".green());

    // Print framework-specific instructions only for reference
    if let Some(framework) = detected_framework {
        println!("{} {}", "Detected framework:".blue(), framework.cyan());
    }

    Ok(())
}

// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

// Function to add a component to a workspace
pub fn add_component(project_dir: &Path) -> Result<()> {
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let project_name = &structure.project_name;

    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

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
        }
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
        }
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
        }
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
        }
        _ => None,
    };

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

    // Print the template being used for debugging
    println!("{}", format!("Using template: {}", template).blue());

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

    // Update component's Cargo.toml to use project-prefixed package name
    let component_cargo_path = component_dir.join("Cargo.toml");
    if component_cargo_path.exists() {
        let component_cargo_content = fs::read_to_string(&component_cargo_path)?;
        let mut component_doc = component_cargo_content
            .parse::<Document>()
            .context("Failed to parse component Cargo.toml")?;

        // Update package name to use project_component format with underscores
        if let Some(package) = component_doc.get_mut("package") {
            if let Some(name) = package.get_mut("name") {
                *name = toml_edit::value(format!("{}_{}", project_name, component_name));
            }
        }

        // Write updated Cargo.toml
        fs::write(component_cargo_path, component_doc.to_string())?;

        // Fix imports in source files to use the new package name
        // Make sure we're using the actual project name, not 'unknown'
        // First try to get the name from workspace.package.name
        // If that doesn't exist, try to get it from the directory name
        // If all else fails, use the project_dir name
        let workspace_cargo_path = project_dir.join("Cargo.toml");
        let actual_project_name = if workspace_cargo_path.exists() {
            if let Ok(workspace_content) = fs::read_to_string(&workspace_cargo_path) {
                if let Ok(workspace_doc) = workspace_content.parse::<Document>() {
                    // Try workspace.package.name first
                    if let Some(workspace) = workspace_doc.get("workspace") {
                        if let Some(pkg) = workspace.get("package") {
                            if let Some(name) = pkg.get("name") {
                                if let Some(name_str) = name.as_str() {
                                    name_str.to_string()
                                } else {
                                    // Fall back to project directory name
                                    project_dir
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(project_name)
                                        .to_string()
                                }
                            } else {
                                // Fall back to project directory name
                                project_dir
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(project_name)
                                    .to_string()
                            }
                        } else {
                            // Try package.name directly (for non-workspace projects)
                            if let Some(pkg) = workspace_doc.get("package") {
                                if let Some(name) = pkg.get("name") {
                                    if let Some(name_str) = name.as_str() {
                                        name_str.to_string()
                                    } else {
                                        // Fall back to project directory name
                                        project_dir
                                            .file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or(project_name)
                                            .to_string()
                                    }
                                } else {
                                    // Fall back to project directory name
                                    project_dir
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(project_name)
                                        .to_string()
                                }
                            } else {
                                // Fall back to project directory name
                                project_dir
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(project_name)
                                    .to_string()
                            }
                        }
                    } else {
                        // Try package.name directly (for non-workspace projects)
                        if let Some(pkg) = workspace_doc.get("package") {
                            if let Some(name) = pkg.get("name") {
                                if let Some(name_str) = name.as_str() {
                                    name_str.to_string()
                                } else {
                                    // Fall back to project directory name
                                    project_dir
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .unwrap_or(project_name)
                                        .to_string()
                                }
                            } else {
                                // Fall back to project directory name
                                project_dir
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .unwrap_or(project_name)
                                    .to_string()
                            }
                        } else {
                            // Fall back to project directory name
                            project_dir
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or(project_name)
                                .to_string()
                        }
                    }
                } else {
                    // Fall back to project directory name
                    project_dir
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(project_name)
                        .to_string()
                }
            } else {
                // Fall back to project directory name
                project_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(project_name)
                    .to_string()
            }
        } else {
            // Fall back to project directory name
            project_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(project_name)
                .to_string()
        };

        println!(
            "{}",
            format!("Using project name '{}' for imports", actual_project_name).blue()
        );
        fix_component_imports(&component_dir, &component_name, &actual_project_name)?;
    }

    // Update workspace Cargo.toml to include the new component
    // Note: The update_workspace_members function will automatically detect and add the component
    if let Err(e) = update_workspace_members(project_dir) {
        println!(
            "{} {}",
            "Warning: Failed to update workspace members:"
                .yellow()
                .bold(),
            e
        );
    }

    println!(
        "{}",
        format!(
            "Component '{}' successfully added to workspace!",
            component_name
        )
        .green()
    );

    Ok(())
}

// Function to add a component without converting to workspace
pub fn add_component_without_workspace(project_dir: &Path) -> Result<()> {
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;
    let _project_name = &structure.project_name;

    // Select component type - ONLY show module-compatible components
    let component_types = vec![
        "shared - Shared code between client and server",
        "minimal - Simple Rust module with minimal dependencies",
        // Could add other simple module types here if needed
    ];

    let component_idx = Select::new()
        .with_prompt("Select module type:")
        .items(&component_types)
        .default(0)
        .interact()?;

    // Map index to component type
    let component_type = match component_idx {
        0 => "shared",
        1 => "minimal",
        _ => "shared", // Default to shared
    };

    // Prompt for component name with default based on component type
    let module_name = Input::<String>::new()
        .with_prompt(format!("Module name [{}]", component_type))
        .default(component_type.to_string())
        .interact_text()?;

    // Create module directory inside src instead of at project root
    let src_dir = project_dir.join("src");
    let module_dir = src_dir.join(&module_name);

    // Check if directory already exists
    if module_dir.exists() {
        println!(
            "{} {}",
            "Error:".red().bold(),
            format!("Module directory '{}' already exists", module_name).red()
        );
        return Ok(());
    }

    // Create module directory
    create_directory(&module_dir)?;

    // First, create a temporary directory to generate the component template
    let temp_dir = tempfile::tempdir()?;
    let temp_component_dir = temp_dir.path().join(&module_name);

    // Save current directory
    let current_dir = std::env::current_dir()?;

    // Change to temp directory to create component template
    std::env::set_current_dir(temp_dir.path())?;

    // Map component type to template
    let template = map_component_to_template(component_type);

    println!(
        "{}",
        format!(
            "Creating {} module with name: {}",
            component_type, module_name
        )
        .blue()
    );

    // Call the new command to create the component template in temp directory
    let result = crate::commands::new::execute(
        Some(&module_name),
        Some(template),
        None, // No framework needed for modules
        None,
        None,
        false,
        false,
        true, // Use non-interactive mode
        None,
    );

    // Change back to original directory
    std::env::set_current_dir(current_dir)?;

    if let Err(e) = result {
        println!("{} {}", "Error creating component:".red().bold(), e);
        return Err(anyhow!("Failed to create component"));
    }

    // Extract dependencies from template's Cargo.toml
    let template_cargo_path = temp_component_dir.join("Cargo.toml");
    let mut dependencies_to_add = Vec::new();

    if template_cargo_path.exists() {
        let template_cargo_content = fs::read_to_string(&template_cargo_path)?;
        let template_doc = template_cargo_content
            .parse::<Document>()
            .context("Failed to parse template Cargo.toml")?;

        if let Some(deps) = template_doc.get("dependencies") {
            dependencies_to_add = extract_dependencies(deps)?;
        }
    }

    // Copy src files from template to module directory
    let template_src_dir = temp_component_dir.join("src");
    if template_src_dir.exists() {
        copy_dir_contents(&template_src_dir, &module_dir)?;

        // Rename lib.rs to mod.rs if it exists
        let lib_rs_path = module_dir.join("lib.rs");
        let mod_rs_path = module_dir.join("mod.rs");

        if lib_rs_path.exists() && !mod_rs_path.exists() {
            fs::rename(lib_rs_path, mod_rs_path)?;
        }
    }

    // Update project's Cargo.toml with dependencies
    let project_cargo_path = project_dir.join("Cargo.toml");
    if project_cargo_path.exists() && !dependencies_to_add.is_empty() {
        update_cargo_with_dependencies(&project_cargo_path, dependencies_to_add, false)?;
    }

    // Update main.rs or lib.rs to include the new module
    update_project_source_to_include_module(project_dir, &module_name)?;

    println!(
        "{}",
        format!(
            "Module '{}' successfully created within the project!",
            module_name
        )
        .green()
    );

    Ok(())
}

// Helper function to update project source to include the new module
fn update_project_source_to_include_module(project_dir: &Path, module_name: &str) -> Result<()> {
    // Determine if this is a binary or library project
    let main_rs_path = project_dir.join("src/main.rs");
    let lib_rs_path = project_dir.join("src/lib.rs");

    if main_rs_path.exists() {
        // Binary project
        let mut content = fs::read_to_string(&main_rs_path)?;

        // Add module declaration if not already present
        if !content.contains(&format!("mod {};", module_name)) {
            content.push_str(&format!("\nmod {};\n", module_name));
            fs::write(main_rs_path, content)?;
        }
    } else if lib_rs_path.exists() {
        // Library project
        let mut content = fs::read_to_string(&lib_rs_path)?;

        // Add module declaration and pub use if not already present
        if !content.contains(&format!("mod {};", module_name)) {
            content.push_str(&format!("\nmod {};\n", module_name));
            content.push_str(&format!("pub use {}::*;\n", module_name));
            fs::write(lib_rs_path, content)?;
        }
    }

    Ok(())
}

// Helper function to map component type to template
fn map_component_to_template(component_type: &str) -> &str {
    match component_type {
        "client" => "client",
        "server" => "server",
        "shared" => "library", // Use library template for shared components
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        "minimal" => "minimal",
        _ => "server", // Default to server
    }
}

// Helper function to update imports in source files
fn update_source_imports(
    component_dir: &Path,
    project_name: &str,
    component_name: &str,
) -> Result<()> {
    // Create the new package name
    let new_package_name = format!("{0}_{1}", project_name, component_name);

    // Walk through all Rust source files in the component directory
    let src_dir = component_dir.join("src");
    if !src_dir.exists() {
        return Ok(());
    }

    // Process main.rs
    let main_rs_path = src_dir.join("main.rs");
    if main_rs_path.exists() {
        update_imports_in_file(&main_rs_path, project_name, &new_package_name)?;
    }

    // Process lib.rs
    let lib_rs_path = src_dir.join("lib.rs");
    if lib_rs_path.exists() {
        update_imports_in_file(&lib_rs_path, project_name, &new_package_name)?;
    }

    // Process other Rust files in the src directory
    process_directory_imports(&src_dir, project_name, &new_package_name)?;

    Ok(())
}

// Helper function to update imports in a single file
fn update_imports_in_file(
    file_path: &Path,
    project_name: &str,
    new_package_name: &str,
) -> Result<()> {
    // Read the file content
    let content = fs::read_to_string(file_path)?;

    // Only replace exact package name to avoid multiple replacements
    // For example, replace "use app::*;" with "use app_client::*;"
    // but not "use app_client::*;" with "use app_client_client::*;"

    // Use regex to ensure we're only replacing the exact package name
    let re_import = regex::Regex::new(&format!(r"\buse\s+{}\b", regex::escape(project_name)))
        .context("Failed to create regex for import")?;
    let updated_content = re_import.replace_all(&content, format!("use {}", new_package_name));

    // Write the updated content back to the file
    fs::write(file_path, updated_content.to_string())?;

    Ok(())
}

// Helper function to recursively process all Rust files in a directory
fn process_directory_imports(
    dir_path: &Path,
    project_name: &str,
    new_package_name: &str,
) -> Result<()> {
    if !dir_path.exists() || !dir_path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            process_directory_imports(&path, project_name, new_package_name)?;
        } else if path.is_file() {
            // Process Rust files
            if let Some(extension) = path.extension() {
                if extension == "rs" {
                    update_imports_in_file(&path, project_name, new_package_name)?;
                }
            }
        }
    }

    Ok(())
}

// Function to detect component type based on project files
fn detect_component_type(project_dir: &Path) -> Result<&'static str> {
    // First, check if there's an explicit component type in the Cargo.toml metadata
    let cargo_toml = project_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml)?;
        let cargo_doc = cargo_content.parse::<Document>().ok();

        if let Some(doc) = cargo_doc {
            // Check for ferrisup metadata in Cargo.toml
            if let Some(package) = doc.get("package") {
                if let Some(metadata) = package.get("metadata") {
                    if let Some(ferrisup) = metadata.get("ferrisup") {
                        if let Some(component_type) = ferrisup.get("component_type") {
                            if let Some(component_str) = component_type.as_str() {
                                match component_str {
                                    "client" => return Ok("client"),
                                    "server" => return Ok("server"),
                                    "shared" => return Ok("shared"),
                                    "edge" => return Ok("edge"),
                                    "serverless" => return Ok("serverless"),
                                    "data-science" => return Ok("data-science"),
                                    "embedded" => return Ok("embedded"),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Check for .ferrisup/metadata.toml - this would have the original component type
    let metadata_path = project_dir.join(".ferrisup/metadata.toml");
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let metadata_doc = metadata_content.parse::<Document>().ok();

        if let Some(doc) = metadata_doc {
            // Try to get component_type directly
            if let Some(component_type) = doc.get("component_type") {
                if let Some(component_str) = component_type.as_str() {
                    match component_str {
                        "client" => return Ok("client"),
                        "server" => return Ok("server"),
                        "shared" => return Ok("shared"),
                        "edge" => return Ok("edge"),
                        "serverless" => return Ok("serverless"),
                        "data-science" => return Ok("data-science"),
                        "embedded" => return Ok("embedded"),
                        _ => {}
                    }
                }
            }

            // Try to infer from template
            if let Some(template) = doc.get("template") {
                if let Some(template_str) = template.as_str() {
                    if template_str.contains("serverless") {
                        return Ok("serverless");
                    } else if template_str.contains("edge") {
                        return Ok("edge");
                    } else if template_str.contains("client") {
                        return Ok("client");
                    } else if template_str.contains("server") {
                        return Ok("server");
                    } else if template_str.contains("shared") {
                        return Ok("shared");
                    } else if template_str.contains("data-science") {
                        return Ok("data-science");
                    } else if template_str.contains("embedded") {
                        return Ok("embedded");
                    }
                }
            }
        }

        // Fallback to simple string matching if parsing fails
        if metadata_content.contains("component_type = \"client\"") {
            return Ok("client");
        } else if metadata_content.contains("component_type = \"server\"") {
            return Ok("server");
        } else if metadata_content.contains("component_type = \"shared\"") {
            return Ok("shared");
        } else if metadata_content.contains("component_type = \"edge\"") {
            return Ok("edge");
        } else if metadata_content.contains("component_type = \"serverless\"") {
            return Ok("serverless");
        } else if metadata_content.contains("component_type = \"data-science\"") {
            return Ok("data-science");
        } else if metadata_content.contains("component_type = \"embedded\"") {
            return Ok("embedded");
        } else if metadata_content.contains("template = \"client\"") {
            return Ok("client");
        } else if metadata_content.contains("template = \"server\"") {
            return Ok("server");
        } else if metadata_content.contains("template = \"shared\"") {
            return Ok("shared");
        } else if metadata_content.contains("template = \"edge\"") {
            return Ok("edge");
        } else if metadata_content.contains("template = \"serverless\"") {
            return Ok("serverless");
        } else if metadata_content.contains("template = \"data-science\"") {
            return Ok("data-science");
        } else if metadata_content.contains("template = \"embedded\"") {
            return Ok("embedded");
        }
    }

    // Check for edge-specific files and directories
    if project_dir.join("workers-site").exists() || project_dir.join("wrangler.toml").exists() {
        return Ok("edge");
    }

    // Check for Vercel files - could be edge or serverless
    if project_dir.join("vercel.json").exists() || project_dir.join(".vercel").exists() {
        // Check if this is a serverless function by looking at the Cargo.toml
        let cargo_toml = project_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let cargo_content = fs::read_to_string(&cargo_toml)?;

            // If it contains serverless keywords, it's a serverless project
            if cargo_content.contains("lambda")
                || cargo_content.contains("aws-lambda")
                || cargo_content.contains("aws_lambda")
                || cargo_content.contains("serverless")
            {
                return Ok("serverless");
            }

            // If it contains edge keywords, it's an edge project
            if cargo_content.contains("wasm")
                || cargo_content.contains("static-site")
                || cargo_content.contains("edge")
            {
                return Ok("edge");
            }
        }

        // Check for serverless directory structure
        if project_dir.join("api").exists() {
            return Ok("serverless");
        }

        // Check for edge-specific files
        if project_dir.join("index.html").exists() && project_dir.join("pkg").exists() {
            return Ok("edge");
        }

        // Check metadata files for clues
        let metadata_path = project_dir.join(".ferrisup/metadata.toml");
        if metadata_path.exists() {
            let metadata_content = fs::read_to_string(&metadata_path)?;
            if metadata_content.contains("static-site") || metadata_content.contains("edge/static")
            {
                return Ok("edge");
            }
        }

        // Default to edge for Vercel projects with no serverless indicators
        return Ok("edge");
    }

    // Check for serverless-specific files
    if project_dir.join("serverless.yml").exists()
        || project_dir.join(".aws").exists()
        || project_dir.join("template.yaml").exists()
        || project_dir.join("template.yml").exists()
        || project_dir.join("sam-template.yaml").exists()
        || project_dir.join("sam-template.yml").exists()
    {
        return Ok("serverless");
    }

    // Check for client-specific files and imports
    let src_dir = project_dir.join("src");
    let main_rs = src_dir.join("main.rs");
    let lib_rs = src_dir.join("lib.rs");
    let index_html = project_dir.join("index.html");
    let cargo_toml = project_dir.join("Cargo.toml");

    // Check for frameworks in Cargo.toml
    if cargo_toml.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml)?;

        // Check for serverless frameworks first (higher priority)
        if cargo_content.contains("lambda")
            || cargo_content.contains("aws-lambda")
            || cargo_content.contains("aws_lambda")
            || cargo_content.contains("serverless")
        {
            return Ok("serverless");
        }

        // Check for edge frameworks
        if cargo_content.contains("worker")
            || cargo_content.contains("cloudflare")
            || cargo_content.contains("fastly")
        {
            return Ok("edge");
        }

        // Check for Vercel - could be edge or serverless
        if cargo_content.contains("vercel") {
            // Look for clues that this is a serverless function
            if project_dir.join("api").exists() {
                return Ok("serverless");
            } else {
                return Ok("edge");
            }
        }

        // Check for client frameworks
        if cargo_content.contains("leptos")
            || cargo_content.contains("dioxus")
            || cargo_content.contains("yew")
            || cargo_content.contains("trunk")
            || cargo_content.contains("wasm")
        {
            return Ok("client");
        }
    }

    // Check for index.html (typical in client projects)
    if index_html.exists() {
        return Ok("client");
    }

    // Check for imports in source files
    for rs_file in &[main_rs, lib_rs] {
        if rs_file.exists() {
            let content = fs::read_to_string(rs_file)?;

            // Check for edge frameworks
            if content.contains("use worker")
                || content.contains("use cloudflare")
                || content.contains("use vercel")
                || content.contains("use fastly")
            {
                return Ok("edge");
            }

            // Check for serverless frameworks
            if content.contains("use lambda")
                || content.contains("use aws_lambda")
                || content.contains("use lambda_runtime")
                || content.contains("lambda::handler")
                || content.contains("lambda::function")
            {
                return Ok("serverless");
            }

            // Check for client frameworks
            if content.contains("use leptos")
                || content.contains("use dioxus")
                || content.contains("use yew")
                || content.contains("wasm_bindgen")
            {
                return Ok("client");
            }

            // Check for server frameworks
            if content.contains("use poem")
                || content.contains("use axum")
                || content.contains("use actix")
                || content.contains("use rocket")
                || content.contains("use warp")
            {
                return Ok("server");
            }
        }
    }

    // Look for Trunk.toml (client project)
    if project_dir.join("Trunk.toml").exists() {
        return Ok("client");
    }

    // Check for data-science specific dependencies
    if cargo_toml.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml)?;
        if cargo_content.contains("polars")
            || cargo_content.contains("linfa")
            || cargo_content.contains("ndarray")
        {
            return Ok("data-science");
        }

        // Check for minimal template
        if let Some(metadata) = cargo_content.find("[package.metadata.ferrisup]") {
            let after_metadata = &cargo_content[metadata..];
            if after_metadata.contains("component_type = \"minimal\"") {
                return Ok("minimal");
            }
        }
    }

    // Check for embedded specific dependencies
    if cargo_toml.exists() {
        let cargo_content = fs::read_to_string(&cargo_toml)?;
        if cargo_content.contains("embedded-hal")
            || cargo_content.contains("cortex-m")
            || cargo_content.contains("stm32")
        {
            return Ok("embedded");
        }
    }

    // Check for serverless-related file patterns
    let src_dir = project_dir.join("src");
    if src_dir.exists() {
        let entries = fs::read_dir(src_dir)?;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            // Look for handler.rs, lambda.rs, or function.rs which are common in serverless projects
            if file_name == "handler.rs" || file_name == "lambda.rs" || file_name == "function.rs" {
                return Ok("serverless");
            }
        }
    }

    // Check for minimal project structure
    let src_dir = project_dir.join("src");
    let main_rs = src_dir.join("main.rs");

    // If it's just a simple binary with a main.rs and no other special files, it's likely minimal
    if main_rs.exists() && src_dir.exists() {
        // Check if src directory has only main.rs and no other files
        let entries = fs::read_dir(&src_dir)?;
        let file_count = entries.count();

        if file_count == 1 && cargo_toml.exists() {
            // Read Cargo.toml to check for minimal dependencies
            let cargo_content = fs::read_to_string(&cargo_toml)?;

            // If Cargo.toml is simple (few dependencies), it's likely minimal
            if !cargo_content.contains("axum")
                && !cargo_content.contains("actix")
                && !cargo_content.contains("poem")
                && !cargo_content.contains("rocket")
                && !cargo_content.contains("leptos")
                && !cargo_content.contains("dioxus")
                && !cargo_content.contains("yew")
            {
                // Check main.rs for simplicity
                let main_content = fs::read_to_string(&main_rs)?;
                if main_content.lines().count() < 30 {
                    return Ok("minimal");
                }
            }
        }
    }

    // Default to server for binary projects, shared for libraries
    let structure = analyze_project_structure(project_dir)?;
    if structure.is_binary {
        // For binary projects, prefer serverless over server as default if we detect any AWS-related files
        if project_dir.join(".aws").exists()
            || cargo_toml.exists() && fs::read_to_string(&cargo_toml)?.contains("aws")
        {
            Ok("serverless")
        } else {
            Ok("server")
        }
    } else {
        Ok("shared")
    }
}

// Function to store component type in Cargo.toml metadata
fn store_component_type_in_cargo(component_dir: &Path, component_type: &str) -> Result<()> {
    let cargo_path = component_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        return Ok(());
    }

    let cargo_content = fs::read_to_string(&cargo_path)?;
    let mut doc = cargo_content
        .parse::<Document>()
        .context("Failed to parse Cargo.toml")?;

    // Ensure package section exists
    if doc.get("package").is_none() {
        doc.insert("package", Item::Table(Table::new()));
    }

    // Get or create metadata section
    let package = doc["package"].as_table_mut().unwrap();
    if package.get("metadata").is_none() {
        package.insert("metadata", Item::Table(Table::new()));
    }

    // Get or create ferrisup section in metadata
    let metadata = package["metadata"].as_table_mut().unwrap();
    if metadata.get("ferrisup").is_none() {
        metadata.insert("ferrisup", Item::Table(Table::new()));
    }

    // Set component_type in ferrisup metadata
    let ferrisup = metadata["ferrisup"].as_table_mut().unwrap();
    ferrisup.insert("component_type", value(component_type));

    // Write updated Cargo.toml
    fs::write(cargo_path, doc.to_string())?;

    Ok(())
}

// Function to store transformation metadata
fn store_transformation_metadata(
    project_dir: &Path,
    component_name: &str,
    template: &str,
    framework: Option<&str>,
) -> Result<()> {
    // Ensure .ferrisup directory exists
    let ferrisup_dir = project_dir.join(".ferrisup");
    create_directory(&ferrisup_dir)?;

    let metadata_path = ferrisup_dir.join("metadata.toml");

    // Create or load existing metadata
    let metadata_content = if metadata_path.exists() {
        fs::read_to_string(&metadata_path)?
    } else {
        String::new()
    };

    let mut doc = if metadata_content.is_empty() {
        Document::new()
    } else {
        metadata_content
            .parse::<Document>()
            .context("Failed to parse metadata.toml")?
    };

    // Ensure components table exists
    if doc.get("components").is_none() {
        doc.insert("components", Item::Table(Table::new()));
    }

    // Add component metadata
    let components = doc["components"].as_table_mut().unwrap();

    let mut component_table = Table::new();
    component_table.insert("template", value(template));

    // Explicitly store the component_type based on the template or component name
    let component_type = match template {
        "client" => "client",
        "server" => "server",
        "shared" => "shared",
        "edge" => "edge",
        "serverless" => "serverless",
        "data-science" => "data-science",
        "embedded" => "embedded",
        _ => match component_name {
            "client" => "client",
            "server" => "server",
            "shared" => "shared",
            "edge" => "edge",
            "serverless" => "serverless",
            "data-science" => "data-science",
            "embedded" => "embedded",
            _ => template, // Default to template name if no match
        },
    };
    component_table.insert("component_type", value(component_type));

    if let Some(fw) = framework {
        component_table.insert("framework", value(fw));
    }
    component_table.insert("created_at", value(chrono::Local::now().to_rfc3339()));

    components.insert(component_name, Item::Table(component_table));

    // Write metadata back to file
    fs::write(metadata_path, doc.to_string()).context("Failed to write metadata.toml")?;

    Ok(())
}

// Function to print final next steps
fn print_final_next_steps(project_dir: &Path) -> Result<()> {
    // Get project structure
    let structure = analyze_project_structure(project_dir)?;

    if !structure.is_workspace {
        return Ok(());
    }

    // Get workspace members from Cargo.toml
    let workspace_cargo_path = project_dir.join("Cargo.toml");
    let workspace_cargo_content = fs::read_to_string(&workspace_cargo_path)?;
    let workspace_doc = workspace_cargo_content
        .parse::<Document>()
        .context("Failed to parse workspace Cargo.toml")?;

    // Extract component names from workspace members
    let mut component_names = Vec::new();
    if let Some(workspace) = workspace_doc.get("workspace") {
        if let Some(members) = workspace.get("members") {
            if let Some(members_array) = members.as_array() {
                for member in members_array {
                    if let Some(member_str) = member.as_str() {
                        component_names.push(member_str.to_string());
                    }
                }
            }
        }
    }

    // Get project name for package prefixes - more reliable method for workspaces
    let mut project_name = structure.project_name.to_lowercase();

    // If project_name is "unknown", try to determine it from directory name
    if project_name == "unknown" {
        if let Some(dir_name) = project_dir.file_name() {
            if let Some(dir_str) = dir_name.to_str() {
                project_name = dir_str.to_lowercase();
            }
        }
    }

    // Also try to get it from metadata if available
    let metadata_path = project_dir.join(".ferrisup/metadata.toml");
    if metadata_path.exists() && project_name == "unknown" {
        if let Ok(metadata_content) = fs::read_to_string(&metadata_path) {
            if let Ok(metadata_doc) = metadata_content.parse::<Document>() {
                if let Some(project_metadata) = metadata_doc.get("project") {
                    if let Some(name) = project_metadata.get("name") {
                        if let Some(name_str) = name.as_str() {
                            project_name = name_str.to_lowercase();
                        }
                    }
                }
            }
        }
    }

    println!("{}", "\nFinal Steps:\n".green().bold());

    // 1. Navigate to project directory
    println!("{}", "1. Navigate to project directory".blue());
    println!("cd {}", project_dir.display());
    println!();

    // Print comprehensive build instructions
    println!("{}", "Working with your components:".yellow().bold());

    // 2. Build all components at once
    println!("{}", "2. To build all components at once:".blue());
    print!("cargo build");

    // Add individual component build commands
    for component in &component_names {
        print!(" && cargo build -p {0}_{1}", project_name, component);
    }
    println!("\n");

    // 3. Build specific components
    println!("{}", "3. To build specific components:".blue());
    for component in &component_names {
        println!("   cargo build -p {0}_{1}", project_name, component);
    }
    println!();

    // 4. Run specific components
    println!("{}", "4. To run specific components:".blue());
    for component in &component_names {
        println!("   cargo run -p {0}_{1}", project_name, component);
    }
    println!();

    // 5. Test specific components
    println!("{}", "5. To test specific components:".blue());
    for component in &component_names {
        println!("   cargo test -p {0}_{1}", project_name, component);
    }
    println!();

    // 6. Adding dependencies
    println!("{}", "6. To add dependencies to components:".blue());
    println!("   cd [component_name] && cargo add [dependency_name]");
    println!("   OR");
    println!(
        "   cargo add [dependency_name] --package {0}_[component_name]",
        project_name
    );
    println!();

    // 7. Adding more components
    println!("{}", "7. To add more components in the future:".blue());
    println!("   ferrisup transform");
    println!();

    Ok(())
}

// End of file

// Function to fix imports in a component after updating the package name
