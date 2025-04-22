# Axum Web Server

This template provides a foundation for building web servers and APIs using the Axum framework, a modern, fast, and ergonomic web framework for Rust.

## Features

- Async runtime with Tokio
- Routing with Axum's powerful router
- JSON serialization/deserialization with Serde
- Middleware support with Tower
- Logging and tracing infrastructure
- Type-safe request handling
- Easy testing with reqwest

## Getting Started

After generating your project with FerrisUp, follow these steps:

1. Navigate to your project directory:
   ```bash
   cd {{project_name}}
   ```

2. Run the server:
   ```bash
   cargo run
   ```

3. Test the API:
   ```bash
   curl http://localhost:3000/
   curl http://localhost:3000/hello/world
   ```

## Project Structure

- `src/main.rs`: Main application entry point with route definitions
- `Cargo.toml`: Project dependencies and configuration

## Customization

### Adding Routes

Add new routes in the `main.rs` file:

```rust
// Add a new route
app.route("/users/:id", get(get_user))

// Define the handler
async fn get_user(Path(id): Path<String>) -> impl IntoResponse {
    format!("User ID: {}", id)
}
```

### Adding Middleware

Axum uses Tower middleware. Add middleware to your application:

```rust
let app = Router::new()
    .route("/", get(root))
    .layer(TraceLayer::new_for_http());
```

### Error Handling

Implement custom error handling:

```rust
#[derive(Debug)]
enum AppError {
    NotFound,
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
        }
    }
}
```

## Next Steps

- Add database integration (e.g., SQLx, Diesel)
- Implement authentication and authorization
- Add OpenAPI documentation with utoipa
- Set up containerization with Docker
- Configure deployment to cloud platforms

## Resources

- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [Tower Documentation](https://docs.rs/tower/latest/tower/)
- [Serde Documentation](https://serde.rs/)
