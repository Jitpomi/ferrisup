# {{project_name}} - Rust WebAssembly Static Site for Cloudflare Pages

This project is a static website built with Rust and WebAssembly, designed to be deployed on Cloudflare Pages. It provides a foundation for building interactive web applications that run at the edge.

## 📋 Features

- ⚡️ High-performance WebAssembly compiled from Rust
- 🌐 Ready for deployment on Cloudflare's global edge network
- 📱 Responsive design with modern CSS
- 🧩 Component-based structure for easy extensibility
- 🔧 Interactive UI elements with Rust-powered logic

## 🚀 Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (for Wrangler CLI)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/)

### Development

1. Install the Wrangler CLI if you haven't already:
   ```
   npm install -g wrangler
   ```

2. Build the WebAssembly binary:
   ```
   wasm-pack build --target web
   ```

3. Preview your site locally:
   ```
   npx wrangler pages dev .
   ```

4. Your site will be available at `http://localhost:8788`

### Deployment

1. Login to your Cloudflare account:
   ```
   wrangler login
   ```

2. Build and deploy your site:
   ```
   wasm-pack build --target web
   npx wrangler pages publish .
   ```

## 🔧 Customization

### Modifying the Rust Code

The main application logic is in `src/lib.rs`. This file contains the WebAssembly initialization and DOM manipulation code.

To add new features:

1. Modify the `init_app` function in `src/lib.rs`
2. Add new public functions with the `#[wasm_bindgen]` attribute to expose them to JavaScript
3. Rebuild with `wasm-pack build --target web`

### Styling and HTML

The HTML structure and CSS styling are in `index.html`. You can modify this file to change the layout and design of your site.

### Adding Pages

To add more pages to your static site:

1. Create additional HTML files in the root directory
2. Link to them from your main page
3. In each page, import and initialize the WebAssembly module as shown in the main `index.html`

## 📚 Advanced Features

### Working with Web APIs

To use more Web APIs from Rust, add the required features to the `web-sys` dependency in `Cargo.toml`:

```toml
web-sys = { version = "0.3", features = [
  "console",
  "Window",
  "Document",
  # Add more Web API features here
  "Fetch",
  "Headers",
  "Request",
  "Response"
]}
```

### Using External Libraries

You can add more Rust crates to enhance your project. For example, to add state management:

```toml
[dependencies]
# ... existing dependencies
yew = "0.20"
```

## 📖 Resources

- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [Cloudflare Pages Documentation](https://developers.cloudflare.com/pages/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
