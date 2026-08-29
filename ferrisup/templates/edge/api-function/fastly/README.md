# {{project_name}} - Rust Fastly Compute@Edge Application

This project is a Rust application built for Fastly's Compute@Edge platform. It leverages WebAssembly for high-performance edge computing with global deployment capabilities.

## Features

- Blazing-fast performance with Rust compiled to WebAssembly
- Global deployment across Fastly's edge network
- Simple API routing with request and response handling
- JSON serialization and deserialization
- Cache control demonstration
- Low latency for end-users worldwide

## Development Options

This template provides two ways to develop your Fastly Compute@Edge application:

### Option 1: Using Docker (Recommended)

The Docker environment provides the Rust and Fastly tooling used by the helper scripts. Rebuild the image after dependency or toolchain changes.

#### Prerequisites
- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

#### Getting Started with Docker
1. Make the helper scripts executable:
   ```bash
   chmod +x *.sh
   ```

2. Start the development environment:
   ```bash
   ./dev.sh
   ```
   This will start a Fastly Compute@Edge local server accessible at http://localhost:7676

3. To build the application:
   ```bash
   ./build.sh
   ```

4. To deploy to Fastly:
   ```bash
   ./deploy.sh
   ```

5. To stop the development environment:
   ```bash
   ./stop.sh
   ```

### Option 2: Local Development

If you prefer to develop without Docker, you can use your local Rust installation.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) stable
- The WebAssembly target required by the current Fastly CLI; recent Rust toolchains provide `wasm32-wasip1`
- [Fastly CLI](https://developer.fastly.com/learning/tools/cli)

## Getting Started

1. Install the Fastly CLI if you haven't already:
   ```
   brew install fastly/tap/fastly
   ```

2. Log in to your Fastly account:
   ```
   fastly profile create
   ```

3. Build the project:
   ```
   fastly compute build
   ```

4. Start the local development server:
   ```
   fastly compute serve
   ```

5. Your application will be available at `http://127.0.0.1:7676`

## Deployment

1. Build the project for deployment:
   ```
   fastly compute build
   ```

2. Deploy to Fastly Compute@Edge:
   ```
   fastly compute deploy
   ```

3. Your application will be available at the provided Fastly domain (e.g., `https://your-service.edgecompute.app`)

## API Documentation

### Available Endpoints

- `GET /` - Returns a simple HTML page
- `GET /api` - Returns a JSON response with a greeting message
- `GET /api/echo?message=your-message` - Echo back the provided message as JSON
- `GET /api/cache` - Demonstrates cache control with appropriate headers

### Example Requests

```bash
# Get the default API response
curl -X GET "https://your-service.edgecompute.app/api"

# Echo a message
curl -X GET "https://your-service.edgecompute.app/api/echo?message=hello-world"

# Check cache headers
curl -X GET "https://your-service.edgecompute.app/api/cache" -v
```

## Customization

### Adding New Routes

Modify the `main` function in `src/main.rs` to add new routes:

```rust
match (method, path) {
    // Existing routes...

    // Add your new route here
    (&Method::GET, "/api/new-endpoint") => {
        // Your handler code
        let response = ApiResponse {
            message: "Your new endpoint response".to_string(),
            status: "success".to_string(),
            timestamp: current_timestamp(),
            path: Some("/api/new-endpoint".to_string()),
            method: Some("GET".to_string()),
        };

        Ok(Response::from_body(serde_json::to_string(&response)?)
            .with_status(StatusCode::OK)
            .with_content_type("application/json"))
    },

    // Default 404 handler
    _ => { /* ... */ }
}
```

### Working with Backends

Fastly Compute@Edge can interact with origin servers and other backends. To configure a backend:

1. Define it in your `fastly.toml`:
   ```toml
   [setup.backends]
     [setup.backends.my_api]
       url = "https://api.example.com"
   ```

2. Use it in your code:
   ```rust
   let backend_req = Request::get("https://api.example.com/data")
       .with_backend("my_api");

   let resp = backend_req.send("my_api")?;
   ```

### Using Dictionaries and KV Stores

Fastly offers edge dictionaries and KV stores for configuration and data storage:

```rust
use fastly::dictionary::Dictionary;

// Access a dictionary
let settings = Dictionary::open("settings");
if let Some(api_key) = settings.get("api_key") {
    // Use the API key
}
```

## Advanced Features

### Edge Dictionaries

Edge Dictionaries provide a way to store key-value pairs that you can access in your code:

1. Create a dictionary in the Fastly UI or via the API
2. Access it in your code:
   ```rust
   let config = Dictionary::open("config");
   let value = config.get("some_key").unwrap_or("default");
   ```

### VCL Interoperability

Compute@Edge applications can interact with VCL services:

1. Set up a Compute@Edge service that processes certain paths
2. Use VCL for other paths or for specific behaviors

### Secrets Management

For sensitive data, use Fastly's secrets management:

```rust
use fastly::secret_store::SecretStore;

let secrets = SecretStore::open("my_secrets");
if let Some(api_key) = secrets.get("api_key") {
    // Use the API key securely
}
```

## Resources

- [Fastly Compute@Edge Documentation](https://developer.fastly.com/learning/compute/)
- [Fastly Rust SDK Documentation](https://docs.rs/fastly/latest/fastly/)
- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
