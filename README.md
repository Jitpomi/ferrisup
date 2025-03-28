# FerrisUp CLI

A powerful Rust project bootstrapping tool - Start Anywhere, Scale Anywhere

## Overview

FerrisUp CLI is a versatile command-line tool for creating and managing Rust projects with flexible templates. Like Create React App for React, FerrisUp makes it easy to start new Rust projects with the right structure and dependencies.

## Features

- **Multiple Templates**: From minimal binaries to full-stack applications
- **Transform Capability**: Start with a simple project and scale as you grow
- **GenAI Integration**: Ready-made AI model templates
- **Edge Computing**: WebAssembly and serverless-ready templates
- **Embedded Systems**: Support for RP2040, ESP32, STM32, and Arduino
- **Interactive CLI**: User-friendly command interface

## Installation

```bash
# Install from crates.io
cargo install jitpomi-ferrisup

# Or install from source
git clone https://github.com/Jitpomi/ferrisup.git
cd ferrisup
cargo install --path .
```

## Quick Start

```bash
# Create a new minimal project
jitpomi-ferrisup new my_project

# Create a full-stack project
jitpomi-ferrisup new my_fullstack --template=full-stack

# Create an AI project
jitpomi-ferrisup new my_ai_app --template=gen-ai

# Create an edge computing project
jitpomi-ferrisup new my_edge_app --template=edge-app

# Create an embedded systems project
jitpomi-ferrisup new my_embedded --template=embedded
```

## Available Templates

View all available templates:

```bash
jitpomi-ferrisup list
```

Current templates include:
- `minimal` - Simple binary with a single main.rs file
- `library` - Rust library crate with a lib.rs file
- `full-stack` - Complete application with client, server, and shared libraries
- `gen-ai` - AI-focused project with inference and model components
- `edge-app` - WebAssembly-based application for edge computing
- `embedded` - Embedded systems firmware for microcontrollers
- `serverless` - Serverless functions for cloud deployment
- `iot-device` - IoT device firmware with connectivity features
- `ml-pipeline` - Machine learning data processing pipeline
- `data-science` - Data science project with analysis tools

## Usage

```bash
jitpomi-ferrisup [OPTIONS]
```

### Commands

FerrisUp offers a variety of commands to help you bootstrap, transform, and manage your Rust projects:

### `new`

Create a new Rust project with a predefined structure.

```bash
jitpomi-ferrisup new [PROJECT_NAME] [--template TEMPLATE_NAME] [--git] [--build] [--no-interactive]
```

- `PROJECT_NAME`: Optional name for your project
- `--template`: Specify a template (web, api, full-stack, etc.)
- `--git`: Initialize a git repository
- `--build`: Run cargo build after creation
- `--no-interactive`: Create project without prompting, using default values

### `transform`

Transform an existing project into a FerrisUp template structure.

```bash
jitpomi-ferrisup transform [--project PATH] [--template TEMPLATE_NAME]
```

- `--project`: Path to existing project
- `--template`: Template to transform to

### `list`

List available templates.

```bash
jitpomi-ferrisup list
```

### `scale`

Interactively scale a project with custom components.

```bash
jitpomi-ferrisup scale
```

### `preview`

Preview a template without creating any files.

```bash
jitpomi-ferrisup preview [--template TEMPLATE_NAME]
```

- `--template`: Template to preview

### `component`

Manage project components (add/remove).

```bash
jitpomi-ferrisup component [--action ACTION] [--component-type TYPE] [--project PATH]
```

- `--action`: Action to perform: add, remove, or list
- `--component-type`: Component type: client, server, database, ai, edge, embedded, etc.
- `--project`: Path to the project

### `config`

Manage configurations (export/import).

```bash
jitpomi-ferrisup config [--export] [--import FILE] [--path PATH]
```

- `--export`: Export current configuration to a file
- `--import`: Import configuration from a file
- `--path`: Path to export/import configuration

### `workspace`

Manage a Cargo workspace structure for your Rust projects. This command helps you create and maintain modular projects with multiple crates organized in a workspace.

```bash
jitpomi-ferrisup workspace <SUBCOMMAND> [OPTIONS]
```

Subcommands:

- `init [PATH]`: Initialize a new workspace with standard directory structure (bin, libs, services, apps)
- `add <CRATE> [PATH]`: Add a new crate to the workspace
- `remove <CRATE> [PATH]`: Remove a crate from the workspace
- `list [PATH]`: List all workspace members
- `optimize [PATH]`: Find common dependencies and move them to workspace-level

Examples:

```bash
# Initialize a new workspace
jitpomi-ferrisup workspace init my_workspace

# Add a library crate to the workspace
jitpomi-ferrisup workspace add libs/core my_workspace

# Add a binary crate
jitpomi-ferrisup workspace add bin/cli my_workspace

# Add a service
jitpomi-ferrisup workspace add services/api my_workspace

# List all workspace members
jitpomi-ferrisup workspace list my_workspace

# Remove a crate from the workspace
jitpomi-ferrisup workspace remove libs/core my_workspace

# Optimize workspace dependencies
jitpomi-ferrisup workspace optimize my_workspace
```

This command supports the "Start Anywhere, Scale Anywhere" philosophy by allowing you to:
- Begin with a small modular project
- Gradually add components as your application grows
- Maintain clean boundaries between different parts of your system
- Share code through internal libraries
- Optimize dependency management across crates

## Workspace Management

FerrisUp provides comprehensive workspace management capabilities through the `workspace` command:

### Initialize a Workspace

```bash
jitpomi-ferrisup workspace --action init
```

This command will:
- Create a Cargo workspace configuration
- Discover existing crates or create new workspace structure
- Set up common dependencies in the workspace

### Add a Crate to Workspace

```bash
jitpomi-ferrisup workspace --action add
```

Add a new crate to your workspace:
- Select crate type (client, server, shared, custom)
- Specify crate name
- Choose between binary or library crate

### Remove a Crate

```bash
jitpomi-ferrisup workspace --action remove
```

Remove a crate from your workspace with the option to:
- Just remove it from workspace members
- Also delete the crate files

### List Workspace Members

```bash
jitpomi-ferrisup workspace --action list
```

Show all workspace members and discovered crates.

### Optimize Workspace

```bash
jitpomi-ferrisup workspace --action optimize
```

Optimize your workspace by:
- Ensuring all crates are included in workspace members
- Adding workspace.dependencies for common dependencies
- Applying best practices for workspace structure

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

### Template Compatibility

FerrisUp supports both new and old template formats, ensuring backward compatibility with existing templates. The system automatically converts older templates to the newer format.

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add some amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## Workflow

FerrisUp follows a simple workflow:

```
┌─────────────────────┐
│  Command Selection  │
│  new, transform,    │
│  scale, list        │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Project Location   │
│  Current dir or     │
│  specified path     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Template Selection │
│  Choose template or │
│  customize          │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Component Selection │
│ Client, Server,     │
│ Database, etc.      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Technology Stack    │
│ Framework, language │
│ and tools           │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   Configuration     │
│   Git, CI/CD,       │
│   Dependencies      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   Project Created   │
│   or Transformed    │
└─────────────────────┘
```

Each step is optional and can be skipped if you provide the information via command-line parameters. FerrisUp only prompts for what it needs to complete your task.

## Configuration

Configure through JSON or environment variables:

```json
{
  "project_name": "my_rust_app",
  "template": "full-stack",
  "components": {
    "client": {
      "apps": ["web", "mobile"],
      "frameworks": ["dioxus", "tauri"]
    },
    "server": {
      "services": ["api", "auth"],
      "frameworks": ["poem", "axum"]
    },
    "database": {
      "enabled": true,
      "engines": ["postgres", "redis", "neo4j", "milvus"],
      "migration_tool": "sqlx"
    },
    "libs": {
      "modules": ["core", "models", "auth"]
    },
    "binaries": {
      "apps": ["cli", "server", "worker"],
      "types": ["app", "service", "utility"]
    },
    "ai": {
      "models": ["llama", "whisper"],
      "backends": ["candle"],
      "features": ["text-generation", "speech-to-text"]
    },
    "edge": {
      "targets": ["wasm", "cloudflare-workers"],
      "features": ["serverless", "cdn-integration"]
    },
    "embedded": {
      "targets": ["rp2040", "esp32"],
      "features": ["no-std", "real-time", "low-power"]
    }
  },
  "dependencies": {
    "dioxus": { "version": "0.4", "features": ["web"] },
    "poem": { "version": "1.3" },
    "sqlx": { "version": "0.7", "features": ["postgres"] }
  },
  "templates": {
    "full-stack": ["client", "server", "database", "libs"],
    "backend-only": ["server", "database", "libs"],
    "frontend-only": ["client", "libs"],
    "api-service": ["server", "database", "libs"],
    "library": ["libs"],
    "minimal": ["binaries"],
    "hello-world": ["binaries", "libs"],
    "cli-app": ["binaries"],
    "gen-ai": ["ai", "server", "libs"],
    "edge-app": ["edge", "libs"],
    "iot-device": ["embedded", "libs"],
    "ml-pipeline": ["ai", "server", "database", "libs"],
    "serverless": ["server", "database", "libs"],
    "data-science": ["ai", "server", "database", "libs"]
  }
}
```

## Maximum Flexibility: From Hello World to Enterprise Scale

FerrisUp is designed with maximum flexibility in mind, allowing you to:

1. **Start Small**: Begin with a simple hello-world application
2. **Grow Incrementally**: Add components as your needs evolve
3. **Scale Massively**: Deploy to enterprise-grade infrastructure

### Starting from Minimal

Use the `--minimal` flag to create a bare-bones Rust project:

```bash
jitpomi-ferrisup new my_project --minimal
```

This creates a simple "Hello, World!" application with a clean workspace structure that's ready to expand.

### Binary Applications

FerrisUp supports standalone binary applications with the `binaries` component:

```json
"binaries": {
  "apps": ["cli", "server", "worker"],
  "types": ["app", "service", "utility"]
}
```

Perfect for command-line tools, background workers, or microservices.

### Template Selection

FerrisUp offers a variety of templates to match your project needs:

#### Core Templates
- **minimal** - Simple binary with a single main.rs file
- **library** - Rust library crate with a lib.rs file
- **full-stack** - Complete application with client, server, and shared libraries

#### Specialized Templates
- **gen-ai** - AI-focused project with inference and model components
  - LLaMA, BERT, Whisper, and Stable Diffusion integration options
  - Model serving and inference optimization
  
- **edge-app** - WebAssembly-based application for edge computing
  - Cloudflare Workers, Deno Deploy support
  - WASM optimized components
  
- **embedded** - Embedded systems firmware for microcontrollers
  - Support for RP2040, ESP32, STM32, Arduino
  - HAL abstraction and peripheral management
  
- **serverless** - Serverless functions for cloud deployment
  - AWS Lambda, Azure Functions, Google Cloud Functions
  - Local development and testing support
  
- **iot-device** - IoT device firmware with connectivity features
  - MQTT, HTTP, and Bluetooth connectivity
  - Secure boot and OTA update support
  
- **ml-pipeline** - Machine learning data processing pipeline
  - Data ingestion, transformation, and model training
  - Result visualization and export
  
- **data-science** - Data science project with analysis tools
  - Data loading and preprocessing
  - Analysis, visualization, and reporting

All templates can be fully customized during the interactive setup process. You're never locked into any specific technology stack or architecture.

## Enterprise Scaling

When you're ready to scale, use the `--scale` flag to generate deployment configurations:

```bash
jitpomi-ferrisup new my_enterprise_app --scale
```

This adds:

- **Docker configuration**: Optimized multi-stage build
- **Kubernetes manifests**: Deployment, service, and scaling
- **CI/CD pipelines**: GitHub Actions workflows
- **Cloud deployments**: AWS, GCP, Azure, and Digital Ocean templates

## Start Anywhere, Scale Anywhere

FerrisUp truly embodies the "start anywhere, scale anywhere" philosophy with its transformation capability. You can begin with the simplest project and evolve it as your needs grow:

### Transformation

```bash
# Start with a minimal project
jitpomi-ferrisup new my_project

# Later transform it to a library
jitpomi-ferrisup transform --project=my_project --template=library

# Add AI capabilities when needed
jitpomi-ferrisup transform --project=my_project --template=gen-ai

# Eventually scale to a full-stack application
jitpomi-ferrisup transform --project=my_project --template=full-stack

# Add enterprise scaling when ready for production
jitpomi-ferrisup scale --project=my_project
```

This transformation feature intelligently:
- Converts binaries to libraries when appropriate
- Preserves your existing code
- Adds only the components you need
- Updates your workspace configuration

## Project Structure

```
project_name/
├── Cargo.toml
├── client/
│   ├── app1/
│   ├── app2/
│   └── common/
├── server/
│   ├── service1/
│   ├── service2/
│   └── common/
├── database/
│   ├── migrations/
│   ├── schema/
│   ├── seeds/
│   └── src/
├── libs/
│   ├── core/
│   ├── models/
│   └── auth/
├── binaries/
│   ├── cli/
│   ├── server/
│   └── worker/
├── ai/
│   ├── models/
│   ├── backends/
│   └── features/
├── edge/
│   ├── targets/
│   └── features/
├── embedded/
│   ├── targets/
│   └── features/
└── deploy/
    ├── docker/
    ├── kubernetes/
    ├── github-actions/
    └── cloud-deployments/
```

## Database Integration

FerrisUp supports a comprehensive range of database technologies:

### SQL Databases
- **PostgreSQL**: Traditional relational database with advanced features
- **MySQL**: Popular open-source relational database
- **SQLite**: Lightweight file-based SQL database

### NoSQL Databases
- **Redis**: In-memory data structure store, cache, and message broker

### Vector Databases
- **Milvus**: Open-source vector database for similarity search and AI applications
- **Qdrant**: Vector database for vector similarity search

### Graph Databases
- **Neo4j**: Graph database with powerful query capabilities
- **DGraph**: Distributed graph database with GraphQL integration
- **ArangoDB**: Multi-model database for graphs, documents, and key-value pairs

### Time Series Databases
- **ScyllaDB**: High-performance NoSQL database compatible with Cassandra

### Semantic/Knowledge Databases
- **TypeDB**: Logical database for knowledge engineering

### P2P/Decentralized Databases
- **Iroh**: Rust-native distributed database for peer-to-peer applications
- **Hypercore**: Append-only log for peer-to-peer applications

### ORM/Query Builders
- **SQLx**: Async SQL toolkit with compile-time checked queries
- **Diesel**: Type-safe ORM with powerful query builder
- **SeaORM**: Async ORM with active record pattern

Each integration includes:
- Connection pool setup
- Environment variable configuration
- Error handling
- Migration structures (where applicable)
- Sample schemas and models

## Choosing Database Engines

Configure your database engines in the `config.json` file:

```json
"database": {
  "enabled": true,
  "engines": ["postgres", "redis", "neo4j", "milvus"],
  "migration_tool": "sqlx"
}
```

FerrisUp will automatically generate the appropriate code and connections for each selected engine.

## From Zero to Production

FerrisUp allows your project to evolve naturally:

1. Start with a minimal hello world app
2. Add libraries for shared functionality
3. Create service components as needed
4. Integrate databases when persistence is required
5. Add client interfaces when a UI is needed
6. Deploy to production with enterprise-grade infrastructure

Your app can remain as simple as needed or grow to any scale imaginable, all without changing tools or frameworks.

## Requirements

- Bash shell
- Rust and Cargo
- Git (optional)
- jq (recommended)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
