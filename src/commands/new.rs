use std::path::Path;
use std::process::Command;
use anyhow::{Result, anyhow};
use dialoguer::{Select, Input};
use crate::template_manager;
use crate::utils::create_directory;
use serde_json::{self, json};
use std::fs;
use std::io;

// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
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

// Note: For frameworks and libraries that have official CLIs (like Dioxus and Tauri),
// we use those CLIs directly instead of maintaining our own templates.
// This ensures we're always using the most up-to-date project creation methods
// and reduces maintenance burden.

// Main execute function to handle Leptos project creation
pub fn execute(
    name: Option<&str>,
    template: Option<&str>,
    git: bool,
    build: bool,
    no_interactive: bool,
    _project_type: Option<&str>,
) -> Result<()> {
    // Get project name
    let name = match name {
        Some(name) => name.to_string(),
        None => {
            if no_interactive {
                return Err(anyhow!("Project name is required in non-interactive mode"));
            }
            Input::<String>::new()
                .with_prompt("Project name")
                .interact()?
        }
    };

    // Create project directory
    let app_path = Path::new(&name);
    create_directory(app_path)?;

    // Get template
    let mut template = match template {
        Some(template) => template.to_string(),
        None => {
            if no_interactive {
                return Err(anyhow!("Template is required in non-interactive mode"));
            }
            
            let templates_with_desc = template_manager::list_templates()?;
            let templates: Vec<&str> = templates_with_desc.iter().map(|(name, _)| name.as_str()).collect();
            
            let selection = Select::new()
                .with_prompt("Select a template")
                .items(&templates)
                .default(0)
                .interact()?;
                
            templates[selection].to_string()
        }
    };

    // Declare additional_vars here
    let mut additional_vars = None;

    // Get template configuration to check for options
    let template_config = template_manager::get_template_config(&template)?;
    
    // Check if the template has options that need user input
    if let Some(options) = template_config.get("options").and_then(|o| o.as_array()) {
        let mut vars = serde_json::Map::new();
        
        for option in options {
            if let (Some(name), Some(desc), Some(option_type)) = (
                option.get("name").and_then(|n| n.as_str()),
                option.get("description").and_then(|d| d.as_str()),
                option.get("type").and_then(|t| t.as_str())
            ) {
                if option_type == "select" && option.get("options").is_some() {
                    let option_values: Vec<&str> = option.get("options")
                        .and_then(|o| o.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                        .unwrap_or_default();
                    
                    if !option_values.is_empty() {
                        let default_idx = option.get("default")
                            .and_then(|d| d.as_str())
                            .and_then(|d| option_values.iter().position(|&v| v == d))
                            .unwrap_or(0);
                        
                        let selection = Select::new()
                            .with_prompt(desc)
                            .items(&option_values)
                            .default(default_idx)
                            .interact()?;
                        
                        let selected_value = option_values[selection];
                        vars.insert(name.to_string(), json!(selected_value));
                        println!("Using {} as the {}", selected_value, name);
                    }
                }
            }
        }
        
        if !vars.is_empty() {
            additional_vars = Some(serde_json::Value::Object(vars));
        }
    }

    // Handle special cases for client frameworks
    let mut _framework = String::new();

    // For client template, prompt for framework selection
    if template == "client" {
        println!("Template description: Custom template: client");
        println!("Using template: client");
        
        // Get client framework
        let frameworks = vec!["dioxus", "tauri", "leptos"];
        let selection = Select::new()
            .with_prompt("Select Rust client framework")
            .items(&frameworks)
            .default(0)
            .interact()?;
            
        let framework_selected = frameworks[selection];
        
        // For Leptos, prompt for specific template type
        if framework_selected == "leptos" {
            println!("📦 Using Leptos templates to bootstrap the project");
            println!("🔧 Checking for required dependencies...");
            
            // Check for wasm32-unknown-unknown target
            println!("🔍 Checking for wasm32-unknown-unknown target...");
            let wasm_check = Command::new("rustup")
                .args(["target", "list", "--installed"])
                .output()?;
            
            let wasm_output = String::from_utf8_lossy(&wasm_check.stdout);
            if !wasm_output.contains("wasm32-unknown-unknown") {
                println!("⚠️ wasm32-unknown-unknown target not found. Installing...");
                let status = Command::new("rustup")
                    .args(["target", "add", "wasm32-unknown-unknown"])
                    .status()?;
                
                if !status.success() {
                    println!("❌ Failed to install wasm32-unknown-unknown target.");
                    println!("Please install it manually with: rustup target add wasm32-unknown-unknown");
                } else {
                    println!("✅ wasm32-unknown-unknown target installed successfully");
                }
            } else {
                println!("✅ wasm32-unknown-unknown target is already installed");
            }
            
            // Check for trunk (needed for counter, router, todo templates)
            println!("🔍 Checking for Trunk...");
            let trunk_check = Command::new("trunk")
                .arg("--version")
                .output();
            
            match trunk_check {
                Ok(_) => println!("✅ Trunk is already installed"),
                Err(_) => {
                    println!("⚠️ Trunk not found. Installing...");
                    let status = Command::new("cargo")
                        .args(["install", "trunk", "--locked"])
                        .status()?;
                    
                    if !status.success() {
                        println!("❌ Failed to install Trunk.");
                        println!("Please install it manually with: cargo install trunk --locked");
                    } else {
                        println!("✅ Trunk installed successfully");
                    }
                }
            }
            
            let leptos_templates = vec![
                "Counter - Simple counter with reactive state",
                "Router - Multi-page application with routing",
                "Todo - Todo application with filtering",
            ];
            
            let leptos_selection = Select::new()
                .with_prompt("✨ Which Leptos template would you like to use?")
                .items(&leptos_templates)
                .default(0)
                .interact()?;
                
            // Map selection to template name
            template = match leptos_selection {
                0 => "counter".to_string(),
                1 => "router".to_string(),
                2 => "todo".to_string(),
                _ => "counter".to_string(), // Default to counter if somehow none selected
            };
            
            println!("🔧 Creating new Leptos project with {} template...", template);
            
            // Use template_manager to apply the template instead of hardcoded functions
            let template_path = format!("client/leptos/{}", template);
            
            // Apply the template using the template manager
            template_manager::apply_template(&template_path, app_path, &name, additional_vars.clone())?;
            
        } else if framework_selected == "dioxus" {
            println!("📦 Creating Dioxus project with dioxus-cli");
            
            // Check if dioxus-cli is installed
            println!("🔍 Checking for dioxus-cli...");
            let dx_check = Command::new("dx")
                .arg("--version")
                .output();
                
            let dx_installed = match dx_check {
                Ok(output) => output.status.success(),
                Err(_) => false
            };
            
            if !dx_installed {
                println!("🔧 Installing dioxus-cli...");
                let install_status = Command::new("cargo")
                    .args(["install", "dioxus-cli"])
                    .status()?;
                    
                if !install_status.success() {
                    return Err(anyhow!("Failed to install dioxus-cli"));
                }
                println!("✅ dioxus-cli installed successfully");
            } else {
                println!("✅ dioxus-cli is already installed");
            }
            
            // Create project directory
            create_directory(app_path)?;
            
            // Let dioxus-cli handle the project creation with its own prompts
            println!("🔧 Creating Dioxus project...");
            let create_status = Command::new("dx")
                .args(["init"])
                .arg(".")
                .current_dir(app_path)
                .status()?;
                
            if !create_status.success() {
                return Err(anyhow!("Failed to create Dioxus project with dioxus-cli"));
            }
            
            // Ensure WASM target is installed for web projects
            println!("🔧 Ensuring WASM target is installed...");
            let _ = Command::new("rustup")
                .args(["target", "add", "wasm32-unknown-unknown"])
                .status();
            
            // Print success message with instructions
            println!("\n🎉 Project {} created successfully!", name);
            println!("\nNext steps:");
            println!("  cd {}", name);
            println!("  dx serve");
            
            return Ok(());
            
        } else if framework_selected == "tauri" {
            println!("📦 Creating Tauri project with create-tauri-app");
            
            // Create project directory
            create_directory(app_path)?;
            
            // Check if Node.js is installed
            println!("🔍 Checking for Node.js...");
            let node_check = Command::new("node")
                .arg("--version")
                .output();
                
            let node_installed = match node_check {
                Ok(output) => output.status.success(),
                Err(_) => false
            };
            
            if !node_installed {
                return Err(anyhow!("Node.js is required to create Tauri projects. Please install Node.js and try again."));
            }
            println!("✅ Node.js is installed");
            
            // Use npx to run create-tauri-app
            println!("🔧 Creating Tauri project...");
            let create_status = Command::new("npx")
                .args(["create-tauri-app@latest", "--ci"])
                .arg(".")
                .current_dir(app_path)
                .status()?;
                
            if !create_status.success() {
                return Err(anyhow!("Failed to create Tauri project with create-tauri-app"));
            }
            
            // Print success message with instructions
            println!("\n🎉 Project {} created successfully!", name);
            println!("\nNext steps:");
            println!("  cd {}", name);
            println!("  npm install");
            println!("  npm run tauri dev");
            
            return Ok(());
        } else {
            // If not Leptos, Dioxus, or Tauri, use the selected framework as the template
            template = framework_selected.to_string();
        }
    }

    // Check for required dependencies based on template
    check_dependencies(&template)?;

    // Apply the template using the template_manager
    if template.starts_with("client/") {
        // Template path is already fully qualified
        template_manager::apply_template(&template, app_path, &name, additional_vars.clone())?;
    } else if template == "embedded" {
        // For embedded template, prompt for microcontroller selection
        println!("📦 Creating embedded project for microcontrollers");
        
        // Ask if user wants to use a framework
        let use_framework = Select::new()
            .with_prompt("Do you want to use an embedded framework?")
            .items(&["No, use standard embedded template", "Yes, use Embassy framework"])
            .default(0)
            .interact()?;
            
        if use_framework == 1 {
            // User selected Embassy framework
            println!("📦 Creating Embassy project using cargo-embassy");
            
            // Check if cargo-embassy is installed
            println!("🔍 Checking for cargo-embassy...");
            let embassy_check = Command::new("cargo")
                .args(["install", "--list"])
                .output()?;
                
            let embassy_output = String::from_utf8_lossy(&embassy_check.stdout);
            let is_embassy_installed = embassy_output.contains("cargo-embassy");
                
            if is_embassy_installed {
                println!("✅ cargo-embassy is already installed");
            } else {
                println!("⚠️ cargo-embassy not found. Installing...");
                let status = Command::new("cargo")
                    .args(["install", "cargo-embassy"])
                    .status()?;
                
                if !status.success() {
                    println!("❌ Failed to install cargo-embassy.");
                    println!("Please install it manually with: cargo install cargo-embassy");
                    return Err(anyhow!("Failed to install cargo-embassy"));
                } else {
                    println!("✅ cargo-embassy installed successfully");
                }
            }
            
            // Get microcontroller target
            let mcu_targets = vec!["rp2040", "stm32f4", "nrf52840", "esp32c3"];
            let selection = Select::new()
                .with_prompt("Select microcontroller chip")
                .items(&mcu_targets)
                .default(0)
                .interact()?;
                
            let mcu_chip = mcu_targets[selection];
            println!("Using {} as the microcontroller chip", mcu_chip);
            
            // Create the project using cargo-embassy
            println!("🔄 Creating new Embassy project...");
            
            // Create a parent directory for the Embassy project
            let parent_dir = Path::new(".").join("embassy_temp");
            fs::create_dir_all(&parent_dir)?;
            
            // Run cargo-embassy init from the parent directory
            let status = Command::new("cargo")
                .args(["embassy", "init", "--chip", mcu_chip, &name])
                .current_dir(&parent_dir)
                .status()?;
                
            if !status.success() {
                println!("❌ Failed to create Embassy project.");
                return Err(anyhow!("Failed to create Embassy project"));
            }
            
            // Move the generated project to the target directory
            let project_dir = parent_dir.join(&name);
            if project_dir.exists() {
                // Copy all files from the generated project to the target directory
                let target_dir = Path::new(&name);
                if !target_dir.exists() {
                    fs::create_dir_all(target_dir)?;
                }
                
                copy_dir_all(&project_dir, target_dir)?;
                
                // Clean up the temporary directory
                fs::remove_dir_all(parent_dir)?;
                
                println!("🎉 Embassy project {} created successfully!", name);
                println!("\nNext steps:");
                println!("  cd {}", name);
                println!("  # Build the project");
                println!("  cargo build --release");
                println!("  # Run the project");
                println!("  cargo run --release");
                
                return Ok(());
            } else {
                println!("❌ Failed to create Embassy project.");
                return Err(anyhow!("Failed to create Embassy project: Project directory not found"));
            }
        } else {
            // Standard embedded template
            // Get microcontroller target
            let mcu_targets = vec!["rp2040", "stm32", "esp32", "arduino"];
            let selection = Select::new()
                .with_prompt("Select microcontroller target")
                .items(&mcu_targets)
                .default(0)
                .interact()?;
                
            let mcu_target = mcu_targets[selection];
            println!("Using {} as the microcontroller target", mcu_target);
            
            // Create target-specific dependencies string
            let mcu_target_deps = match mcu_target {
                "rp2040" => "rp2040-hal = \"0.9\"\nrp2040-boot2 = \"0.3\"\nusb-device = \"0.2\"\nusbd-serial = \"0.1\"",
                "stm32" => "stm32f4xx-hal = { version = \"0.17\", features = [\"stm32f411\"] }",
                "esp32" => "esp32-hal = \"0.16\"\nesp-backtrace = \"0.9\"\nesp-println = \"0.6\"",
                "arduino" => "arduino-hal = \"0.1\"\navr-device = \"0.5\"\nufmt = \"0.2\"",
                _ => "",
            };
            
            // Create variables for template substitution
            additional_vars = Some(serde_json::json!({
                "mcu_target": mcu_target,
                "mcu_target_deps": mcu_target_deps
            }));
            
            // Apply the template using the template manager
            template_manager::apply_template(&template, app_path, &name, additional_vars.clone())?;
            
            // Suggest installing the appropriate Rust target
            let rust_target = match mcu_target {
                "rp2040" => "thumbv6m-none-eabi",
                "stm32" => "thumbv7em-none-eabihf",
                "esp32" => "xtensa-esp32-none-elf",
                "arduino" => "avr-unknown-gnu-atmega328",
                _ => "thumbv6m-none-eabi",
            };
            
            println!("\nℹ️ You'll need to install the appropriate Rust target:");
            println!("  rustup target add {}", rust_target);
            
            match mcu_target {
                "rp2040" => {
                    println!("  cargo install probe-run");
                    println!("  cargo run --target {}", rust_target);
                },
                "esp32" => {
                    println!("  cargo install espflash");
                    println!("  cargo build --target {}", rust_target);
                    println!("  espflash flash --monitor target/{}/debug/{}", rust_target, name);
                },
                "arduino" => {
                    println!("  cargo install ravedude");
                    println!("  cargo run --target {}", rust_target);
                },
                _ => {
                    println!("  cargo build --target {}", rust_target);
                }
            }
        }
    } else if template == "counter" || template == "router" || template == "todo" {
        // For Leptos templates, prepend "client/leptos/"
        let template_path = format!("client/leptos/{}", template);
        template_manager::apply_template(&template_path, app_path, &name, additional_vars.clone())?;
    } else {
        // For other templates, use as is
        template_manager::apply_template(&template, app_path, &name, additional_vars.clone())?;
    }

    // Initialize git repository if requested
    if git {
        println!("🔄 Initializing git repository...");
        let status = Command::new("git")
            .args(["init"])
            .current_dir(app_path)
            .status()?;
        if !status.success() {
            return Err(anyhow!("Failed to initialize git repository"));
        }
        
        // Create .gitignore file
        let gitignore = r#"/target
/dist
/Cargo.lock
**/*.rs.bk
"#;
        std::fs::write(app_path.join(".gitignore"), gitignore)?;
        println!("✅ Git repository initialized");
    }

    // Build project if requested
    if build {
        println!("🔄 Building project...");
        let status = Command::new("cargo")
            .args(["build"])
            .current_dir(app_path)
            .status()?;
        if !status.success() {
            return Err(anyhow!("Failed to build project"));
        }
        println!("✅ Project built successfully");
    }

    // Print success message with instructions
    println!("\n🎉 Project {} created successfully!", name);
    println!("\nNext steps:");
    println!("  cd {}", name);
    
    // Provide appropriate next steps based on the template
    if template == "counter" || template == "router" || template == "todo" || template.contains("client/leptos/counter") || template.contains("client/leptos/router") || template.contains("client/leptos/todo") {
        println!("  trunk serve --open");
    } else if template.contains("client/dioxus") {
        println!("  dx serve --hot-reload true");
    } else if template.contains("client/tauri") {
        println!("  cargo tauri dev");
    } else if template == "embedded" {
        // For embedded template, provide target-specific instructions
        if let Some(vars) = &additional_vars {
            if let Some(mcu_target) = vars.get("mcu_target").and_then(|v| v.as_str()) {
                let rust_target = match mcu_target {
                    "rp2040" => "thumbv6m-none-eabi",
                    "stm32" => "thumbv7em-none-eabihf",
                    "esp32" => "xtensa-esp32-none-elf",
                    "arduino" => "avr-unknown-gnu-atmega328",
                    _ => "thumbv6m-none-eabi",
                };
                
                println!("\nℹ️ You'll need to install the appropriate Rust target:");
                println!("  rustup target add {}", rust_target);
                
                match mcu_target {
                    "rp2040" => {
                        println!("  cargo install probe-run");
                        println!("  cargo run --target {}", rust_target);
                    },
                    "esp32" => {
                        println!("  cargo install espflash");
                        println!("  cargo build --target {}", rust_target);
                        println!("  espflash flash --monitor target/{}/debug/{}", rust_target, name);
                    },
                    "arduino" => {
                        println!("  cargo install ravedude");
                        println!("  cargo run --target {}", rust_target);
                    },
                    _ => {
                        println!("  cargo build --target {}", rust_target);
                    }
                }
            }
        } else {
            println!("  cargo build");
        }
    } else {
        println!("  cargo run");
    }

    Ok(())
}

// Helper function to check and install required dependencies
fn check_dependencies(template: &str) -> Result<()> {
    // Check for wasm32-unknown-unknown target
    println!("🔍 Checking for wasm32-unknown-unknown target...");
    let wasm_check = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()?;
    
    let wasm_output = String::from_utf8_lossy(&wasm_check.stdout);
    if !wasm_output.contains("wasm32-unknown-unknown") {
        println!("⚠️ wasm32-unknown-unknown target not found. Installing...");
        let status = Command::new("rustup")
            .args(["target", "add", "wasm32-unknown-unknown"])
            .status()?;
        
        if !status.success() {
            println!("❌ Failed to install wasm32-unknown-unknown target.");
            println!("Please install it manually with: rustup target add wasm32-unknown-unknown");
        } else {
            println!("✅ wasm32-unknown-unknown target installed successfully");
        }
    } else {
        println!("✅ wasm32-unknown-unknown target is already installed");
    }
    
    // Check for trunk (needed for counter, router, todo templates)
    if template == "counter" || template == "router" || template == "todo" {
        println!("🔍 Checking for Trunk...");
        let trunk_check = Command::new("trunk")
            .arg("--version")
            .output();
        
        match trunk_check {
            Ok(_) => println!("✅ Trunk is already installed"),
            Err(_) => {
                println!("⚠️ Trunk not found. Installing...");
                let status = Command::new("cargo")
                    .args(["install", "trunk", "--locked"])
                    .status()?;
                
                if !status.success() {
                    println!("❌ Failed to install Trunk.");
                    println!("Please install it manually with: cargo install trunk --locked");
                } else {
                    println!("✅ Trunk installed successfully");
                }
            }
        }
    }
    
    Ok(())
}
