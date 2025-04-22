# Actix Web Server

This template provides a foundation for building high-performance web servers and APIs using the Actix Web framework, one of the fastest web frameworks available for Rust.

## Features

- Async runtime with Actix's actor system
- Powerful routing system
- JSON serialization/deserialization with Serde
- Middleware support
- Structured logging
- Easy testing with actix-rt
- High performance and scalability

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
   curl http://localhost:8080/
   curl http://localhost:8080/hello/world
   ```

## Project Structure

- `src/main.rs`: Main application entry point with route definitions
- `Cargo.toml`: Project dependencies and configuration

## Customization

### Adding Routes

Add new routes in the `main.rs` file:

```rust
// Add a new route
app.service(web::resource("/users/{id}").route(web::get().to(get_user)));

// Define the handler
async fn get_user(id: web::Path<String>) -> impl Responder {
    format!("User ID: {}", id)
}
```

### Adding Middleware

Actix Web supports middleware for request/response processing:

```rust
use actix_web::middleware::{Logger, Compress};

// In your app configuration
App::new()
    .wrap(Logger::default())
    .wrap(Compress::default())
    .service(hello)
```

### Error Handling

Implement custom error handling:

```rust
#[derive(Debug)]
enum MyError {
    NotFound,
    InternalError,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        match self {
            MyError::NotFound => HttpResponse::NotFound().json("Resource not found"),
            MyError::InternalError => HttpResponse::InternalServerError().json("Internal server error"),
        }
    }
}
```

## Next Steps

- Add database integration (e.g., SQLx, Diesel)
- Implement authentication and authorization
- Add OpenAPI documentation
- Set up containerization with Docker
- Configure deployment to cloud platforms

## Resources

- [Actix Web Documentation](https://actix.rs/docs/)
- [Actix Web Examples](https://github.com/actix/examples)
- [Actix Web API Guidelines](https://actix.rs/docs/application/)
- [Serde Documentation](https://serde.rs/)
