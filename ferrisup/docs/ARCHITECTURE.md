# FerrisUp Architecture

This document explains the architectural design of FerrisUp, focusing on how it handles different project creation methods while maintaining proper separation of concerns.

## Core Components

FerrisUp's architecture is built around the following key components:

### 1. Project Handlers System

At the heart of FerrisUp is the ProjectHandler trait, which provides a unified interface for all project creation methods:

```rust
pub trait ProjectHandler {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn can_handle(&self, template_name: &str, variables: &Value) -> bool;
    fn initialize_project(&self, project_name: &str, target_dir: &Path, variables: &Value) -> Result<()>;
    fn get_next_steps(&self, project_name: &str, variables: &Value) -> Vec<String>;
}
```

This trait ensures consistent behavior regardless of how a project is created.

### 2. CLI Project Handlers

For projects that should be created using external CLI tools (like Embassy, Dioxus, Tauri), FerrisUp delegates to the appropriate CLI tool:

```
┌───────────────┐     ┌──────────────────┐     ┌──────────────────┐
│ FerrisUp CLI  │────▶│ CLI Handler      │────▶│ External CLI Tool│
└───────────────┘     │ - Detection      │     │ (cargo-embassy,  │
                      │ - Installation    │     │  dioxus-cli,     │
                      │ - Arg mapping     │     │  cargo-tauri)    │
                      └──────────────────┘     └──────────────────┘
```

Benefits:
- Leverages specialized tooling built for specific ecosystems
- Ensures compatibility with native workflows
- Reduces duplication of features

### 3. Template Project Handlers

For projects created from FerrisUp templates, the system uses its template engine:

```
┌───────────────┐     ┌──────────────────┐     ┌──────────────────┐
│ FerrisUp CLI  │────▶│ Template Handler │────▶│ Template Manager │
└───────────────┘     │ - Template match │     │ - File copying   │
                      │ - Variable setup │     │ - Substitution   │
                      │ - Next steps     │     │ - Post-processing│
                      └──────────────────┘     └──────────────────┘
```

Benefits:
- Customizable templates with variable substitution
- Conditional file inclusion based on user choices
- Standardized project structure

### 4. Command Flow

The main command flow in FerrisUp:

1. Parse command line arguments (template, project name, etc.)
2. Collect template variables through prompts (if interactive)
3. Find the appropriate handler for the template
4. Initialize the project using the handler
5. Apply common post-processing (git init, cargo build)
6. Display next steps to the user

## Why This Architecture?

This architecture was specifically designed to address several challenges:

1. **Independent Evolution**: CLI tools and templates can evolve independently without breaking each other
2. **Easy Extension**: New project types can be added without modifying core code
3. **Clear Responsibilities**: Each component has a single, well-defined responsibility
4. **Consistent Experience**: Users get a consistent experience regardless of project type

## Extending the System

The system is designed to be easily extended:

### Adding a New CLI Tool

1. Create a CLI handler in `src/project/handlers/mod.rs`
2. Define parameter mappings from FerrisUp variables to CLI arguments
3. Provide installation and version check commands
4. Define appropriate next steps

### Adding a New Template

1. Create a template in `templates/your-template/`
2. Define prompts, variables, and file conditions in `template.json`
3. Register the template with a TemplateProjectHandler in `src/project/handlers/mod.rs`

## Implementation Details

### Registry of Handlers

All handlers are registered in the `get_handlers()` function, making it easy to find and modify available project types.

### Handler Selection

When a user selects a template, FerrisUp finds the appropriate handler by:

1. Checking if any handler's `can_handle()` method returns true for the template
2. Using the first matching handler to initialize the project

### Variable Collection

Variables are collected from:
1. Command line arguments
2. Interactive prompts (if not in no-interactive mode)
3. Derived values in the template configuration

These variables are passed to the handler for use during project initialization.

## Future Enhancements

The architecture supports future enhancements such as:

1. Plugin system for third-party handlers
2. Configuration file for handlers
3. Support for additional project transformation capabilities
4. Integration with more external tools

## Conclusion

This architecture ensures that FerrisUp can "Start Anywhere, Scale Anywhere" by providing a clean separation between different project creation approaches while maintaining a consistent user experience.
