# Poem Web Server

This template provides a foundation for building web servers and APIs using the Poem framework, a full-featured and easy-to-use web framework for Rust.

## Features

- Async runtime with Tokio
- Elegant routing system
- JSON serialization/deserialization with Serde
- Middleware support
- Structured logging with tracing
- WebSocket support
- OpenAPI integration capabilities

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
   curl http://localhost:3000/api/info
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
async fn get_user(Path(id): Path<String>) -> Result<String> {
    Ok(format!("User ID: {}", id))
}
```

### Adding Middleware

Poem supports middleware for request/response processing:

```rust
use poem::middleware::Tracing;

// In your app configuration
let app = Route::new()
    .at("/", get(hello))
    .with(Tracing);
```

### Error Handling

Implement custom error handling:

```rust
#[derive(Debug, Error)]
enum MyError {
    #[error("Resource not found")]
    NotFound,
    #[error("Internal server error")]
    InternalError,
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        match self {
            MyError::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Resource not found"),
            MyError::InternalError => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Internal server error"),
        }
    }
}
```

## Next Steps

- Add database integration (e.g., SQLx, Diesel)
- Implement authentication and authorization
- Add OpenAPI documentation with Poem's OpenAPI support
- Set up containerization with Docker
- Configure deployment to cloud platforms

## Resources

- [Poem Documentation](https://docs.rs/poem/latest/poem/)
- [Poem GitHub Repository](https://github.com/poem-web/poem)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [Serde Documentation](https://serde.rs/)
