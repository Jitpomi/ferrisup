# {{project_name}}

A web server implementation using Rust and the **{{server_framework}}** framework.

## Features

{{#if (eq server_framework "axum")}}
- Type-safe routing with Axum
- Tower middleware integration 
- Extensible extractors
- JSON request/response handling
- Async request processing with Tokio
{{else}}{{#if (eq server_framework "actix")}}
- High-performance HTTP handling
- Support for HTTP/1.x and HTTP/2
- Actix actor system integration
- Flexible middleware system
- WebSockets support
{{else}}{{#if (eq server_framework "poem")}}
- Ergonomic API design with Poem
- OpenAPI integration capability
- Middleware support
- WebSockets capabilities
- Multipart file handling
{{/if}}{{/if}}{{/if}}

## Getting Started

### Prerequisites

- Rust toolchain (1.70.0 or later recommended)
- Cargo package manager

### Running the Server

```bash
cd {{project_name}}
cargo run
```

The server will start and listen on:
{{#if (eq server_framework "actix")}}
- http://localhost:8080
{{else}}
- http://localhost:3000
{{/if}}

### Testing the API

```bash
# Health check
{{#if (eq server_framework "actix")}}
curl http://localhost:8080/
{{else}}
curl http://localhost:3000/
{{/if}}

# Echo endpoint (sends back the JSON you provide)
{{#if (eq server_framework "actix")}}
curl -X POST http://localhost:8080/echo -H "Content-Type: application/json" -d '{"message": "Hello World"}'
{{else}}
curl -X POST http://localhost:3000/echo -H "Content-Type: application/json" -d '{"message": "Hello World"}'
{{/if}}
```

## Framework Details

{{#if (eq server_framework "axum")}}
### About Axum

Axum is a web application framework focusing on ergonomics and modularity. It's built on top of Tokio, Tower, and Hyper.

Resources:
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [GitHub Repository](https://github.com/tokio-rs/axum)
- [Tokio Discord](https://discord.gg/tokio)
{{else}}{{#if (eq server_framework "actix")}}
### About Actix Web

Actix Web is a powerful, pragmatic, and extremely fast web framework for Rust, built on top of the actor system provided by the Actix crate.

Resources:
- [Actix Web Documentation](https://actix.rs/docs/)
- [GitHub Repository](https://github.com/actix/actix-web)
- [Discord Community](https://discord.gg/actix)
{{else}}{{#if (eq server_framework "poem")}}
### About Poem

Poem is a full-featured, easy-to-use web framework with a focus on ergonomics, modularity, and extensibility.

Resources:
- [Poem Documentation](https://docs.rs/poem/latest/poem/)
- [GitHub Repository](https://github.com/poem-web/poem)
- [API Reference](https://docs.rs/poem)
{{/if}}{{/if}}{{/if}}

## Project Structure

```
{{project_name}}/
├── src/
│   └── main.rs       # Server application entry point
├── Cargo.toml        # Project dependencies
└── README.md         # This file
```

## Potential Enhancements

{{#if (eq server_framework "axum")}}
- Add Tower middleware for authentication
- Implement API versioning
- Add OpenAPI/Swagger with utoipa
- Add database integration (SQLx recommended for Axum)
{{else}}{{#if (eq server_framework "actix")}}
- Implement actor-based background workers
- Add database integration with Diesel
- Create RESTful resource handlers
- Add authentication middleware
{{else}}{{#if (eq server_framework "poem")}}
- Add OpenAPI integration with poem-openapi
- Implement API versioning
- Add database connectivity
- Create comprehensive error handling
{{/if}}{{/if}}{{/if}}

## License

This project is licensed under the MIT License.
