# {{project_name}} - Cloudflare Workers API

This is a Rust-powered Cloudflare Workers API template, generated with [FerrisUp](https://github.com/Jitpomi/ferrisup).

## Development Options

This template provides two ways to develop your Cloudflare Workers application:

### Option 1: Using Docker (Recommended)

The Docker environment comes pre-configured with Rust 1.69.0 and all necessary tools, providing a consistent development experience regardless of your local Rust version.

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
   This will start a Wrangler development server accessible at http://localhost:8787

3. To build the application:
   ```bash
   ./build.sh
   ```

4. To deploy to Cloudflare:
   ```bash
   ./deploy.sh
   ```
   
5. To stop the development environment:
   ```bash
   ./stop.sh
   ```

### Option 2: Local Development

If you prefer to develop without Docker, you can use your local Rust installation.

#### Important Compatibility Note

Due to dependency compatibility issues between the `worker` crate, `wasm-bindgen`, and modern Rust versions, this template works best with **Rust 1.69.0**. If you encounter build errors, try using this specific Rust version:

```bash
rustup install 1.69.0
rustup override set 1.69.0
```

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) - Version 1.69.0 recommended
- [wrangler](https://developers.cloudflare.com/workers/wrangler/install-and-update/) - Cloudflare Workers CLI
- [Node.js](https://nodejs.org/) - Required for wrangler

#### Getting Started with Local Development

1. Install the recommended Rust version (if needed):

```bash
rustup install 1.69.0
rustup override set 1.69.0
```

2. Install the wrangler CLI:

```bash
npm install -g wrangler
```

3. Login to Cloudflare:

```bash
wrangler login
```

4. Build the project:

```bash
cargo build --release
```

5. Deploy to Cloudflare Workers:

```bash
wrangler deploy
```

## Local Development

To test locally:

```bash
wrangler dev
```

This will start a local server, typically on http://localhost:8787.

## Available Endpoints

- `GET /` - HTML landing page
- `GET /api` - Returns a JSON response
- `POST /api` - Accepts JSON data and returns it
- `GET /api/kv-example` - Shows KV storage example
- `GET /api/env-example` - Shows environment variables example

## Project Structure

- `src/lib.rs` - Main API code with routing and handlers
- `wrangler.toml` - Cloudflare Workers configuration
- `Dockerfile` & `docker-compose.yml` - Docker configuration (if using Docker)
- Helper scripts (`*.sh`) - Simplify Docker-based development

## Customization

This template includes:

- A simple REST API with example endpoints
- HTML landing page
- JSON request/response handling
- Environment variable and KV storage examples
- Docker development environment for consistent builds

## Troubleshooting

### Docker Issues
- If you encounter permission issues with the scripts, ensure they're executable: `chmod +x *.sh`
- If port 8787 is already in use, modify the port mapping in `docker-compose.yml`

### Dependency Issues
If you're developing locally and encounter dependency compatibility issues:

1. Make sure you're using Rust 1.69.0 as recommended
2. Clear your Cargo cache and lock file: `rm -f Cargo.lock`
3. Try building with the `--locked` flag: `cargo build --release --locked`
4. If problems persist, check the [Cloudflare Workers Rust documentation](https://developers.cloudflare.com/workers/runtime-apis/webassembly/rust/) for the latest guidance

## Learn More

- [Cloudflare Workers documentation](https://developers.cloudflare.com/workers/)
- [worker-rs Crate Documentation](https://docs.rs/worker/latest/worker/)
- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [Rust on Cloudflare Workers](https://developers.cloudflare.com/workers/runtime-apis/webassembly/rust/)

---

Generated with ❤️ by [FerrisUp](https://github.com/Jitpomi/ferrisup)
