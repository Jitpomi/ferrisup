use std::path::Path;
use std::process::Command;
use anyhow::{Result, anyhow};
use dialoguer::{Select, Input};
use crate::templates;

// Helper function to create a directory
fn create_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

// Main execute function to handle Leptos project creation
pub fn execute(
    name: Option<&str>,
    template: Option<&str>,
    git: bool,
    build: bool,
    no_interactive: bool,
) -> Result<()> {
    // Get project name
    let name = match name {
        Some(name) => name.to_string(),
        None => {
            if no_interactive {
                return Err(anyhow!("Project name is required in non-interactive mode"));
            }
            Input::new()
                .with_prompt("Project name:")
                .default("my_app".to_string())
                .interact()?
        }
    };

    // Create project directory
    let app_path = Path::new(&name);
    if app_path.exists() {
        return Err(anyhow!("Directory {} already exists", name));
    }
    create_directory(app_path)?;

    // Get template
    let mut template = match template {
        Some(template) => template.to_string(),
        None => {
            if no_interactive {
                return Err(anyhow!("Template is required in non-interactive mode"));
            }
            
            // Get available templates from the templates module
            let templates_with_desc = templates::list_templates()?;
            let templates: Vec<&str> = templates_with_desc.iter().map(|(name, _)| name.as_str()).collect();
            
            let selection = Select::new()
                .with_prompt("Select a template:")
                .items(&templates)
                .default(0)
                .interact()?;
                
            templates[selection].to_string()
        }
    };
    
    // For client template, prompt for framework selection
    if template == "client" {
        println!("Template description: Custom template: client");
        println!("Using template: client");
        
        // Get client framework
        let frameworks = vec!["dioxus", "tauri", "leptos", "yew"];
        let selection = Select::new()
            .with_prompt("Select Rust client framework")
            .items(&frameworks)
            .default(0)
            .interact()?;
            
        let framework = frameworks[selection];
        
        // For Leptos, prompt for specific template type
        if framework == "leptos" {
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
                "SSR - Server-side rendered application",
                "Fullstack - Complete application with API endpoints",
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
                3 => "ssr".to_string(),
                4 => "fullstack".to_string(),
                _ => "counter".to_string(), // Default to counter if somehow none selected
            };
            
            println!("🔧 Creating new Leptos project with {} template...", template);
        } else if framework == "dioxus" {
            // Use Dioxus CLI for project creation
            println!("📦 Using Dioxus CLI to bootstrap the project");
            
            // Check if Dioxus CLI is installed
            println!("🔍 Checking if Dioxus CLI is installed...");
            
            // Try with 'dioxus' command first (newer versions)
            println!("🔍 Attempting to create Dioxus project");
            let dioxus_result = Command::new("dioxus")
                .args(["create", &name])
                .status();
                
            match dioxus_result {
                Ok(status) if status.success() => {
                    println!("✅ Dioxus project created successfully using 'dioxus create'");
                },
                _ => {
                    println!("⚠️ Error executing 'dioxus' command: {}", dioxus_result.err().map_or_else(|| "Unknown error".to_string(), |e| e.to_string()));
                    
                    // Try with 'dx' command next
                    println!("🔄 Trying with 'dx' command instead...");
                    let dx_new_result = Command::new("dx")
                        .args(["new", &name])
                        .status();
                        
                    match dx_new_result {
                        Ok(status) if status.success() => {
                            println!("✅ Dioxus project created successfully using 'dx new'");
                        },
                        _ => {
                            // Print error status without consuming dx_new_result
                            match &dx_new_result {
                                Ok(status) => println!("⚠️ 'dx new' command failed with status: {}", status),
                                Err(e) => println!("⚠️ 'dx new' command failed with error: {}", e),
                            }
                            
                            // Final attempt with older dx CLI format
                            println!("🔄 Trying older dx CLI format...");
                            let dx_create_result = Command::new("dx")
                                .args(["create", &name])
                                .status();
                                
                            match dx_create_result {
                                Ok(status) if status.success() => {
                                    println!("✅ Dioxus project created successfully using 'dx create'");
                                },
                                _ => {
                                    // Print error status without consuming dx_create_result
                                    match &dx_create_result {
                                        Ok(status) => println!("⚠️ 'dx create' command failed with status: {}", status),
                                        Err(e) => println!("⚠️ 'dx create' command failed with error: {}", e),
                                    }
                                    
                                    println!("⚠️ All Dioxus CLI commands failed");
                                    println!("🔄 Falling back to manual Dioxus project creation...");
                                    create_manual_dioxus_project(&app_path, &name)?;
                                }
                            }
                        }
                    }
                }
            }
            
            // Ensure WASM target is installed
            println!("🔧 Ensuring WASM target is installed...");
            let _ = Command::new("rustup")
                .args(["target", "add", "wasm32-unknown-unknown"])
                .status();
            
            // Initialize git repository if requested
            if git {
                println!("🔄 Initializing git repository...");
                let status = Command::new("git")
                    .args(["init"])
                    .current_dir(app_path)
                    .status()?;
                if !status.success() {
                    println!("⚠️ Failed to initialize git repository");
                } else {
                    println!("✅ Git repository initialized");
                }
            }
            
            // Print success message with instructions
            println!("\n🎉 Project {} created successfully!", name);
            println!("\nNext steps:");
            println!("  cd {}", name);
            println!("  dx serve --hot-reload");
            
            return Ok(());
        } else if framework == "tauri" {
            // Use create-tauri-app for project creation
            println!("📦 Using create-tauri-app to bootstrap the project");
            
            // First, prompt for frontend language
            let frontend_languages = vec![
                "TypeScript/JavaScript",
                "Rust (Experimental)"
            ];
            
            let language_selection = Select::new()
                .with_prompt("Choose which language to use for your Tauri frontend")
                .items(&frontend_languages)
                .default(0)
                .interact()?;
            
            let frontend_language = match language_selection {
                0 => "js",
                1 => "rust",
                _ => "js",
            };
            
            // For JavaScript frontend, prompt for package manager and UI framework
            let mut package_manager = "npm";
            let mut ui_framework = "react";
            let mut ui_flavor = "ts";
            
            if frontend_language == "js" {
                // Prompt for package manager
                let package_managers = vec![
                    "pnpm",
                    "yarn",
                    "npm",
                    "bun"
                ];
                
                let package_manager_selection = Select::new()
                    .with_prompt("Choose your package manager")
                    .items(&package_managers)
                    .default(0)
                    .interact()?;
                
                package_manager = package_managers[package_manager_selection];
                
                // Prompt for UI framework
                let js_framework_options = vec![
                    "React (JavaScript/TypeScript)",
                    "Vue (JavaScript/TypeScript)",
                    "Svelte (JavaScript/TypeScript)",
                    "Preact (JavaScript/TypeScript)",
                    "Solid (JavaScript/TypeScript)",
                    "Qwik (JavaScript/TypeScript)",
                    "Angular (JavaScript/TypeScript)",
                    "Vanilla (JavaScript/TypeScript)",
                    "None"
                ];
                
                let js_selection = Select::new()
                    .with_prompt("Choose your UI template")
                    .items(&js_framework_options)
                    .default(0)
                    .interact()?;
                
                ui_framework = match js_selection {
                    0 => "react",
                    1 => "vue",
                    2 => "svelte",
                    3 => "preact",
                    4 => "solid",
                    5 => "qwik",
                    6 => "angular",
                    7 => "vanilla",
                    8 => "none",
                    _ => "react", // Default to React
                };
                
                // If a UI framework is selected, prompt for JavaScript or TypeScript
                if ui_framework != "none" {
                    let flavor_options = vec![
                        "TypeScript",
                        "JavaScript"
                    ];
                    
                    let flavor_selection = Select::new()
                        .with_prompt("Choose JavaScript or TypeScript")
                        .items(&flavor_options)
                        .default(0)
                        .interact()?;
                    
                    ui_flavor = match flavor_selection {
                        0 => "ts",
                        1 => "js",
                        _ => "ts", // Default to TypeScript
                    };
                }
            } else if frontend_language == "rust" {
                // For Rust frontend, prompt for Rust UI framework
                let rust_ui_frameworks = vec![
                    "Vanilla (No framework)",
                    "Yew",
                    "Leptos",
                    "Sycamore"
                ];
                
                let rust_ui_selection = Select::new()
                    .with_prompt("Choose your Rust UI framework")
                    .items(&rust_ui_frameworks)
                    .default(0)
                    .interact()?;
                
                ui_framework = match rust_ui_selection {
                    0 => "vanilla",
                    1 => "yew",
                    2 => "leptos",
                    3 => "sycamore",
                    _ => "vanilla", // Default to Vanilla
                };
            }
            
            // Check if create-tauri-app is installed
            println!("🔍 Checking for create-tauri-app...");
            let tauri_check = Command::new("create-tauri-app")
                .arg("--version")
                .output();
                
            let cta_installed = match tauri_check {
                Ok(_) => {
                    println!("✅ create-tauri-app is already installed");
                    true
                },
                Err(_) => {
                    println!("⚠️ create-tauri-app not found. Installing...");
                    let status = Command::new("npm")
                        .args(["install", "-g", "create-tauri-app"])
                        .status()?;
                        
                    if !status.success() {
                        println!("❌ Failed to install create-tauri-app.");
                        println!("Please install it manually with: npm install -g create-tauri-app");
                        return Err(anyhow!("Failed to install create-tauri-app"));
                    } else {
                        println!("✅ create-tauri-app installed successfully");
                        true
                    }
                }
            };
            
            if cta_installed {
                // Remove the app directory if it exists to allow create-tauri-app to create it
                if app_path.exists() {
                    std::fs::remove_dir_all(app_path)?;
                }
                
                // Build the create-tauri-app command with all the options
                println!("🔧 Creating new Tauri project...");
                let mut command = Command::new("create-tauri-app");
                command.arg(&name);
                
                // Add template options based on selections
                if frontend_language == "js" {
                    command.arg("--template").arg(format!("{}-{}", ui_framework, ui_flavor));
                    command.arg("--manager").arg(package_manager);
                } else if frontend_language == "rust" {
                    command.arg("--template").arg(ui_framework);
                }
                
                // Execute the command
                let status = command.status()?;
                
                if !status.success() {
                    return Err(anyhow!("Failed to create Tauri project"));
                }
                
                println!("✅ Tauri project created successfully");
                
                // Initialize git repository if requested
                if git {
                    println!("🔄 Initializing git repository...");
                    let status = Command::new("git")
                        .args(["init"])
                        .current_dir(app_path)
                        .status()?;
                    if !status.success() {
                        println!("⚠️ Failed to initialize git repository");
                    } else {
                        println!("✅ Git repository initialized");
                    }
                }
                
                // Print success message with instructions
                println!("\n🎉 Project {} created successfully!", name);
                println!("\nNext steps:");
                println!("  cd {}", name);
                
                if frontend_language == "js" {
                    println!("  {} install", package_manager);
                    println!("  {} run tauri dev", package_manager);
                } else {
                    println!("  cargo tauri dev");
                }
                
                return Ok(());
            } else {
                return Err(anyhow!("Failed to set up create-tauri-app"));
            }
        } else {
            // If not Leptos, Dioxus, or Tauri, use the selected framework as the template
            template = framework.to_string();
        }
    }

    // Check for required dependencies based on template
    check_dependencies(&template)?;

    // Create project based on template
    match template.as_str() {
        "counter" => create_leptos_counter_project(app_path)?,
        "router" => create_leptos_router_project(app_path)?,
        "todo" => create_leptos_todo_project(app_path)?,
        "ssr" => create_leptos_ssr_project(app_path)?,
        "fullstack" => create_leptos_fullstack_project(app_path)?,
        _ => return Err(anyhow!("Unknown template: {}", template)),
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
    
    match template.as_str() {
        "counter" | "router" | "todo" => {
            println!("  trunk serve --open");
        },
        "ssr" | "fullstack" => {
            println!("  cargo leptos watch");
        },
        _ => {}
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
    
    // Check for cargo-leptos (needed for ssr and fullstack templates)
    if template == "ssr" || template == "fullstack" {
        println!("🔍 Checking for cargo-leptos...");
        let cargo_install_output = Command::new("cargo")
            .args(["install", "--list"])
            .output()
            .expect("Failed to execute cargo install --list");
        
        let cargo_install_output_str = String::from_utf8_lossy(&cargo_install_output.stdout);
        
        if !cargo_install_output_str.contains("cargo-leptos") {
            println!("⚠️ cargo-leptos not found. Installing...");
            let install_result = Command::new("cargo")
                .args(["install", "cargo-leptos"])
                .status()
                .expect("Failed to install cargo-leptos");
            
            if !install_result.success() {
                println!("❌ Failed to install cargo-leptos.");
                println!("Please install it manually with: cargo install cargo-leptos");
            } else {
                println!("✅ cargo-leptos installed successfully");
            }
        } else {
            println!("✅ cargo-leptos is already installed");
        }
    }
    
    Ok(())
}

// Helper function to create a Leptos full-stack project with API endpoints
pub fn create_leptos_fullstack_project(app_path: &Path) -> Result<()> {
    println!("📝 Creating a Leptos full-stack project with API endpoints...");
    
    let app_name = app_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("leptos_app");
    
    // Create src directory
    let src_dir = app_path.join("src");
    create_directory(&src_dir)?;
    
    // Create Cargo.toml with Leptos dependencies including SSR features
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
axum = {{ version = "0.6", optional = true }}
console_error_panic_hook = "0.1"
console_log = "1.0"
leptos = {{ version = "0.5", features = ["nightly"] }}
leptos_axum = {{ version = "0.5", optional = true }}
leptos_meta = {{ version = "0.5", features = ["nightly"] }}
leptos_router = {{ version = "0.5", features = ["nightly"] }}
log = "0.4"
simple_logger = "4"
tokio = {{ version = "1", features = ["full"], optional = true }}
tower = {{ version = "0.4", optional = true }}
tower-http = {{ version = "0.4", features = ["fs"], optional = true }}
wasm-bindgen = "=0.2.87"
serde = {{ version = "1.0", features = ["derive"] }}

[features]
hydrate = [
    "leptos/hydrate",
    "leptos_meta/hydrate",
    "leptos_router/hydrate",
]
ssr = [
    "dep:axum",
    "dep:tokio",
    "dep:tower",
    "dep:tower-http",
    "dep:leptos_axum",
    "leptos/ssr",
    "leptos_meta/ssr",
    "leptos_router/ssr",
]

[package.metadata.cargo-all-features]
denylist = ["axum", "tokio", "tower", "tower-http", "leptos_axum"]
skip_feature_sets = [["ssr", "hydrate"]]

[profile.release]
opt-level = 'z'
lto = true
codegen-units = 1
"#, app_name);

    std::fs::write(app_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create lib.rs with app component and API integrations
    let lib_rs = r#"use leptos::*;
use leptos_meta::*;
use leptos_router::*;

// API models
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Task {
    id: usize,
    title: String,
    completed: bool,
}

// Server functions
#[server(GetTasks, "/api")]
pub async fn get_tasks() -> Result<Vec<Task>, ServerFnError> {
    // In a real app, this would fetch from a database
    Ok(vec![
        Task { id: 1, title: "Learn Leptos".to_string(), completed: true },
        Task { id: 2, title: "Build a full-stack app".to_string(), completed: false },
        Task { id: 3, title: "Share with the community".to_string(), completed: false },
    ])
}

#[server(AddTask, "/api")]
pub async fn add_task(title: String) -> Result<Task, ServerFnError> {
    // In a real app, this would insert into a database
    Ok(Task {
        id: 4, // In a real app, this would be generated
        title,
        completed: false,
    })
}

#[server(ToggleTask, "/api")]
pub async fn toggle_task(id: usize) -> Result<(), ServerFnError> {
    // In a real app, this would update a database
    Ok(())
}

// Main app component
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        <Stylesheet id="leptos" href="/pkg/leptos_app.css"/>

        // sets the document title
        <Title text="Leptos Full-Stack App"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/tasks" view=TasksPage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

// Home page component
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="container">
            <header>
                <h1>"Welcome to Leptos Full-Stack!"</h1>
                <p>"This is a simple example of a Leptos application with server functions."</p>
            </header>
            
            <div class="nav-links">
                <A href="/tasks" class="button">"View Tasks"</A>
            </div>
            
            <footer>
                <p>"Built with Leptos - A Rust framework for building web applications"</p>
            </footer>
        </div>
    }
}

// Tasks page component
#[component]
fn TasksPage() -> impl IntoView {
    let tasks = create_resource(|| (), |_| get_tasks());
    let (new_task, set_new_task) = create_signal("".to_string());
    
    let add_task_action = create_server_action::<AddTask>();
    let toggle_task_action = create_server_action::<ToggleTask>();
    
    // Refresh tasks when actions complete
    let add_task_pending = add_task_action.pending();
    let toggle_task_pending = toggle_task_action.pending();
    
    create_effect(move |_| {
        if !add_task_pending.get() && add_task_action.version().get() > 0 {
            tasks.refetch();
        }
    });
    
    create_effect(move |_| {
        if !toggle_task_pending.get() && toggle_task_action.version().get() > 0 {
            tasks.refetch();
        }
    });
    
    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let task = new_task.get();
        if !task.is_empty() {
            add_task_action.dispatch(AddTask { title: task });
            set_new_task.set("".to_string());
        }
    };
    
    view! {
        <div class="container">
            <header>
                <h1>"Task Manager"</h1>
                <p>"A simple task manager with server functions"</p>
            </header>
            
            <div class="task-form">
                <form on:submit=handle_submit>
                    <input
                        type="text"
                        placeholder="Add a new task"
                        prop:value=move || new_task.get()
                        on:input=move |ev| set_new_task.set(event_target_value(&ev))
                    />
                    <button type="submit">"Add Task"</button>
                </form>
            </div>
            
            <div class="task-list">
                <Suspense fallback=move || view! { <p>"Loading tasks..."</p> }>
                    {move || {
                        tasks.get().map(|tasks| match tasks {
                            Err(e) => view! { <p>"Error loading tasks: " {e.to_string()}</p> }.into_view(),
                            Ok(tasks) => {
                                if tasks.is_empty() {
                                    view! { <p>"No tasks yet. Add one above!"</p> }.into_view()
                                } else {
                                    view! {
                                        <ul>
                                            {tasks.into_iter().map(|task| {
                                                let id = task.id;
                                                view! {
                                                    <li class:completed=task.completed>
                                                        <label>
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=task.completed
                                                                on:change=move |_| {
                                                                    toggle_task_action.dispatch(ToggleTask { id });
                                                                }
                                                            />
                                                            <span>{task.title}</span>
                                                        </label>
                                                    </li>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </ul>
                                    }.into_view()
                                }
                            }
                        })
                    }}
                </Suspense>
            </div>
            
            <div class="nav-links">
                <A href="/" class="button">"Back to Home"</A>
            </div>
        </div>
    }
}

// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // Set a response status
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    view! {
        <div class="container">
            <h1>"404 - Not Found"</h1>
            <p>"The page you requested could not be found."</p>
            <div class="nav-links">
                <A href="/" class="button">"Back to Home"</A>
            </div>
        </div>
    }
}
"#;

    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    // Create main.rs for the server
    let main_rs = r#"use axum::{
    body::Body as AxumBody,
    extract::State,
    http::Request,
    response::{IntoResponse, Response as AxumResponse},
    routing::post,
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_app::*;

#[tokio::main]
async fn main() {
    // Set up logging
    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // build our app with a route
    let app = Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .leptos_routes(
            leptos_options.clone(),
            routes,
            App,
        )
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    // run our app with hyper
    log::info!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn file_and_error_handler(
    path: Path<String>,
    options: Extension<Arc<LeptosOptions>>,
) -> Response {
    let root = options.site_root.clone();
    let path = path.0;

    // try to serve a static file
    if let Ok(file) = leptos_axum::handle_static_file(format!("{root}/{path}")).await {
        return file.into_response();
    }

    // if that fails, render the app and return the response
    let handler = leptos_axum::render_app_to_stream(options.to_owned(), |cx| view! { cx, <App/> });
    handler.await.into_response()
}
"#;

    std::fs::write(src_dir.join("main.rs"), main_rs)?;
    
    // Create CSS file
    let css_file = r#"/* Global styles */
html, body {
    margin: 0;
    padding: 0;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    color: #333;
    background-color: #f5f5f5;
}

.container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
}

header {
    text-align: center;
    margin-bottom: 2rem;
}

h1 {
    color: #2563eb;
    margin-bottom: 0.5rem;
}

p {
    color: #64748b;
    margin-bottom: 1.5rem;
}

.nav-links {
    display: flex;
    justify-content: center;
    margin: 2rem 0;
}

.button {
    display: inline-block;
    background-color: #3b82f6;
    color: white;
    padding: 0.75rem 1.5rem;
    border-radius: 0.25rem;
    text-decoration: none;
    font-weight: 500;
    transition: background-color 0.2s;
}

.button:hover {
    background-color: #2563eb;
}

/* Task styles */
.task-form {
    margin-bottom: 2rem;
}

.task-form form {
    display: flex;
}

.task-form input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 0.25rem 0 0 0.25rem;
    font-size: 1rem;
}

.task-form button {
    padding: 0.75rem 1.5rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0 0.25rem 0.25rem 0;
    cursor: pointer;
    font-size: 1rem;
    transition: background-color 0.2s;
}

.task-form button:hover {
    background-color: #2563eb;
}

.task-list ul {
    list-style: none;
    padding: 0;
    margin: 0;
}

.task-list li {
    padding: 1rem;
    border-bottom: 1px solid #eee;
    background-color: white;
    margin-bottom: 0.5rem;
    border-radius: 0.25rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.task-list li.completed span {
    text-decoration: line-through;
    color: #999;
}

.task-list label {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
}

.task-list input[type="checkbox"] {
    height: 1.2rem;
    width: 1.2rem;
}

footer {
    text-align: center;
    margin-top: 3rem;
    color: #777;
    font-size: 0.9rem;
}
"#;
    std::fs::write(app_path.join("assets").join("leptos_app.css"), css_file)?;
    
    // Create Cargo.toml file
    let cargo_config = r#"[build]
target = "wasm32-unknown-unknown"
"#;
    let config_dir = app_path.join(".cargo");
    create_directory(&config_dir)?;
    std::fs::write(config_dir.join("config"), cargo_config)?;
    
    // Create a Leptos.toml configuration file
    let leptos_toml = format!(r#"site_root = "target/site"
site_pkg_dir = "pkg"
output_name = "{}"
site_addr = "127.0.0.1:3000"
reload_port = 3001
end_reload_port = 3030
browserquery = "defaults"
style_file = "assets/leptos_app.css"
watch = ["src/**/*.rs", "assets/**/*"]
"#, app_name);

    std::fs::write(app_path.join("Leptos.toml"), leptos_toml)?;
    
    // Create README.md
    let readme = format!(r#"# {} - Leptos Full-Stack App

A full-stack web application built with [Leptos](https://github.com/leptos-rs/leptos), demonstrating server functions and API integration.

## Features

- Server-side rendering (SSR) with hydration
- Client-side navigation with Leptos Router
- Server functions for API calls
- Task management functionality
- Responsive design

## Prerequisites

- Rust and Cargo
- Leptos CLI: `cargo install cargo-leptos`
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

## Running the Application

```bash
# Navigate to the project directory
cd {}

# Start the development server
cargo leptos watch
```

This will start a local development server at http://127.0.0.1:3000 and automatically rebuild and reload when you make changes.

## Building for Production

```bash
cargo leptos build --release
```

This will create optimized build files in the `target/site` directory.

## Project Structure

- `src/lib.rs`: Contains the main Leptos components, server functions, and client-side logic
- `src/main.rs`: Server implementation with Axum
- `assets/leptos_app.css`: Global styles for the application
- `Leptos.toml`: Configuration for the Leptos build system

## Learn More

- [Leptos Documentation](https://leptos.dev/)
- [Leptos Server Functions](https://book.leptos.dev/server/server_functions.html)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
"#, app_name, app_name);
    
    std::fs::write(app_path.join("README.md"), readme)?;
    
    println!("✅ Successfully created a Leptos full-stack project with API endpoints");
    
    Ok(())
}

// Helper function to create a simple Leptos counter project
pub fn create_leptos_counter_project(app_path: &Path) -> Result<()> {
    println!("📝 Creating a Leptos counter project...");
    
    let app_name = app_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("leptos_app");
    
    // Create src directory
    let src_dir = app_path.join("src");
    create_directory(&src_dir)?;
    
    // Create Cargo.toml with Leptos dependencies
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = {{ version = "0.5", features = ["csr"] }}
console_log = "1.0"
log = "0.4"
console_error_panic_hook = "0.1"

[profile.release]
opt-level = 'z'
codegen-units = 1
lto = true
"#, app_name);

    std::fs::write(app_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create lib.rs with counter component
    let lib_rs = r#"use leptos::*;

// Counter component
#[component]
pub fn Counter() -> impl IntoView {
    // Create a reactive signal with the initial value of 0
    let (count, set_count) = create_signal(0);
    
    // Functions to update the count
    let increment = move |_| set_count.update(|count| *count += 1);
    let decrement = move |_| set_count.update(|count| *count -= 1);
    let reset = move |_| set_count.set(0);

    view! {
        <div class="counter-container">
            <h1>"Leptos Counter Example"</h1>
            <p>"A simple counter built with Leptos"</p>
            
            <div class="counter">
                <div class="count-display">
                    <span class="count">{count}</span>
                </div>
                
                <div class="buttons">
                    <button on:click=decrement>"-1"</button>
                    <button on:click=reset class="reset">"Reset"</button>
                    <button on:click=increment>"+1"</button>
                </div>
            </div>
            
            <div class="counter-info">
                <p>"Current count: " <strong>{count}</strong></p>
                <p class="description">
                    "This counter demonstrates reactive state management in Leptos."
                </p>
            </div>
        </div>
    }
}

// Main app component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <main>
            <Counter/>
        </main>
    }
}
"#;

    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    // Create main.rs
    let main_rs = format!(r#"use leptos::*;
use {}::App;

fn main() {{
    // Set up logging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    
    mount_to_body(|| view! {{ <App/> }})
}}
"#, app_name);

    std::fs::write(src_dir.join("main.rs"), main_rs)?;
    
    // Create index.html
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link data-trunk rel="rust" data-wasm-opt="z"/>
    <link data-trunk rel="css" href="style.css"/>
    <title>Leptos Counter</title>
</head>
<body>
    <!-- This is where your Leptos app will be mounted -->
</body>
</html>
"#;
    std::fs::write(app_path.join("index.html"), index_html)?;
    
    // Create style.css
    let style_css = r#"html, body {
    margin: 0;
    padding: 0;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    background-color: #f5f5f5;
    color: #333;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
}

main {
    width: 100%;
    max-width: 500px;
}

.counter-container {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    padding: 2rem;
    text-align: center;
}

h1 {
    text-align: center;
    color: #2563eb;
    margin-top: 0;
}

p {
    color: #64748b;
    margin-bottom: 1.5rem;
}

.counter {
    margin: 2rem 0;
}

button {
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 8px 16px;
    cursor: pointer;
    font-size: 16px;
    transition: background-color 0.2s;
}

button:hover {
    background-color: #2563eb;
}

.counter-info {
    margin-top: 2rem;
    padding-top: 1.5rem;
    border-top: 1px solid #e2e8f0;
}

.counter-info p {
    margin: 0.5rem 0;
}

.description {
    font-size: 0.9rem;
    color: #94a3b8;
}
"#;
    std::fs::write(app_path.join("style.css"), style_css)?;
    
    // Create README.md
    let readme = format!(r#"# {} - Leptos Counter

A Leptos application with a simple counter.

## Features

- Increment, decrement, and reset counter
- Reactive state updates
- Clean, responsive UI

## Prerequisites

- Rust and Cargo
- Trunk: `cargo install trunk --locked`
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

## Development

```bash
trunk serve --open
```

This will start a development server and open your application in a browser.

## Building for Production

```bash
trunk build --release
```

This will create optimized WebAssembly files in the `dist` directory.

## Project Structure

- `src/lib.rs`: Contains the main application logic and components
- `src/main.rs`: Entry point that mounts the application to the DOM
- `index.html`: HTML template with Trunk directives
- `style.css`: Styling for the application

## Learn More

- [Leptos Documentation](https://leptos.dev/)
- [Reactive Primitives in Leptos](https://docs.rs/leptos/latest/leptos/primitives/index.html)
"#, app_name);
    
    std::fs::write(app_path.join("README.md"), readme)?;
    
    println!("✅ Successfully created a Leptos counter project");
    
    Ok(())
}

// Helper function to create a Leptos project with routing
pub fn create_leptos_router_project(app_path: &Path) -> Result<()> {
    println!("📝 Creating a Leptos project with routing...");
    
    let app_name = app_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("leptos_app");
    
    // Prompt for router feature
    let router_features = vec!["csr", "hydrate"];
    let router_feature = Select::new()
        .with_prompt("Which feature would you like to enable for leptos_router?")
        .items(&router_features)
        .default(0)
        .interact()?;
    
    let selected_feature = router_features[router_feature];
    println!("Using feature: {}", selected_feature);
    
    // Create src directory
    let src_dir = app_path.join("src");
    create_directory(&src_dir)?;
    
    // Create Cargo.toml with Leptos dependencies including routing
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = {{ version = "0.5", features = ["{}"] }}
leptos_router = {{ version = "0.5", features = ["{}"] }}
console_log = "1.0"
log = "0.4"
console_error_panic_hook = "0.1"

[profile.release]
opt-level = 'z'
codegen-units = 1
lto = true
"#, app_name, selected_feature, selected_feature);

    std::fs::write(app_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create main.rs with routing
    let main_rs = format!(r#"use leptos::*;
use leptos_router::*;
use {}::App;

fn main() {{
    // Set up logging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    
    mount_to_body(|| view! {{ <App/> }})
}}
"#, app_name);

    std::fs::write(src_dir.join("main.rs"), main_rs)?;
    
    // Create lib.rs with routing components
    let lib_rs = r#"use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/about" view=AboutPage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"Welcome to Leptos Router"</h1>
            <p>"This is a simple example of routing with Leptos."</p>
            <A href="/about">"About"</A>
        </div>
    }
}

#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"About"</h1>
            <p>"This is the about page."</p>
            <A href="/">"Home"</A>
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    // Set a response status
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    view! {
        <div class="page">
            <h1>"404"</h1>
            <p>"Page not found"</p>
            <A href="/">"Go Home"</A>
        </div>
    }
}
"#;

    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    // Create index.html
    let index_html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Leptos Router</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        .container {{
            padding: 20px;
            border-radius: 8px;
            background-color: #f9f9f9;
        }}
        a {{
            display: inline-block;
            margin-top: 20px;
            padding: 10px 20px;
            background-color: #4a6cf7;
            color: white;
            text-decoration: none;
            border-radius: 4px;
        }}
        a:hover {{
            background-color: #3a5ce7;
        }}
    </style>
</head>
<body>
    <div id="root"></div>
</body>
</html>
"#, app_name);

    std::fs::write(app_path.join("index.html"), index_html)?;
    
    // Create README.md with instructions
    let mut readme_content = format!(r#"# {}

A Leptos application with routing.

## Features

- Multi-page application with routing
- Client-side navigation
- Clean, responsive UI

## Prerequisites

- Rust and Cargo
- Trunk: `cargo install trunk --locked`
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

## Development

```bash
trunk serve --open
```

This will start a development server and open your application in a browser.

## Building for Production

```bash
trunk build --release
```

This will create optimized WebAssembly files in the `dist` directory.

## Project Structure

- `src/lib.rs`: Contains the main application logic and components
- `src/main.rs`: Entry point that mounts the application to the DOM
- `index.html`: HTML template with Trunk directives
- `style.css`: Styling for the application

## Learn More

- [Leptos Documentation](https://leptos.dev/)
- [Leptos GitHub Repository](https://github.com/leptos-rs/leptos)
"#, app_name);

    // Add note about hydrate feature if selected
    if selected_feature == "hydrate" {
        readme_content.push_str(r#"
## Important Note About Hydration

You've selected the "hydrate" feature for this project, which is designed for server-side rendering with client-side hydration. This requires:

1. A server-side rendering environment
2. Proper configuration for hydration

For simple client-side development with Trunk, you may want to switch to the "csr" feature:

1. Edit `Cargo.toml` and change:
   ```toml
   leptos = { version = "0.5", features = ["hydrate"] }
   leptos_router = { version = "0.5", features = ["hydrate"] }
   ```
   to:
   ```toml
   leptos = { version = "0.5", features = ["csr"] }
   leptos_router = { version = "0.5", features = ["csr"] }
   ```

2. Then run `trunk serve --open` as usual
"#);
    }

    std::fs::write(app_path.join("README.md"), readme_content)?;
    
    // Create .gitignore
    let gitignore = r#"# Generated by Cargo
/target/
/dist/

# Cargo.lock from the project
Cargo.lock

# Remove Cargo.lock from gitignore if creating an executable, keep it for libraries
**/*.rs.bk

# Trunk artifacts
.trunk/
"#;

    std::fs::write(app_path.join(".gitignore"), gitignore)?;
    
    println!("✅ Successfully created a Leptos router project");
    
    Ok(())
}

// Helper function to create a Leptos todo application
pub fn create_leptos_todo_project(app_path: &Path) -> Result<()> {
    println!("📝 Creating a Leptos todo application...");
    
    let app_name = app_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("leptos_app");
    
    // Create src directory
    let src_dir = app_path.join("src");
    create_directory(&src_dir)?;
    
    // Create Cargo.toml with Leptos dependencies
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = {{ version = "0.5", features = ["csr"] }}
console_log = "1.0"
log = "0.4"
console_error_panic_hook = "0.1"
uuid = {{ version = "1.4", features = ["v4", "js"] }}

[profile.release]
opt-level = 'z'
codegen-units = 1
lto = true
"#, app_name);

    std::fs::write(app_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create lib.rs with todo app
    let lib_rs = r#"use leptos::*;
use uuid::Uuid;

// Todo item model
#[derive(Debug, Clone, PartialEq, Eq)]
struct Todo {
    id: String,
    text: String,
    completed: bool,
}

// Filter options for todos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    fn matches(&self, todo: &Todo) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !todo.completed,
            Filter::Completed => todo.completed,
        }
    }
}

// Main app component
#[component]
pub fn App() -> impl IntoView {
    // Create reactive signals for todos and filter
    let (todos, set_todos) = create_signal(vec![
        Todo { id: Uuid::new_v4().to_string(), text: "Learn Leptos".to_string(), completed: false },
        Todo { id: Uuid::new_v4().to_string(), text: "Build a todo app".to_string(), completed: false },
        Todo { id: Uuid::new_v4().to_string(), text: "Profit!".to_string(), completed: false },
    ]);
    
    let (filter, set_filter) = create_signal(Filter::All);
    let (new_todo_text, set_new_todo_text) = create_signal(String::new());
    
    // Derived signal for filtered todos
    let filtered_todos = move || {
        todos.get()
            .iter()
            .filter(|todo| filter.get().matches(todo))
            .cloned()
            .collect::<Vec<_>>()
    };
    
    // Count of active todos
    let active_count = move || {
        todos.get()
            .iter()
            .filter(|todo| !todo.completed)
            .count()
    };
    
    // Add a new todo
    let add_todo = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        let text = new_todo_text.get();
        if !text.is_empty() {
            set_todos.update(|todos| {
                todos.push(Todo {
                    id: Uuid::new_v4().to_string(),
                    text,
                    completed: false,
                });
            });
            set_new_todo_text.set(String::new());
        }
    };
    
    // Toggle a todo's completed status
    let toggle_todo = move |id: String| {
        set_todos.update(|todos| {
            if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
                todo.completed = !todo.completed;
            }
        });
    };
    
    // Delete a todo
    let delete_todo = move |id: String| {
        set_todos.update(|todos| {
            todos.retain(|t| t.id != id);
        });
    };
    
    // Toggle all todos
    let toggle_all = move |_| {
        let all_completed = todos.get().iter().all(|todo| todo.completed);
        set_todos.update(|todos| {
            for todo in todos.iter_mut() {
                todo.completed = !all_completed;
            }
        });
    };
    
    // Clear completed todos
    let clear_completed = move |_| {
        set_todos.update(|todos| {
            todos.retain(|todo| !todo.completed);
        });
    };
    
    view! {
        <div class="todo-app">
            <h1>"Leptos Todo App"</h1>
            
            <form on:submit=add_todo class="todo-form">
                <input
                    type="text"
                    placeholder="What needs to be done?"
                    prop:value=move || new_todo_text.get()
                    on:input=move |ev| set_new_todo_text.set(event_target_value(&ev))
                />
                <button type="submit">"Add"</button>
            </form>
            
            <div class="todo-controls">
                <button 
                    class="toggle-all"
                    on:click=toggle_all
                    disabled=move || todos.get().is_empty()
                >
                    "Toggle All"
                </button>
                
                <div class="filters">
                    <button 
                        class=move || if filter.get() == Filter::All { "active" } else { "" }
                        on:click=move |_| set_filter.set(Filter::All)
                    >
                        "All"
                    </button>
                    <button 
                        class=move || if filter.get() == Filter::Active { "active" } else { "" }
                        on:click=move |_| set_filter.set(Filter::Active)
                    >
                        "Active"
                    </button>
                    <button 
                        class=move || if filter.get() == Filter::Completed { "active" } else { "" }
                        on:click=move |_| set_filter.set(Filter::Completed)
                    >
                        "Completed"
                    </button>
                </div>
                
                <button 
                    class="clear-completed"
                    on:click=clear_completed
                    disabled=move || !todos.get().iter().any(|todo| todo.completed)
                >
                    "Clear completed"
                </button>
            </div>
            
            <ul class="todo-list">
                <For
                    each=filtered_todos
                    key=|todo| todo.id.clone()
                    children=move |todo| {
                        let todo_id_for_toggle = todo.id.clone();
                        let todo_id_for_delete = todo.id.clone();
                        
                        view! {
                            <li class=move || if todo.completed { "completed" } else { "" }>
                                <div class="todo-item">
                                    <input 
                                        type="checkbox" 
                                        prop:checked=todo.completed
                                        on:change=move |_| {
                                            toggle_todo(todo_id_for_toggle.clone())
                                        }
                                    />
                                    <span>{todo.text.clone()}</span>
                                    <button 
                                        class="delete"
                                        on:click=move |_| {
                                            delete_todo(todo_id_for_delete.clone())
                                        }
                                    >
                                        "×"
                                    </button>
                                </div>
                            </li>
                        }
                    }
                />
            </ul>
            
            <div class="todo-count">
                {move || {
                    let count = active_count();
                    format!("{} item{} left", count, if count == 1 { "" } else { "s" })
                }}
            </div>
        </div>
    }
}
"#;

    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    // Create main.rs
    let main_rs = format!(r#"use leptos::*;
use {}::App;

fn main() {{
    // Set up logging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    
    mount_to_body(|| view! {{ <App/> }})
}}
"#, app_name);

    std::fs::write(src_dir.join("main.rs"), main_rs)?;
    
    // Create index.html
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link data-trunk rel="rust" data-wasm-opt="z"/>
    <link data-trunk rel="css" href="style.css"/>
    <title>Leptos Todo App</title>
</head>
<body>
    <!-- This is where your Leptos app will be mounted -->
</body>
</html>
"#;
    std::fs::write(app_path.join("index.html"), index_html)?;
    
    // Create style.css
    let style_css = r#"html, body {
    margin: 0;
    padding: 0;
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    background-color: #f5f5f5;
    color: #4a4a4a;
}

.todo-app {
    max-width: 550px;
    margin: 2rem auto;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    padding: 2rem;
}

h1 {
    text-align: center;
    color: #2563eb;
    margin-top: 0;
}

p {
    color: #64748b;
    margin-bottom: 1.5rem;
}

.todo-form {
    display: flex;
    margin-bottom: 1.5rem;
}

.todo-form input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #e2e8f0;
    border-radius: 4px 0 0 4px;
    font-size: 1rem;
}

.todo-form button {
    padding: 0.75rem 1.5rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0 4px 4px 0;
    cursor: pointer;
    font-size: 1rem;
    transition: background-color 0.2s;
}

.todo-form button:hover {
    background-color: #2563eb;
}

.todo-controls {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #e2e8f0;
}

.filters {
    display: flex;
    gap: 0.5rem;
}

.filters button, .toggle-all, .clear-completed {
    background: none;
    border: 1px solid #e2e8f0;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    color: #64748b;
}

.filters button:hover, .toggle-all:hover, .clear-completed:hover {
    background-color: #f8fafc;
}

.filters button.active {
    border-color: #3b82f6;
    color: #3b82f6;
}

button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.todo-list {
    list-style: none;
    padding: 0;
    margin: 0;
}

.todo-item {
    display: flex;
    align-items: center;
    padding: 1rem 0;
    border-bottom: 1px solid #f1f5f9;
}

.todo-item input[type="checkbox"] {
    margin-right: 1rem;
    width: 1.25rem;
    height: 1.25rem;
}

.todo-item span {
    flex: 1;
}

li.completed span {
    text-decoration: line-through;
    color: #94a3b8;
}

.delete {
    background: none;
    border: none;
    color: #ef4444;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0 0.5rem;
}

.delete:hover {
    color: #dc2626;
}

.todo-count {
    margin-top: 1rem;
    text-align: left;
    color: #64748b;
    font-size: 0.875rem;
}
"#;
    std::fs::write(app_path.join("style.css"), style_css)?;
    
    // Create README.md
    let readme = format!(r#"# {} - Leptos Todo App

A feature-rich todo application built with [Leptos](https://github.com/leptos-rs/leptos), demonstrating reactive state management and component composition.

## Features

- Add, toggle, and delete todos
- Filter todos by status (All, Active, Completed)
- Clear completed todos
- Responsive design

## Prerequisites

- Rust and Cargo
- Trunk: `cargo install trunk --locked`
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

## Running the Application

```bash
cd {}
trunk serve --open
```

This will start a local development server and open the application in your default web browser.

## Building for Production

```bash
trunk build --release
```

This will create optimized WebAssembly files in the `dist` directory.

## Project Structure

- `src/lib.rs`: Contains the main application logic and components
- `src/main.rs`: Entry point that mounts the application to the DOM
- `index.html`: HTML template with Trunk directives
- `style.css`: Styling for the todo application

## Learn More

- [Leptos Documentation](https://leptos.dev/)
- [Reactive Primitives in Leptos](https://docs.rs/leptos/latest/leptos/primitives/index.html)
"#, app_name, app_name);
    
    std::fs::write(app_path.join("README.md"), readme)?;
    
    println!("✅ Successfully created a Leptos todo project");
    
    Ok(())
}

// Helper function to create a Leptos SSR (Server-Side Rendering) project
pub fn create_leptos_ssr_project(app_path: &Path) -> Result<()> {
    println!("📝 Creating a Leptos SSR (Server-Side Rendering) project...");
    
    // Prompt user to select server technology
    let server_options = vec!["Axum", "Actix Web"];
    let server_selection = Select::new()
        .with_prompt("Select server technology for your Leptos SSR application")
        .items(&server_options)
        .default(0)
        .interact()?;
    
    let server_tech = server_options[server_selection];
    println!("🚀 Using {} as the server technology", server_tech);
    
    let app_name = app_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("leptos_app");
    
    // Create src directory
    let src_dir = app_path.join("src");
    create_directory(&src_dir)?;
    
    // Create Cargo.toml with Leptos dependencies for SSR
    let cargo_toml = match server_tech {
        "Axum" => format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos = {{ version = "0.7.0" }}
leptos_router = {{ version = "0.7.0" }}
leptos_meta = {{ version = "0.7.0" }}
axum = {{ version = "0.7", optional = true }}
console_error_panic_hook = {{ version = "0.1", optional = true }}
leptos_axum = {{ version = "0.7.0", optional = true }}
tokio = {{ version = "1", features = ["rt-multi-thread"], optional = true }}
wasm-bindgen = {{ version = "=0.2.100", optional = true }}
serde = {{ version = "1.0", features = ["derive"] }}

[features]
hydrate = [
    "leptos/hydrate",
    "dep:console_error_panic_hook",
    "dep:wasm-bindgen",
]
ssr = [
    "dep:axum",
    "dep:tokio",
    "dep:leptos_axum",
    "leptos/ssr",
    "leptos_meta/ssr",
    "leptos_router/ssr",
]

# Defines a size-optimized profile for the WASM bundle in release mode
[profile.wasm-release]
inherits = "release"
opt-level = 'z'
lto = true
codegen-units = 1
panic = "abort"

[package.metadata.leptos]
# The name used by wasm-bindgen/cargo-leptos for the JS/WASM bundle. Defaults to the crate name
output-name = "{}"
# The site root folder is where cargo-leptos generate all output. WARNING: all content of this folder will be erased on a rebuild. Use it in your server setup.
site-root = "target/site"
# The site-root relative folder where all compiled output (JS, WASM and CSS) is written
# Defaults to pkg
site-pkg-dir = "pkg"
# [Optional] The source CSS file. If it ends with .sass or .scss then it will be compiled by dart-sass into CSS. The CSS is optimized by Lightning CSS before being written to <site-root>/<site-pkg>/app.css
style-file = "style/main.scss"
# Assets source dir. All files found here will be copied and synchronized to site-root.
# The assets-dir cannot have a sub directory with the same name/path as site-pkg-dir.
assets-dir = "public"
# The IP and port where the server serves the content. Use it in your server setup.
site-addr = "127.0.0.1:3000"
# The port to use for automatic reload monitoring
reload-port = 3001
# [Optional] Command to use when running end2end tests. It will run in the end2end dir.
end2end-cmd = "npx playwright test"
end2end-dir = "end2end"
#  The browserlist query used for optimizing the CSS.
browserquery = "defaults"
# Set by cargo-leptos watch when building with the option --release.
watch = false
# The environment Leptos will run in, usually either "DEV" or "PROD"
env = "DEV"
# The features to use when compiling the bin target
#
# Optional. Can be over-ridden with the command line parameter --bin-features
bin-features = ["ssr"]

# If the --no-default-features flag should be used when compiling the bin target
#
# Optional. Defaults to false.
bin-default-features = false

# The features to use when compiling the lib target
#
# Optional. Can be over-ridden with the command line parameter --lib-features
lib-features = ["hydrate"]

# If the --no-default-features flag should be used when compiling the lib target
#
# Optional. Defaults to false.
lib-default-features = false

# The profile to use for the lib target when compiling for release
#
# Optional. Defaults to "release".
lib-profile-release = "wasm-release"
"#, app_name, app_name),
        "Actix Web" => format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos = {{ version = "0.7.0" }}
leptos_router = {{ version = "0.7.0" }}
leptos_meta = {{ version = "0.7.0" }}
actix-web = {{ version = "4.4", optional = true }}
actix-files = {{ version = "0.6", optional = true }}
console_error_panic_hook = {{ version = "0.1", optional = true }}
leptos_actix = {{ version = "0.7.0", optional = true }}
tokio = {{ version = "1", features = ["rt-multi-thread"], optional = true }}
wasm-bindgen = {{ version = "=0.2.100", optional = true }}
serde = {{ version = "1.0", features = ["derive"] }}

[features]
hydrate = [
    "leptos/hydrate",
    "dep:console_error_panic_hook",
    "dep:wasm-bindgen",
]
ssr = [
    "dep:actix-web",
    "dep:actix-files",
    "dep:tokio",
    "dep:leptos_actix",
    "leptos/ssr",
    "leptos_meta/ssr",
    "leptos_router/ssr",
]

# Defines a size-optimized profile for the WASM bundle in release mode
[profile.wasm-release]
inherits = "release"
opt-level = 'z'
lto = true
codegen-units = 1
panic = "abort"

[package.metadata.leptos]
# The name used by wasm-bindgen/cargo-leptos for the JS/WASM bundle. Defaults to the crate name
output-name = "{}"
# The site root folder is where cargo-leptos generate all output. WARNING: all content of this folder will be erased on a rebuild. Use it in your server setup.
site-root = "target/site"
# The site-root relative folder where all compiled output (JS, WASM and CSS) is written
# Defaults to pkg
site-pkg-dir = "pkg"
# [Optional] The source CSS file. If it ends with .sass or .scss then it will be compiled by dart-sass into CSS. The CSS is optimized by Lightning CSS before being written to <site-root>/<site-pkg>/app.css
style-file = "style/main.scss"
# Assets source dir. All files found here will be copied and synchronized to site-root.
# The assets-dir cannot have a sub directory with the same name/path as site-pkg-dir.
assets-dir = "public"
# The IP and port where the server serves the content. Use it in your server setup.
site-addr = "127.0.0.1:3000"
# The port to use for automatic reload monitoring
reload-port = 3001
# [Optional] Command to use when running end2end tests. It will run in the end2end dir.
end2end-cmd = "npx playwright test"
end2end-dir = "end2end"
#  The browserlist query used for optimizing the CSS.
browserquery = "defaults"
# Set by cargo-leptos watch when building with the option --release.
watch = false
# The environment Leptos will run in, usually either "DEV" or "PROD"
env = "DEV"
# The features to use when compiling the bin target
#
# Optional. Can be over-ridden with the command line parameter --bin-features
bin-features = ["ssr"]

# If the --no-default-features flag should be used when compiling the bin target
#
# Optional. Defaults to false.
bin-default-features = false

# The features to use when compiling the lib target
#
# Optional. Can be over-ridden with the command line parameter --lib-features
lib-features = ["hydrate"]

# If the --no-default-features flag should be used when compiling the lib target
#
# Optional. Defaults to false.
lib-default-features = false

# The profile to use for the lib target when compiling for release
#
# Optional. Defaults to "release".
lib-profile-release = "wasm-release"
"#, app_name, app_name),
        _ => return Err(anyhow!("Unsupported server technology selected")),
    };
    
    std::fs::write(app_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create app.rs with Leptos components
    let app_rs = match server_tech {
        "Axum" => format!(r#"use leptos::prelude::*;
use leptos_meta::{{provide_meta_context, MetaTags, Stylesheet, Title}};
use leptos_router::{{
    components::{{Route, Router, Routes}},
    StaticSegment,
}};

pub fn shell(options: LeptosOptions) -> impl IntoView {{
    view! {{
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }}
}}

#[component]
pub fn App() -> impl IntoView {{
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {{
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/{}.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("about") view=AboutPage/>
                </Routes>
            </main>
        </Router>
    }}
}}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {{
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {{
        <div class="container">
            <h1>"Welcome to Leptos!"</h1>
            <button on:click=on_click>"Click Me: " {{count}}</button>
            <p>"This is a simple Leptos application with server-side rendering."</p>
            <div class="navigation">
                <a href="/about">"About"</a>
            </div>
        </div>
    }}
}}

/// Renders the about page.
#[component]
fn AboutPage() -> impl IntoView {{
    view! {{
        <div class="container">
            <h1>"About"</h1>
            <p>"This is a simple Leptos application with server-side rendering."</p>
            <p>"Here are some key features of Leptos:"</p>
            <ul>
                <li>"Fine-grained reactivity"</li>
                <li>"Server-side rendering"</li>
                <li>"Hydration"</li>
                <li>"Server functions"</li>
            </ul>
            
            <div class="navigation">
                <a href="/">"Back to Home"</a>
            </div>
        </div>
    }}
}}
"#, app_name),
        "Actix Web" => format!(r#"use leptos::prelude::*;
use leptos_meta::{{provide_meta_context, MetaTags, Stylesheet, Title}};
use leptos_router::{{
    components::{{Route, Router, Routes}},
    StaticSegment,
}};

pub fn shell(options: LeptosOptions) -> impl IntoView {{
    view! {{
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }}
}}

#[component]
pub fn App() -> impl IntoView {{
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {{
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/{}.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("about") view=AboutPage/>
                </Routes>
            </main>
        </Router>
    }}
}}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {{
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {{
        <div class="container">
            <h1>"Welcome to Leptos!"</h1>
            <button on:click=on_click>"Click Me: " {{count}}</button>
            <p>"This is a simple Leptos application with server-side rendering."</p>
            <div class="navigation">
                <a href="/about">"About"</a>
            </div>
        </div>
    }}
}}

/// Renders the about page.
#[component]
fn AboutPage() -> impl IntoView {{
    view! {{
        <div class="container">
            <h1>"About"</h1>
            <p>"This is a simple Leptos application with server-side rendering."</p>
            <p>"Here are some key features of Leptos:"</p>
            <ul>
                <li>"Fine-grained reactivity"</li>
                <li>"Server-side rendering"</li>
                <li>"Hydration"</li>
                <li>"Server functions"</li>
            </ul>
            
            <div class="navigation">
                <a href="/">"Back to Home"</a>
            </div>
        </div>
    }}
}}
"#, app_name),
        _ => return Err(anyhow!("Unsupported server technology selected")),
    };
    
    std::fs::write(src_dir.join("app.rs"), app_rs)?;
    
    // Create lib.rs with Leptos components
    let lib_rs = format!(r#"pub mod app;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {{
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}}
"#);

    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    // Create main.rs with server setup
    let main_rs = match server_tech {
        "Axum" => format!(r#"#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {{
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{{generate_route_list, LeptosRoutes}};
    use {}::app::*;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // build our app with a route
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {{
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        }})
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log!("listening on http://{{}}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}}

#[cfg(not(feature = "ssr"))]
pub fn main() {{
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}}
"#, app_name),
        "Actix Web" => format!(r#"#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {{
    use actix_files::Files;
    use actix_web::{{web, App, HttpServer}};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_actix::{{generate_route_list, LeptosRoutes}};
    use {}::app::*;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Set up HTTP server
    log!("listening on http://{{}}", &addr);
    HttpServer::new(move || {{
        let leptos_options = leptos_options.clone();
        let site_root = leptos_options.site_root.clone();
        let routes = routes.clone();

        App::new()
            .route("/api/{{tail:.*}}", leptos_actix::handle_server_fns())
            // serve JS/WASM/CSS from site_root
            .service(Files::new("/pkg", format!("{{}}/pkg", site_root)))
            .service(Files::new("/assets", format!("{{}}/assets", site_root)))
            // serve other assets from the site_root
            .leptos_routes(leptos_options.clone(), routes, {{
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            }})
            .app_data(web::Data::new(leptos_options))
    }})
    .bind(&addr)?
    .run()
    .await
}}

#[cfg(not(feature = "ssr"))]
pub fn main() {{
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}}
"#, app_name),
        _ => return Err(anyhow!("Unsupported server technology selected")),
    };
    
    std::fs::write(src_dir.join("main.rs"), main_rs)?;
    
    // Create assets directory
    let assets_dir = app_path.join("assets");
    std::fs::create_dir_all(&assets_dir)?;
    
    // Create style directory
    let style_dir = app_path.join("style");
    std::fs::create_dir_all(&style_dir)?;
    
    let main_scss = r#"html, body {
    font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    margin: 0;
    padding: 0;
    line-height: 1.6;
}

.container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
}

h1 {
    color: #333;
    margin-bottom: 1rem;
}

p {
    margin-bottom: 1rem;
}

button {
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-size: 1rem;
    transition: background-color 0.2s;
}

button:hover {
    background-color: #2563eb;
}

.navigation {
    margin-top: 2rem;
}

.navigation a {
    color: #3b82f6;
    text-decoration: none;
    margin-right: 1rem;
}

.navigation a:hover {
    text-decoration: underline;
}

.counter {
    margin: 2rem 0;
}

ul {
    margin-bottom: 1rem;
    padding-left: 1.5rem;
}

li {
    margin-bottom: 0.5rem;
}
"#;
    
    std::fs::write(style_dir.join("main.scss"), main_scss)?;
    
    // Create public directory instead of assets (to match Leptos 0.7 conventions)
    let public_dir = app_path.join("public");
    std::fs::create_dir_all(&public_dir)?;
    
    // Create a simple favicon.ico in the public directory
    let favicon_path = public_dir.join("favicon.ico");
    if !favicon_path.exists() {
        // Create an empty favicon.ico file
        std::fs::write(favicon_path, &[])?;
    }
    
    // Create Leptos config file
    let leptos_config = r#"{
  "site-root": "target/site",
  "site-pkg-dir": "pkg",
  "style-file": "style/main.scss",
  "assets-dir": "public",
  "site-addr": "127.0.0.1:3000",
  "reload-port": 3001,
  "browserquery": "defaults",
  "watch": false,
  "env": "DEV"
}"#;
    std::fs::write(app_path.join("Leptos.toml"), leptos_config)?;
    
    // Create .gitignore
    let gitignore = r#"# Generated files
/target/
/pkg/
/dist/

# Cargo lock file
Cargo.lock

# Editor files
.idea/
.vscode/
*.swp
*.swo

# OS files
.DS_Store
"#;
    
    std::fs::write(app_path.join(".gitignore"), gitignore)?;
    
    // Create README.md
    let readme = format!(r#"# {}

A Leptos SSR application using {} as the server technology.

## Getting Started

1. Install Rust and cargo-leptos:
   ```bash
   rustup update
   cargo install cargo-leptos --locked
   ```

2. Run the development server:
   ```bash
   cargo leptos watch
   ```

3. Open your browser and navigate to http://localhost:3000

## Building for Production

```bash
cargo leptos build --release
```

## Features

- Server-side rendering with {}
- Hydration for client-side interactivity
- Routing with leptos_router
- Styling with SCSS
"#, app_name, server_tech, server_tech);

    std::fs::write(app_path.join("README.md"), readme)?;
    
    println!("✅ Leptos SSR project with {} created successfully", server_tech);
    Ok(())
}

// Helper function to create a manual Dioxus project
pub fn create_manual_dioxus_project(app_path: &Path, app_name: &str) -> Result<()> {
    println!("⚠️ Creating manual Dioxus project structure");
    
    // Create src directory
    let src_dir = app_path.join("src");
    create_directory(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
dioxus = "0.4"
dioxus-web = "0.4"
log = "0.4"
"#, app_name);
    
    std::fs::write(app_path.join("Cargo.toml"), cargo_toml)?;
    
    // Create main.rs
    let main_rs = r#"use dioxus::prelude::*;

fn main() {
    // Init logger for debug info
    dioxus_web::launch(App);
}

fn App() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div { class: "container",
            h1 { "Welcome to Dioxus!" }
            p { "This is a simple counter example." }
            
            div { class: "counter",
                button { 
                    onclick: move |_| count -= 1,
                    "Decrement" 
                }
                
                span { class: "count", "{count}" }
                
                button { 
                    onclick: move |_| count += 1,
                    "Increment" 
                }
            }
            
            footer {
                p { "Made with Dioxus" }
            }
        }
    }
}
"#;
    
    std::fs::write(src_dir.join("main.rs"), main_rs)?;
    
    // Create index.html
    let index_html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Dioxus App</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            margin: 0;
            padding: 0;
            background-color: #f5f5f5;
            color: #333;
        }}
        
        .container {{
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            text-align: center;
        }}
        
        h1 {{
            color: #2563eb;
            margin-bottom: 1rem;
        }}
        
        p {{
            color: #64748b;
            margin-bottom: 2rem;
        }}
        
        .counter {{
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1rem;
            margin: 2rem 0;
        }}
        
        button {{
            background-color: #3b82f6;
            color: white;
            border: none;
            border-radius: 4px;
            padding: 0.5rem 1rem;
            cursor: pointer;
            font-size: 1rem;
            transition: background-color 0.2s;
        }}
        
        button:hover {{
            background-color: #2563eb;
        }}
        
        .count {{
            font-size: 2rem;
            font-weight: bold;
            color: #2563eb;
            min-width: 4rem;
        }}
        
        footer {{
            margin-top: 4rem;
            color: #94a3b8;
            font-size: 0.875rem;
        }}
    </style>
</head>
<body>
    <div id="main"></div>
</body>
</html>
"#, app_name);

    std::fs::write(app_path.join("index.html"), index_html)?;
    
    // Create Dioxus.toml configuration file
    let dioxus_toml = r#"[application]
name = "dioxus-app"
default_platform = "web"
out_dir = "dist"
asset_dir = "public"

[web.app]
title = "Dioxus App"

[web.watcher]
reload_html = true
watch_path = ["src", "public"]

[web.resource]
style = []
script = []

[web.resource.dev]
"#;

    std::fs::write(app_path.join("Dioxus.toml"), dioxus_toml)?;
    
    // Create README.md
    let readme = format!(r#"# {} - Dioxus App

A simple web application built with [Dioxus](https://dioxuslabs.com/).

## Features

- Simple counter example
- Clean, responsive UI
- Hot reloading during development

## Prerequisites

- Rust and Cargo
- Dioxus CLI: `cargo install dioxus-cli --locked`
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

## Development

```bash
# Navigate to the project directory
cd {}

# Start the development server
dx serve --hot-reload
```

This will start a local development server and open your application in a browser.

## Building for Production

```bash
dx build --release
```

This will create optimized WebAssembly files in the `dist` directory.

## Project Structure

- `src/main.rs`: Contains the main application code and components
- `index.html`: HTML template with Trunk directives
- `Dioxus.toml`: Configuration file for the Dioxus CLI

## Learn More

- [Dioxus Documentation](https://dioxuslabs.com/docs/0.4/guide/en/)
- [Dioxus GitHub Repository](https://github.com/DioxusLabs/dioxus)
"#, app_name, app_name);
    
    std::fs::write(app_path.join("README.md"), readme)?;
    
    // Create .gitignore
    let gitignore = r#"# Generated files
/target/
/dist/

# Cargo lock file
Cargo.lock

# Editor files
.idea/
.vscode/
*.swp
*.swo

# OS files
.DS_Store
"#;
    
    std::fs::write(app_path.join(".gitignore"), gitignore)?;
    
    println!("✅ Successfully created manual Dioxus project structure");
    
    Ok(())
}
