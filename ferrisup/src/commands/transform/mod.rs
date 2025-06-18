use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use dialoguer::Select;
use crate::commands::test_mode::is_test_mode;

// Re-export component functions for backward compatibility
pub use self::component::add_component;
pub use self::component::add_component_without_workspace;

// Declare submodules
pub mod project_structure;
pub mod workspace;
pub mod component;
pub mod utils;
pub mod ui;
pub mod workspace_utils;
pub mod constants;

pub fn execute(project_path: Option<&str>, template_name: Option<&str>) -> Result<()> {
    ui::print_banner();

    // Get project path from argument or use current directory
    let project_path_buf = if let Some(path) = project_path {
        Path::new(path).to_path_buf()
    } else {
        std::env::current_dir()?
    };
    
    let path_str = project_path_buf.to_string_lossy().to_string();
    let project_dir = Path::new(&path_str);

    // Check if directory exists
    if !project_dir.exists() {
        ui::print_error(format!("Directory '{}' does not exist", path_str));

        // Ask if user wants to specify a different path
        if ui::confirm_action("Would you like to specify a different path?", true)? {
            return execute(None, template_name);
        } else {
            return Ok(());
        }
    }

    // Check if it's a valid Rust project (has Cargo.toml)
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        ui::print_error(format!(
            "'{}' is not a valid Rust project (Cargo.toml not found)",
            path_str
        ));

        // Ask if user wants to specify a different path
        if ui::confirm_action("Would you like to specify a different path?", true)? {
            return execute(None, template_name);
        } else {
            return Ok(());
        }
    }

    // Analyze project structure
    let structure = project_structure::analyze_project_structure(project_dir)?;

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

        let option_idx = if is_test_mode() {
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
                    component::add_component(project_dir)?;
                }
                1 => {
                    // Exit
                    println!("{}", "Exiting transformation.".blue());
                    ui::print_final_next_steps(project_dir)?;
                    break;
                }
                _ => unreachable!(),
            }
        } else {
            match option_idx {
                0 => {
                    // Convert to workspace
                    workspace::convert_to_workspace(project_dir)?;
                    is_workspace = true;
                    // Continue to the next iteration with workspace options
                    continue;
                }
                1 => {
                    // Use current structure
                    println!("{}", "Using current structure.".blue());
                    component::add_component_without_workspace(project_dir)?;
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
