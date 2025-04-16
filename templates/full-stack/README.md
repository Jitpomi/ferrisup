# Rust Full-Stack Application

This template provides a complete foundation for building full-stack applications entirely in Rust, with client, server, and shared libraries organized in a workspace structure.

## Features

- Workspace-based project organization
- Multiple client applications with Dioxus and Tauri
- Server services with Poem
- Shared libraries for code reuse
- Database integration with PostgreSQL and SQLx
- Comprehensive error handling
- Observability with metrics and tracing
- Deployment options for various platforms

## Getting Started

After generating your project with FerrisUp, follow these steps:

1. Navigate to your project directory:
   ```bash
   cd your-project-name
   ```

2. Set up the database:
   ```bash
   # Install PostgreSQL if needed
   # Create a database
   createdb your_database_name
   
   # Set the database URL environment variable
   export DATABASE_URL=postgres://username:password@localhost/your_database_name
   ```

3. Build the entire workspace:
   ```bash
   cargo build
   ```

4. Run the server:
   ```bash
   cargo run -p api
   ```

5. Run the client (choose one):
   ```bash
   # For Dioxus web client
   cd client/app1
   trunk serve
   
   # For Tauri desktop client
   cd client/app2
   cargo tauri dev
   ```

## Project Structure

```
your-project-name/
├── Cargo.toml           # Workspace configuration
├── client/              # Client applications
│   ├── app1/            # Dioxus web application
│   │   ├── Cargo.toml
│   │   └── src/
│   └── app2/            # Tauri desktop application
│       ├── Cargo.toml
│       └── src/
├── server/              # Server services
│   ├── api/             # Main API service
│   │   ├── Cargo.toml
│   │   └── src/
│   └── auth/            # Authentication service
│       ├── Cargo.toml
│       └── src/
└── libs/                # Shared libraries
    ├── core/            # Core utilities
    │   ├── Cargo.toml
    │   └── src/
    ├── models/          # Shared data models
    │   ├── Cargo.toml
    │   └── src/
    └── auth/            # Authentication library
        ├── Cargo.toml
        └── src/
```

## Client Applications

### Dioxus Web Application

The template includes a Dioxus-based web application:

```rust
use dioxus::prelude::*;

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div { class: "app",
            h1 { "Hello from Dioxus!" }
            p { "Welcome to your full-stack Rust application." }
        }
    })
}

fn main() {
    dioxus_web::launch(App);
}
```

### Tauri Desktop Application

The template also includes a Tauri-based desktop application:

```rust
fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Server Services

### API Service

The main API service uses Poem:

```rust
use poem::{get, handler, Route, Server};

#[handler]
fn hello() -> String {
    "Hello, World!".to_string()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("/", get(hello));
    
    Server::new(tokio::net::TcpListener::bind("0.0.0.0:3000").await?)
        .run(app)
        .await
}
```

### Authentication Service

A separate service for authentication:

```rust
// Authentication service implementation
```

## Shared Libraries

### Core Library

Contains utilities used across the application:

```rust
// Error handling, logging, configuration, etc.
```

### Models Library

Defines data structures shared between client and server:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}
```

### Auth Library

Shared authentication logic:

```rust
// JWT handling, password hashing, etc.
```

## Database Integration

The template uses SQLx for database access:

```rust
use sqlx::PgPool;

pub async fn get_connection_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    PgPool::connect(&database_url).await
}
```

## Observability

### Metrics

```rust
use prometheus::{register_counter, Counter};

lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: Counter = register_counter!(
        "http_requests_total",
        "Total number of HTTP requests"
    ).unwrap();
}
```

### Tracing

```rust
use tracing::{info, instrument};

#[instrument]
async fn process_request() {
    info!("Processing request");
    // Request handling logic
}
```

## Deployment

The template supports deployment to various platforms:

- Heroku
- Vercel
- Netlify
- Fly.io
- Railway
- Render

## Customization

### Adding a New Client Application

1. Create a new directory in the `client` folder
2. Add a new package to the workspace in the root `Cargo.toml`
3. Set up the client framework of your choice

### Adding a New Server Service

1. Create a new directory in the `server` folder
2. Add a new package to the workspace in the root `Cargo.toml`
3. Implement your service using Poem or another framework

### Adding a New Shared Library

1. Create a new directory in the `libs` folder
2. Add a new package to the workspace in the root `Cargo.toml`
3. Implement your shared functionality

## Next Steps

- Implement authentication and authorization
- Set up database migrations
- Add API documentation with OpenAPI
- Implement real-time communication with WebSockets
- Set up CI/CD pipelines
- Add end-to-end testing

## Resources

- [Dioxus Documentation](https://dioxuslabs.com/)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Poem Documentation](https://docs.rs/poem/latest/poem/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [Rust Workspace Documentation](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
