![FerrisUp - A powerful Rust project bootstrapping tool](https://raw.githubusercontent.com/Jitpomi/ferrisup/main/ferrisup/img.png)

# FerrisUp CLI

A powerful Rust project bootstrapping tool - Start Anywhere, Scale Anywhere

> **Note:** FerrisUp is under active development. While the core templates are fully functional, some advanced features are still in development.

## Overview

FerrisUp CLI is a versatile command-line tool for creating and managing Rust projects with flexible templates. Like Create React App for React, FerrisUp makes it easy to start new Rust projects with the right structure and dependencies.


## Features

- **Project Creation** (`new`): Create new Rust projects with interactive configuration
- **Project Transformation** (`transform`): Convert existing projects with interactive configuration
- **Component Insights** (`list`, `preview`): List and preview available components
- **Component Management** (`component`): Add and remove project components
- **Configuration Management** (`config`): Export and import configurations
- **Workspace Management** (`workspace`): Manage Cargo workspaces
- **Dependency Management** (`dependency`, `unused-features`): Manage dependencies and optimize feature usage
- **Cross-Platform**: Works on Unix-based systems and Windows

## Project Structure

FerrisUp is organized as a Cargo workspace with the following components:

- **ferrisup**: The main CLI application
- **ferrisup_common**: Shared utilities and functions used across the project
- **client**: Web interface for FerrisUp (optional)

## Commands

```
Commands:
  new              Create a new Rust project with interactive configuration
  transform        Transform an existing project with interactive configuration
  list             List available component types
  preview          Preview a component  without actually creating files
  component        Manage project components (add/remove/list) with consistent component types
  config           Manage configurations (export/import)
  workspace        Manage Cargo workspaces
  dependency       Manage project dependencies
  unused-features  Find unused features in Cargo dependencies
```

## Installation

```bash
# Install from crates.io
cargo install ferrisup

# Or install from source
git clone https://github.com/Jitpomi/ferrisup.git
cd ferrisup
cargo install --path ferrisup
```

## Usage

FerrisUp provides a simple and intuitive command-line interface for creating and managing Rust projects. Here's how to use it:

```bash
# Get help information
ferrisup --help

# Create a new project (interactive mode)
ferrisup new

# List available component types
ferrisup list

# Preview a component type (work in progress)
ferrisup preview --component-type server --framework axum

# Preview a client component with specific framework
ferrisup preview --component-type client --framework dioxus

# Transform an existing project
ferrisup transform

# Manage dependencies
ferrisup dependency add tokio serde
```

See the Commands section below for more detailed usage instructions.

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

## Available Component Types

View all available component types:

```bash
ferrisup list
```

Current component types include:
- `minimal` - Simple binary with a single main.rs file
- `binary` - Command-line application with CLI features
- `library` - Rust library crate with a lib.rs file
- `embedded` - Embedded systems firmware for microcontrollers
- `server` - Web server with API endpoints (Axum, Actix, or Poem)
- `client` - Frontend web application (Leptos, Yew, or Dioxus)
- `serverless` - Serverless function (AWS Lambda, Cloudflare Workers, etc.)
- `data-science` - Data science and machine learning projects
- `edge` - Edge computing applications (Cloudflare, Vercel, Fastly, AWS, etc.)
- `shared` - Shared code libraries for workspace components

## Commands

### `new`

Create a new Rust project with a predefined structure.

```bash
ferrisup new [PROJECT_NAME] [--git] [--build]

# Component-specific options (use one of these combinations):
ferrisup new [PROJECT_NAME] [--component-type TYPE] [--framework FRAMEWORK]
ferrisup new [PROJECT_NAME] [--component-type TYPE] [--provider PROVIDER]
ferrisup new [PROJECT_NAME] [--component-type TYPE] [--application-type APPLICATION_TYPE]
```

- `PROJECT_NAME`: Optional name for your project
- `--component-type`: Specify a component type (server, client, data-science, edge, binary, etc.)
- `--framework`: Specify a framework for the selected component type (e.g., polars, linfa for data-science; axum, actix, poem for server)
- `--provider`: Specify a cloud provider for serverless or edge components (e.g., cloudflare, vercel, aws)
- `--application-type`: Specify an application type for certain components
- `--git`: Initialize a git repository
- `--build`: Run cargo build after creation

### `preview` (Work in Progress)

Preview a component type without actually creating files. This command shows what files and features would be included in a project of the specified component type.

```bash
ferrisup preview [--component-type TYPE] [--framework FRAMEWORK] [--provider PROVIDER] [--application-type APPLICATION_TYPE]
```

- `--component-type`: Specify a component type to preview (server, client, data-science, edge, binary, etc.)
- `--framework`: Specify a framework for client, server, or embedded components
- `--provider`: Specify a cloud provider for serverless or edge components
- `--application-type`: Specify an application type for edge components

**Note: The preview command is currently a work in progress with several known limitations:**

- Template variable replacement is incomplete and may not accurately represent final output
- Framework-specific features and files may not be fully represented
- Some complex template combinations may not preview correctly
- File content previews are simplified and may differ from actual generated files
- Not all component options are fully supported

Future improvements will address these limitations to provide a more accurate preview experience.

### `transform`

Transform an existing project into a different structure or add components. This command enables the "Start Anywhere, Scale Anywhere" philosophy by allowing you to evolve your project structure as your needs grow.

```bash
ferrisup transform [--project PATH] [--test-mode]
```

The transform command provides an interactive menu with the following capabilities:

- **Convert to Workspace**: Transform a single-crate project into a Rust workspace
  - Intelligently detects the component type of your existing project (including binary/CLI projects)
  - Uses the project name as the default name suggestion (preserves the project's identity for publishing and deployments)
  - Implements strict safeguards to prevent critical source and build files from remaining at root
  - Smart file selection prompts that only allow build artifacts and temporary files to remain at root
  - Automatic backup of existing root-level files (README.md, .gitignore) before creating workspace versions
  - Creates comprehensive workspace README.md with project structure documentation
  - Properly moves all project files to the component directory
  - Updates package names to use project-prefixed format (e.g., `projectname_componentname`)
  - Automatically updates import paths to maintain project integrity after transformation
  - Provides clear color-coded terminal messaging throughout the transformation process
  - Supports non-interactive test mode via the `--test-mode` flag or `FERRISUP_TEST_MODE` environment variable

- **Add Components**: Add new components to an existing workspace
  - Supports all component types (client, server, shared, edge, data-science, binary, etc.)
  - Uses the component type as the default name suggestion when adding new components
  - Uses the same framework options as the `new` command
  - Creates components with proper package naming conventions
  - When adding a shared component, makes it accessible to all other workspace members via a module named `shared::*`
  - Directly uses the `new` command functionality for consistent component creation
  - Updates the workspace Cargo.toml automatically
  - Preserves and updates component metadata during transformation

- **Add Components Without Workspace**: Add related components without converting to a workspace
  - Creates sibling component projects in the same parent directory
  - Maintains the same component selection experience as the workspace version
  - Preserves the original project structure

- **Update Metadata**: Manage project configuration stored in `.ferrisup/metadata.toml`
  - More accurate component type detection based on project structure and dependencies
  - Ensures metadata consistency between .ferrisup/metadata.toml and component Cargo.toml
  - Properly identifies and sets "binary" component type for CLI applications

- **Next Steps Guide**: Provides clear instructions after transformation
  - Shows commands to build all components at once
  - Explains how to run specific components
  - Offers guidance on adding dependencies

### `list`

List available component types and frameworks.

```bash
ferrisup list
```

### `preview`

Preview a component without creating any files.

```bash
ferrisup preview [--component-type TYPE] [--framework FRAMEWORK]
```

### `dependency`

Manage dependencies in your Rust project with interactive features.

```bash
# Add dependencies with interactive feature suggestions
ferrisup dependency add [DEPENDENCIES...] [OPTIONS]

# Remove dependencies
ferrisup dependency remove [DEPENDENCIES...] [OPTIONS]

# Update dependencies
ferrisup dependency update [DEPENDENCIES...] [OPTIONS]
```

Options for `dependency add`:
- `--version, -v`: Specify a version constraint (defaults to "*")
- `--features, -f`: Specify features to enable (comma separated)
- `--dev, -d`: Add as a development dependency
- `--path, -p`: Specify project path (defaults to current directory)

**Smart Dependency Management:**
- When adding a dependency with `--dev` that already exists in main dependencies, it will be automatically moved to dev-dependencies
- When adding a dependency without `--dev` that already exists in dev-dependencies, it will be automatically moved to main dependencies
- Clear output messages inform you when dependencies are being moved between sections

### `component`

Manage project components (add/remove/list) with the same component types available in the `new` and `transform` commands.

```bash
# Add a component to your project (interactive)
ferrisup component --action add

# Add a specific component type
ferrisup component --action add --component-type client

# List components in a project
ferrisup component --action list

# Remove a component (interactive)
ferrisup component --action remove
```

Options:
- `--action, -a`: Specify action to perform (add, remove, list)
- `--component-type, -c`: Specify component type (client, server, shared, edge, data-science, embedded)
- `--project, -p`: Specify project path (defaults to current directory)

The component command uses the same component creation logic as the transform command, ensuring consistency across FerrisUp. When adding components, it provides the same interactive menus and framework selection options as the `new` and `transform` commands.

**Workspace vs. Non-Workspace Projects:**
- In workspace projects, all component types are available: client, server, shared, edge, data-science, and embedded
- In non-workspace projects, only module-compatible components are available: shared and minimal
- The command automatically detects your project structure and shows the appropriate options

The `add` command provides interactive feature suggestions for common crates:
- When adding popular crates like `tokio`, `serde`, or `reqwest`, FerrisUp will suggest commonly used features
- You can select features interactively or specify them directly with the `--features` flag
- This helps you choose the right features without needing to know all options in advance

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

For automated testing and CI/CD pipelines, the `transform` command supports a `--test-mode` flag (or the `FERRISUP_TEST_MODE` environment variable) that allows for non-interactive execution of workspace transformations.

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
- Core component types (minimal, library, server, data-science, edge, embedded, serverless)
- Basic project creation with the `new` command
- Component type listing with the `list` command
- Component preview with the `preview` command
- Transform command for converting projects to workspaces and adding components
- Unused features detection

### Features In Development

The following features are prioritized for the next releases as we work toward version 1.0:

#### Core Functionality Improvements
- **Configuration System Integration**: Properly integrate the existing config command with the rest of the application
- **Component Command Refinement**: Further align component command behavior with transform and new commands
- **CLI Help Text Enhancements**: Improve help text clarity and examples throughout the application
- **Error Handling Improvements**: More user-friendly error messages and recovery suggestions

#### Developer Experience
- **Interactive Component Preview**: Preview components before project creation
- **Project Scaffolding Wizard**: Step-by-step guided project creation for beginners
- **Component Customization**: Allow users to customize components during project creation
- **Dependency Version Management**: Smart handling of dependency version conflicts

#### Stability and Testing
- **Integration Test Suite**: Comprehensive test coverage for all commands
- **Cross-Platform Validation**: Ensure consistent behavior across operating systems
- **Component Validation**: Automated testing of all component combinations
- **Performance Optimization**: Reduce startup and execution time

## Future Work

The following features and improvements are planned for future releases of FerrisUp:

### Configuration System Integration

The existing configuration system (`ferrisup config`) has the foundation for powerful user preference management but needs further integration:

- **Persistent User Preferences**: Allow users to set default values for common flags (git initialization, build after creation, interactive mode)
- **Template Directory Management**: Streamline custom template usage with configurable template directories
- **Project Defaults**: Set default component types, frameworks, and other project settings
- **Profile Support**: Create and switch between different configuration profiles for different types of projects

### Scaling and Deployment Enhancements

- **Enhanced Cloud Integration**: Streamlined deployment workflows for existing cloud providers
- **Advanced Container Orchestration**: Extended Kubernetes support with service mesh integration
- **Multi-Cloud Management**: Tools for managing deployments across multiple cloud providers
- **Serverless Enhancement**: Advanced triggers, event processing, and cold start optimization
- **Infrastructure as Code**: Terraform and Pulumi integration for infrastructure provisioning
- **Multi-Region Deployment**: Tools for deploying applications across multiple regions with replication

### Database Components and Integration

- **Advanced Schema Management**: Enhanced database migration tools with versioning and rollback
- **Cross-Database Compatibility**: Tools for working with multiple database types in the same project
- **Database Performance Optimization**: Query optimization and indexing recommendations
- **Advanced Data Modeling**: Smart entity relationship mapping and code generation
- **Database Replication**: Support for master-slave configurations and read replicas
- **Time-Series Database Support**: Integration with specialized time-series databases
- **Graph Database Templates**: Support for graph databases like Neo4j and Amazon Neptune

### Embedded Systems and IoT Enhancements

- **Ariel OS Integration**: Collaboration with Ariel OS for batteries-included Embassy support
- **OTA Update Infrastructure**: Support for over-the-air updates for embedded devices
- **Embedded Debugging**: Integrated probe support for hardware debugging
- **Cross-Platform Embedded UI**: Dioxus integration for embedded UIs
- **Real-Time Operating System Templates**: Enhanced RTOS support for time-critical applications

### Network Automation and Mesh Networking

- **WireGuard Mesh Networking**: Multi-region mesh network configuration and deployment
- **Network Automation Tools**: Integration with network automation frameworks
- **Service Mesh Configuration**: Automatic service mesh setup for distributed applications
- **P2P Communication**: Peer-to-peer communication infrastructure for distributed applications
- **Network Monitoring**: Built-in monitoring and observability for network components

### Bare Metal and Self-Hosted Deployment

- **PXE/iPXE Boot Support**: Bare metal deployment via network boot
- **Colocation Server Deployment**: Streamlined deployment to self-hosted infrastructure
- **Unified Deployment Pipeline**: Single pipeline for cloud, bare metal, and hybrid deployments
- **Hardware-Optimized Configurations**: Performance tuning based on specific hardware
- **Cross-Environment Compatibility**: Deploy the same application across different infrastructure types

### CI/CD and Deployment Automation

- **One-Click Deployment**: Simplified deployment of Dioxus/Axum backend and database stacks
- **GitHub Actions Integration**: Built-in workflow templates for common CI/CD scenarios
- **Database Schema Migration**: Automated schema migration handling during deployments
- **Environment Management**: Development, staging, and production environment configuration
- **Deployment Rollbacks**: Automated rollback mechanisms for failed deployments
- **Deployment Metrics**: Performance and resource utilization tracking for deployments

### Additional Planned Features

- **Improved Template Management**: Version control and community template sharing
- **More Component Types and Frameworks**: Support for emerging Rust frameworks
- **Enhanced Dependency Management**: Smart dependency resolution and conflict detection
- **Project Analysis and Optimization Tools**: Performance analysis and optimization suggestions
- **Testing Framework Integration**: Built-in testing templates and test generation
- **Documentation Generation**: Automatic documentation generation for projects
- **Internationalization Support**: Tools for building multi-language applications

## License

This project is licensed under the MIT License - see the LICENSE file for details.
