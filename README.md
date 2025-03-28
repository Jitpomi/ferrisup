# FerrisUp 

Flexible Rust workspace initializer for modular applications, now with enhancements for GenAI, edge computing, and embedded systems.

## Overview

FerrisUp bootstraps Rust projects with a well-organized workspace structure, supporting various templates and customization options. With the latest updates, FerrisUp now includes features for GenAI, edge computing, and embedded systems, making it a versatile tool for a wide range of applications.

## Features

- **Multiple Templates**: full-stack, backend-only, frontend-only, api-service, library, gen-ai, edge-app, iot-device, ml-pipeline, serverless, data-science
- **Flexible Configuration**: JSON config or environment variables
- **Framework Support**: 
  - Frontend: Dioxus, Tauri, Yew, Leptos
  - Backend: Poem, Axum, Actix-web, Rocket, Tide
  - AI: LLaMA, BERT, Whisper, Stable Diffusion
  - Edge Computing: WASM, Cloudflare Workers, Deno Deploy, Netlify Functions
  - Embedded Systems: RP2040, ESP32, STM32, Arduino
- **Database Integration**:
  - Engines: PostgreSQL, MySQL, SQLite, Redis, Milvus, Qdrant, Neo4j, DGraph, ArangoDB, ScyllaDB, TypeDB, Iroh, Hypercore
  - ORM/Query Builders: SQLx, Diesel, SeaORM
  - Migration tools and sample schemas
- **Modular Structure**: client apps, server services, database, shared libraries, AI components, edge computing components, embedded systems components
- **Git Integration**: Automatic repository initialization

## Usage

```bash
./ferrisup.sh [OPTIONS]
```

### Options

- `-h, --help`: Show help
- `-c, --config FILE`: Use config file (default: config.json)
- `-t, --template NAME`: Select template
- `-n, --name NAME`: Set project name
- `--skip-git`: Skip Git initialization
- `--list-templates`: List templates
- `--minimal`: Create a bare-bones Rust project
- `--scale`: Generate deployment configurations
- `--gen-ai`: Create a project with AI capabilities
- `--edge-app`: Create a project for edge computing
- `--iot-device`: Create a project for embedded systems

### Examples

```bash
# Default full-stack project
./ferrisup.sh

# Backend-only project named "api-server"
./ferrisup.sh -t backend-only -n api-server

# Custom configuration file
./ferrisup.sh -c my-config.json

# Minimal hello world application
./ferrisup.sh --minimal -n my_project

# Enterprise-scale application
./ferrisup.sh --scale -n my_enterprise_app

# Project with AI capabilities
./ferrisup.sh --gen-ai -n my_ai_project

# Edge computing project
./ferrisup.sh --edge-app -n my_edge_project

# Embedded systems project
./ferrisup.sh --iot-device -n my_iot_project
```

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
./ferrisup.sh --minimal -n my_project
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
./ferrisup.sh --scale -n my_enterprise_app
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
./ferrisup.sh my_project --minimal

# Later transform it to a library
./ferrisup.sh --transform=library --project=my_project

# Add AI capabilities when needed
./ferrisup.sh --transform=gen-ai --project=my_project

# Eventually scale to a full-stack application
./ferrisup.sh --transform=full-stack --project=my_project

# Add enterprise scaling when ready for production
./ferrisup.sh --scale --project=my_project
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
