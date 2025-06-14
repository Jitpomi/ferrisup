# {{project_name}} - Rust WebAssembly Static Site for Netlify

This project is a static website built with Rust and WebAssembly, designed to be deployed on Netlify's global CDN. It provides a fast, interactive user experience with near-native performance.

## 📋 Features

- ⚡️ High-performance WebAssembly compiled from Rust
- 🎨 Modern, responsive design with mobile-first approach
- 🌐 Ready for deployment on Netlify's global CDN
- 🔄 Interactive UI elements powered by Rust
- 🚀 Continuous deployment with Git integration

## 🚀 Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (for Netlify CLI)
- [Netlify CLI](https://docs.netlify.com/cli/get-started/) (optional for local development)

### Development

1. Build the WebAssembly binary:
   ```
   wasm-pack build --target web
   ```

2. Install the Netlify CLI if you haven't already:
   ```
   npm install -g netlify-cli
   ```

3. Start a local development server:
   ```
   netlify dev
   ```
   
   Alternatively, you can use any static file server:
   ```
   # If you have Python installed
   python -m http.server
   
   # If you have Node.js installed
   npx serve
   ```

4. Your site will be available at the provided local URL (typically http://localhost:8888 or http://localhost:8000)

### Deployment

1. Login to your Netlify account (if not already logged in):
   ```
   netlify login
   ```

2. Initialize Netlify in your project:
   ```
   netlify init
   ```

3. Deploy to Netlify:
   ```
   netlify deploy
   ```

4. For production deployment:
   ```
   netlify deploy --prod
   ```

## 🔧 Customization

### Project Structure

```
{{project_name}}/
├── src/
│   └── lib.rs           # Main Rust code for WebAssembly
├── index.html           # Main HTML file
├── Cargo.toml           # Rust dependencies
└── netlify.toml         # Netlify configuration
```

### Modifying the Rust Code

The main application logic is in `src/lib.rs`. This file contains the WebAssembly initialization and DOM manipulation code organized into distinct sections:

- `init_app()`: Main entry point for your application
- `create_header()`: Creates the site header
- `create_hero()`: Creates the hero section
- `create_features()`: Creates the features section
- `create_demo()`: Creates the interactive demo section
- `create_footer()`: Creates the site footer

To modify any section:

1. Locate the appropriate function in `src/lib.rs`
2. Make your changes to the HTML structure and content
3. Rebuild with `wasm-pack build --target web`

### Styling and HTML

The CSS styles are embedded directly in `index.html`. The stylesheet is organized into sections:

- General styling and variables
- Header and navigation
- Hero section
- Features section
- Demo section
- Footer
- Responsive styles

You can modify these styles to match your branding and design preferences.

## 📚 Advanced Features

### Adding Pages

To add more pages to your static site:

1. Create a new HTML file in the root directory
2. Copy the basic structure from `index.html`
3. Add content specific to the new page
4. Update navigation LINKS in `create_header()` function

### Using External Libraries

You can add more Rust crates to enhance your project:

1. Add the dependency to `Cargo.toml`:
   ```toml
   [dependencies]
   # Existing dependencies...
   serde = { version = "1.0", features = ["derive"] }  # Example: Adding Serde for serialization
   ```

2. Import and use the library in your `lib.rs` file

### Working with APIs

To fetch data from external APIs:

1. Update the `web-sys` features in `Cargo.toml` to include:
   ```toml
   web-sys = { version = "0.3", features = [
     # Existing features...
     "Request",
     "RequestInit",
     "RequestMode",
     "Response",
     "Headers"
   ]}
   ```

2. Use `wasm-bindgen-futures` to make asynchronous requests:
   ```rust
   use wasm_bindgen_futures::JsFuture;
   use web_sys::{Request, RequestInit, RequestMode, Response};

   // Example fetch function
   async fn fetch_data(url: &str) -> Result<JsValue, JsValue> {
       let mut opts = RequestInit::new();
       opts.method("GET");
       opts.mode(RequestMode::Cors);
       
       let request = Request::new_with_str_and_init(url, &opts)?;
       let window = web_sys::window().unwrap();
       let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
       let resp: Response = resp_value.dyn_into()?;
       let json = JsFuture::from(resp.json()?).await?;
       
       Ok(json)
   }
   ```

### Environment Variables

1. Add environment variables in the Netlify UI under Site settings > Build & deploy > Environment
2. Access them in your JavaScript:
   ```javascript
   const apiKey = process.env.API_KEY;
   ```

3. For local development, create a `.env` file and use it with Netlify CLI

## 📦 Optimizing for Production

### Bundle Size Optimization

For smaller WebAssembly bundles:

1. Ensure release mode builds:
   ```
   wasm-pack build --target web --release
   ```

2. Use feature flags to control what's included:
   ```toml
   [features]
   minimal = [] # Define a minimal feature set
   ```

3. Consider using `wee_alloc` (already included) for smaller binaries

### Performance Optimization

For best performance:

1. Consider lazy-loading the WebAssembly module
2. Add a loading state to improve perceived performance
3. Use the Intersection Observer API to defer non-critical operations

### Netlify Configuration

Create a `netlify.toml` file in your project root:

```toml
[build]
  publish = "."
  command = "wasm-pack build --target web --release"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

### Headers and Caching

Add custom headers for WebAssembly files:

```toml
[[headers]]
  for = "/*.wasm"
  [headers.values]
    Content-Type = "application/wasm"
    Cache-Control = "public, max-age=31536000"
```

## 🛠️ Troubleshooting

### Common Issues

1. **WebAssembly not loading**
   - Check browser console for errors
   - Ensure `wasm-pack build` completed successfully
   - Verify the import path in the JavaScript snippet

2. **Deploy failures**
   - Check Netlify build logs
   - Ensure `netlify.toml` is configured correctly
   - Make sure all dependencies are properly specified

3. **CORS errors when fetching data**
   - Configure proxy redirects in your `netlify.toml`
   - Use Netlify Functions for API calls that require CORS

## 📖 Resources

- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [Netlify Documentation](https://docs.netlify.com/)
- [web-sys API Documentation](https://rustwasm.github.io/wasm-bindgen/api/web_sys/)
