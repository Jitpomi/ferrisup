# Contributing New Project Types to FerrisUp

This guide explains how to add new project types to FerrisUp following our architecture that properly separates CLI tools and templates.

## Overview

FerrisUp supports two main types of project creation:

1. **CLI-based projects** - Projects that use external CLI tools like Embassy, Dioxus, Tauri, etc.
2. **Template-based projects** - Projects that use FerrisUp's internal template system

Our architecture ensures these approaches don't interfere with each other, making it easy to add new project types without breaking existing ones.

## Quick Start Guide

### Adding a CLI-based Project

1. Identify a Rust CLI tool that creates projects (e.g., `cargo-leptos`, `cargo-tauri`)
2. Create a CLI handler in `src/project/handlers/mod.rs` following the pattern:

```rust
// Example: Adding Leptos CLI support
handlers.push(Box::new(CliProjectHandler::new(
    "Leptos",  // Name
    "Full-stack Rust framework for building web applications",  // Description
    vec!["client-leptos".to_string(), "leptos".to_string()],  // Template names to handle
    "cargo leptos",  // Base CLI command
    |project_name, _target_dir, variables| {
        // Function to generate CLI args based on FerrisUp variables
        let mut args = vec!["new".to_string(), project_name.to_string()];
        
        // Add any additional CLI args based on variables
        if let Some(mode) = variables.get("mode").and_then(|m| m.as_str()) {
            args.push("--mode".to_string());
            args.push(mode.to_string());
        }
        
        args
    },
    |project_name, _variables| {
        // Function to generate next steps for the user
        vec![
            format!("🚀 Navigate to your project: cd {}", project_name),
            "🔧 Build the project: cargo leptos watch".to_string(),
            "🌐 View your app at http://localhost:3000".to_string(),
        ]
    },
    Some("cargo install cargo-leptos".to_string()),  // Installation command
    Some("cargo leptos --version".to_string()),  // Version check command
)));
```

3. Test your handler with: `ferrisup new my-project --template leptos`

### Adding a Template-based Project

1. Create a template directory in `templates/your-template-name/`
2. Add a `template.json` file with:
   - `name` - Template name
   - `description` - Template description
   - `prompts` - User input prompts
   - `variables` - Derived variables
   - `files` - Files to include (with optional conditions)
   - `next_steps` - Guidance for users after creation

3. Register your template in `src/project/handlers/mod.rs`:

```rust
// Example: Adding a new game development template
handlers.push(Box::new(TemplateProjectHandler::new(
    "Game Development",
    "Rust game development with various engines",
    vec!["game".to_string(), "bevy".to_string(), "macroquad".to_string()]
)));
```

4. Test your template with: `ferrisup new my-game --template game`

## Detailed Architecture Guide

### ProjectHandler Interface

All project handlers implement the `ProjectHandler` trait:

```rust
pub trait ProjectHandler {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn can_handle(&self, template_name: &str, variables: &Value) -> bool;
    fn initialize_project(&self, project_name: &str, target_dir: &Path, variables: &Value) -> Result<()>;
    fn get_next_steps(&self, project_name: &str, variables: &Value) -> Vec<String>;
}
```

### CLI Project Handler

The `CliProjectHandler` uses external CLI tools to create projects:

- **Template mapping** - Associates FerrisUp template names with CLI tools
- **CLI detection** - Checks if the CLI tool is installed
- **CLI installation** - Installs the CLI tool if needed
- **Variable mapping** - Maps FerrisUp variables to CLI arguments
- **Next steps** - Provides tailored guidance for CLI-generated projects

### Template Project Handler

The `TemplateProjectHandler` uses FerrisUp's template system:

- **Template application** - Uses the template_manager to apply templates
- **Variable substitution** - Processes template variables
- **Conditional files** - Includes files based on user selections
- **Next steps** - Retrieves guidance from template.json

## Best Practices

1. **Template names** - Use descriptive names that match the project type
2. **CLI integration** - Map FerrisUp variables to CLI arguments consistently
3. **Next steps** - Provide clear, helpful guidance for users
4. **Testing** - Verify your handler works with all options and variables
5. **Documentation** - Update this guide with any new patterns or examples

## Example: Adding Cargo Generate Support

```rust
// Example: Adding support for cargo-generate templates
handlers.push(Box::new(CliProjectHandler::new(
    "Cargo Generate",
    "Create a new Rust project from a template",
    vec!["cargo-generate".to_string(), "generate".to_string()],
    "cargo generate",
    |project_name, _target_dir, variables| {
        let mut args = vec!["--name".to_string(), project_name.to_string()];
        
        if let Some(template) = variables.get("git_template").and_then(|t| t.as_str()) {
            args.push("--git".to_string());
            args.push(template.to_string());
        }
        
        if let Some(branch) = variables.get("branch").and_then(|b| b.as_str()) {
            args.push("--branch".to_string());
            args.push(branch.to_string());
        }
        
        args
    },
    |project_name, variables| {
        vec![
            format!("🚀 Navigate to your project: cd {}", project_name),
            "📝 Review the generated code".to_string(),
            "🔧 Build the project: cargo build".to_string(),
            "▶️ Run the project: cargo run".to_string(),
        ]
    },
    Some("cargo install cargo-generate".to_string()),
    Some("cargo generate --version".to_string()),
)));
```

By following this guide, you can easily add new project types to FerrisUp while maintaining proper separation of concerns between CLI tools and templates.
