![FerrisUp - A powerful Rust project bootstrapping tool](https://raw.githubusercontent.com/Jitpomi/ferrisup/main/img.png)

# FerrisUp CLI

A powerful Rust project bootstrapping tool - Start Anywhere, Scale Anywhere

> **Note:** FerrisUp is under active development. While the core templates are fully functional, some advanced features are still in development.

## Overview

FerrisUp CLI is a versatile command-line tool for creating and managing Rust projects with flexible templates. Like Create React App for React, FerrisUp makes it easy to start new Rust projects with the right structure and dependencies.

## Features

- **Multiple Templates**: From minimal binaries to full-stack applications
- **Transform Capability**: Start with a simple project and scale as you grow
- **Data Science Support**: Built-in templates for Polars and Linfa
- **Edge Computing**: WebAssembly and serverless-ready templates
- **Embedded Systems**: Support for RP2040, ESP32, STM32, and Arduino
- **Interactive CLI**: User-friendly command interface

## Installation

```bash
# Install from crates.io
cargo install ferrisup

# Or install from source
git clone https://github.com/Jitpomi/ferrisup.git
cd ferrisup
cargo install --path .
```

## Quick Start

```bash
# Create a new project
ferrisup new my_project

# Create a new minimal project
ferrisup new my_project --component-type minimal

# Create a  leptos client project
ferrisup new my_client --component-type client --framework leptos

# Create a data science project
ferrisup new my_data_app --component-type data-science --framework polars

# Create a server with Axum
ferrisup new my_server --component-type server --framework axum

# Create an edge computing project for Cloudflare
ferrisup new my_edge_app --component-type edge --provider cloudflare
```

## Available Templates

View all available templates:

```bash
ferrisup list
```

Current templates include:
- `minimal` - Simple binary with a single main.rs file
- `library` - Rust library crate with a lib.rs file
- `embedded` - Embedded systems firmware for microcontrollers
- `server` - Web server with API endpoints (Axum, Actix, or Poem)
- `client` - Frontend web application (Leptos, Yew, or Dioxus)
- `serverless` - Serverless function (AWS Lambda, Cloudflare Workers, etc.)
- `data-science` - Data science and machine learning projects
- `edge` - Edge computing applications (Cloudflare, Vercel, Fastly, AWS, etc.)

## Commands

### `new`

Create a new Rust project with a predefined structure.

```bash
ferrisup new [PROJECT_NAME] [--template TEMPLATE_NAME] [--git] [--build] [--no-interactive]

# Component-specific options (use one of these combinations):
ferrisup new [PROJECT_NAME] [--component-type TYPE] [--framework FRAMEWORK]
ferrisup new [PROJECT_NAME] [--component-type TYPE] [--provider PROVIDER]
ferrisup new [PROJECT_NAME] [--component-type TYPE] [--application-type APPLICATION_TYPE]
```

- `PROJECT_NAME`: Optional name for your project
- `--template`: Specify a template (web, api, full-stack, etc.)
- `--component-type`: Specify a component type (server, client, data-science, edge, etc.)
- `--framework`: Specify a framework for the selected component type (e.g., polars, linfa for data-science; axum, actix, poem for server)
- `--provider`: Specify a cloud provider for serverless or edge components (e.g., cloudflare, vercel, aws)
- `--application-type`: Specify an application type for certain components
- `--git`: Initialize a git repository
- `--build`: Run cargo build after creation
- `--no-interactive`: Create project without prompting, using default values

### `transform`

Transform an existing project into a different structure or add components. This command enables the "Start Anywhere, Scale Anywhere" philosophy by allowing you to evolve your project structure as your needs grow.

```bash
ferrisup transform [--project PATH]
```

The transform command provides an interactive menu with the following capabilities:

- **Convert to Workspace**: Transform a single-crate project into a Rust workspace
  - Intelligently detects the component type of your existing project
  - Uses the component type as the default name suggestion
  - Properly moves all project files to the first component
  - Updates package names to use project-prefixed format (e.g., `projectname_componentname`)
  - Automatically updates source file references to match the new package names

- **Add Components**: Add new components to an existing workspace
  - Supports all component types (client, server, shared, edge, etc.)
  - Uses the same framework options as the `new` command
  - Creates components with proper package naming conventions
  - Directly uses the `new` command functionality for consistent component creation
  - Updates the workspace Cargo.toml automatically

- **Add Components Without Workspace**: Add related components without converting to a workspace
  - Creates sibling component projects in the same parent directory
  - Maintains the same component selection experience as the workspace version
  - Preserves the original project structure

- **Update Metadata**: Manage project configuration stored in `.ferrisup/metadata.toml`

- **Next Steps Guide**: Provides clear instructions after transformation
  - Shows commands to build all components at once
  - Explains how to run specific components
  - Offers guidance on adding dependencies

### `list`

List available templates.

```bash
ferrisup list
```

### `preview`

Preview a template without creating any files.

```bash
ferrisup preview [--template TEMPLATE_NAME]
```

### `unused-features`

Find and remove unused features in your Cargo dependencies to optimize your project.

```bash
ferrisup unused-features [--path PATH]
```

The command will:
1. Check if the `unused-features` tool is installed (and install it if needed)
2. Analyze your project for unused features in dependencies
3. Display a list of unused features grouped by dependency
4. Provide specific recommendations for removing them from your Cargo.toml

## Component Types and Frameworks

FerrisUp supports various component types, each with specialized frameworks or providers:

### Server Components
- **Frameworks**: axum, actix, poem
- **Example**: `ferrisup new my_server --component-type server --framework axum`

### Client Components
- **Frameworks**: leptos, dioxus, tauri
- **Example**: `ferrisup new my_client --component-type client --framework leptos`

### Data Science Projects
- **Frameworks**: 
  - **polars**: Data analysis framework similar to pandas in Python
  - **linfa**: Machine learning framework for classification, regression, and clustering
- **Example**: `ferrisup new my_data_app --component-type data-science --framework polars`

### Edge Components
- **Providers**: cloudflare, vercel, aws
- **Example**: `ferrisup new my_edge_app --component-type edge --provider cloudflare`

## Development and Testing

FerrisUp includes a comprehensive test suite to ensure functionality and compatibility. You can run the tests with:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests in specific file
cargo test --test file_name
```

The `--no-interactive` flag is especially useful for automated testing and CI/CD pipelines, allowing for the creation of projects without requiring user input.

## Contributing

We welcome and encourage contributions from the community! If you believe in the project and would like to help make FerrisUp even better, please consider contributing.

**How to Contribute:**

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add some amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

**Priority Areas for Contribution:**

- Enhancing the `transform` command to fully support project transformation between templates
- Implementing comprehensive `scale` functionality for enterprise deployments
- Expanding the `component` and `workspace` management features
- Improving the dependency management system
- Adding new templates or enhancing existing ones

For major contributions or if you'd like to discuss implementation details before starting work, please email us at dev@jitpomi.com.

## Project Status

FerrisUp is currently in active development. Here's the current status of various features:

### Fully Implemented Features
- Core templates (minimal, library, server, data-science, edge, embedded, serverless)
- Basic project creation with the `new` command
- Template listing with the `list` command
- Template preview with the `preview` command
- Transform command for converting projects to workspaces and adding components
- Unused features detection

### Features In Development
- Scale command for deployment
- Database components
- Machine learning components

## License

This project is licensed under the MIT License - see the LICENSE file for details.
