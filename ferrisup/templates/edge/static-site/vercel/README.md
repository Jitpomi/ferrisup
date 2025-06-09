# {{project_name}} - Rust WebAssembly Static Site for Vercel

This project is a static website built with Rust and WebAssembly, designed to be deployed on Vercel. It provides a foundation for building interactive web applications with native-like performance.

## 📋 Features

- ⚡️ High-performance WebAssembly compiled from Rust
- 🎨 Modern, responsive design with fluid layouts
- 🌐 Ready for deployment on Vercel's global edge network
- 🧩 Component-based architecture for easy customization
- 🔄 Interactive UI elements powered by Rust

## 🚀 Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (for Vercel CLI)
- [Vercel CLI](https://vercel.com/docs/cli) (optional for local development)

### Development

1. Install the Vercel CLI if you haven't already:
   ```
   npm install -g vercel
   ```

2. Build the WebAssembly binary:
   ```
   wasm-pack build --target web
   ```

3. Start a local server for development:
   ```
   vercel dev
   ```
   
   Alternatively, you can use any static file server:
   ```
   # If you have Python installed
   python -m http.server
   
   # If you have Node.js installed
   npx serve
   ```

4. Your site will be available at the provided local URL (typically http://localhost:3000 or http://localhost:8000)

### Deployment

1. Login to your Vercel account (if not already logged in):
   ```
   vercel login
   ```

2. Deploy to Vercel:
   ```
   vercel
   ```

3. For production deployment:
   ```
   vercel --prod
   ```

## 🔧 Customization

### Project Structure

```
{{project_name}}/
├── src/
│   └── lib.rs           # Main Rust code for WebAssembly
├── index.html           # Main HTML file
├── Cargo.toml           # Rust dependencies
└── vercel.json          # Vercel configuration
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
- Header styles
- Hero section
- Features section
- Demo section
- Footer styles
- Responsive styles

You can modify these styles to match your branding and design preferences.

## 📚 Advanced Features

### Adding Pages

To add more pages to your static site:

1. Create a new HTML file in the root directory
2. Copy the basic structure from `index.html`
3. Add content specific to the new page
4. Update navigation links in `create_header()` function

### Using External Libraries

You can add more Rust crates to enhance your project:

1. Add the dependency to `Cargo.toml`:
   ```toml
   [dependencies]
   # Existing dependencies...
   yew = "0.20"  # Example: Adding Yew framework
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

1. Avoid excessive DOM manipulations
2. Batch updates to minimize reflows and repaints
3. Use `web-sys` directly for DOM operations when possible

## 🛠️ Troubleshooting

### Common Issues

1. **WebAssembly not loading**
   - Check browser console for errors
   - Ensure `wasm-pack build` completed successfully
   - Verify the import path in the JavaScript snippet

2. **Styling issues**
   - Use browser developer tools to inspect elements
   - Check for CSS specificity conflicts

3. **Deployment errors**
   - Check Vercel build logs
   - Ensure `vercel.json` is properly configured

## 📖 Resources

- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [Vercel Documentation](https://vercel.com/docs)
- [web-sys API Documentation](https://rustwasm.github.io/wasm-bindgen/api/web_sys/)
