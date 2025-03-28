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
cargo install ferrisup

# Or install from source
git clone https://github.com/jermsam/ferrisup.git
cd ferrisup
cargo install --path .
```

## Quick Start

```bash
# Create a new minimal project
ferrisup new my_project

# Create a full-stack project
ferrisup new my_fullstack --template=full-stack

# Create an AI project
ferrisup new my_ai_app --template=gen-ai

# Create an edge computing project
ferrisup new my_edge_app --template=edge-app

# Create an embedded systems project
ferrisup new my_embedded --template=embedded
```

## Available Templates

View all available templates:

```bash
ferrisup list
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
ferrisup [OPTIONS]
```

### Commands

All FerrisUp commands now feature a fully interactive mode:

- `new [name] [--template TEMPLATE] [--git] [--build]` - Create a new project 
- `transform [--project PATH] [--template TEMPLATE]` - Transform an existing project
- `list` - List available templates with descriptions
- `scale` - Interactive project builder with guided prompts

### Interactive Features

All commands in FerrisUp now support an interactive approach, allowing you to customize your project without memorizing command-line options:

#### Creating a New Project

```bash
# Fully interactive - prompts for all options
ferrisup new

# Semi-interactive - specify some options, prompt for others
ferrisup new my_project --template full-stack

# Non-interactive - specify all options
ferrisup new my_project --template full-stack --git --build
```

#### Transforming an Existing Project

```bash
# Fully interactive - prompts for all options
ferrisup transform

# Semi-interactive - prompt for template selection
ferrisup transform --project ./my_project
```

#### Interactive Project Builder (Scale Command)

The `scale` command provides a complete guided experience:

```bash
ferrisup scale
```

This walks you through:

1. **Project location** - Use current directory or specify a new one
2. **Template selection** - Choose from predefined templates or customize from scratch
3. **Project type** - Select between binary, library, or workspace
4. **Component selection** - For workspaces, choose components to include:
   - Client applications (with framework selection: Dioxus, Tauri, Leptos, Yew)
   - Server services (with framework selection: Poem, Axum, Actix Web, Rocket, Warp)
   - Shared libraries 
   - Database support (PostgreSQL, MySQL, SQLite, MongoDB, Redis, DynamoDB)
   - AI components (Text Generation, Image Generation, Speech Recognition, etc.)
   - Edge computing (WebAssembly, Cloudflare Workers, Deno Deploy, etc.)
   - Embedded systems (RP2040, ESP32, STM32, Arduino)

### Freedom of Choice

FerrisUp is designed to be flexible. Whether you're using templates or starting from scratch:

- You're never locked into any specific tech stack
- All components and features are customizable
- You can always add, remove, or modify components later using the `transform` command

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
ferrisup new my_project --minimal
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

Choose the right starting point for your project:

- **minimal**: Just a binary application with "Hello World"
- **hello-world**: Minimal binary plus shared libraries
- **full-stack**: Complete client-server application with database
- **backend-only**: Server services with database support
- **frontend-only**: Client applications with shared libraries
- **api-service**: API-focused server with database
- **library**: Pure library crate
- **cli-app**: Command-line application
- **gen-ai**: AI components with server and libraries
- **edge-app**: Edge computing application with libraries
- **iot-device**: Embedded systems for IoT use cases
- **ml-pipeline**: Complete machine learning pipeline with AI, server and database
- **serverless**: Serverless functions with database
- **data-science**: Data analysis components with AI and database

### Enterprise Scaling

When you're ready to scale, use the `--scale` flag to generate deployment configurations:

```bash
ferrisup new my_enterprise_app --scale
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
ferrisup new my_project

# Later transform it to a library
ferrisup transform --project=my_project --template=library

# Add AI capabilities when needed
ferrisup transform --project=my_project --template=gen-ai

# Eventually scale to a full-stack application
ferrisup transform --project=my_project --template=full-stack

# Add enterprise scaling when ready for production
ferrisup scale --project=my_project
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
