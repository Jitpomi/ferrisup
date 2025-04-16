# Rust Edge Computing Application

This template provides a foundation for building Rust applications that run at the edge using WebAssembly. It's designed to help you quickly get started with edge computing platforms like Cloudflare Workers, Deno Deploy, or Netlify Edge Functions.

## Features

- WebAssembly compilation target for running Rust at the edge
- Browser testing options (example page and/or headless tests)
- Static file server options for local development
- Support for multiple edge computing platforms
- Optimized for small bundle sizes and fast performance

## Getting Started

After generating your project with FerrisUp, follow these steps:

1. Navigate to your project directory:
   ```bash
   cd your-project-name
   ```

2. Build your WebAssembly package:
   ```bash
   wasm-pack build --target web
   ```

3. Test your application:
   - For browser testing: Open `index.html` in your browser or use the selected static file server
   - For headless testing: Run `wasm-pack test --headless --firefox`

## Project Structure

- `src/lib.rs`: Main library code with WebAssembly exports
- `Cargo.toml`: Project dependencies and configuration
- `index.html`: Browser example page (if selected)
- `tests/web.rs`: Headless browser tests (if selected)

## Customization

### Testing Approach

You can choose from three testing approaches:
- **Browser Example**: Creates an HTML page to manually test your WebAssembly in a browser
- **Headless Tests**: Sets up automated tests that run in a headless browser
- **Both**: Includes both browser example and headless tests

### Static File Server

For local development, you can choose from:
- **miniserve**: A simple, zero-configuration static file server
- **static-web-server**: A fast and asynchronous web server for static files
- **None**: No static file server included

### Edge Platform Support

The template includes support for multiple edge computing platforms:
- **Cloudflare Workers**: Using the `worker` and `worker-macros` crates
- **Deno Deploy**: Using the `deno_core` and `deno_runtime` crates
- **Netlify Edge Functions**: Using the `netlify_lambda_http` crate

## Next Steps

- Add your business logic to `src/lib.rs`
- Customize the WebAssembly exports to match your requirements
- Deploy to your chosen edge computing platform
- Optimize your code for size and performance

## Resources

- [Rust WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/docs/wasm-bindgen/)
- [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)
- [Deno Deploy Documentation](https://deno.com/deploy/docs)
- [Netlify Edge Functions Documentation](https://docs.netlify.com/edge-functions/overview/)
