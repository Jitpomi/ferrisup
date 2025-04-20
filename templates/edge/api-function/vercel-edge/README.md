# {{project_name}} - Rust Vercel Edge Function

This project is a Rust-powered Vercel Edge Function built with WebAssembly. It enables you to run high-performance edge functions globally on Vercel's edge network.

## 📋 Features

- ⚡️ Lightning-fast API responses powered by Rust and WebAssembly
- 🌐 Global deployment on Vercel's edge network
- 🧩 URL routing with path and query parameter support
- 🔄 JSON serialization and deserialization
- 💨 Zero cold starts compared to traditional serverless functions

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

3. Set up the API directory structure:
   ```
   mkdir -p api
   cp pkg/{{project_name}}_bg.wasm api/
   cp api-handler.js api/index.js
   ```

4. Start the local development server:
   ```
   vercel dev
   ```

5. Your API will be available at `http://localhost:3000`

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

## 📖 API Documentation

### Available Endpoints

- `GET /` - Returns a simple HTML page
- `GET /api` - Returns a JSON response with a greeting message
- `GET /api/echo?message=your-message` - Echo back the provided message as JSON

### Example Requests

```bash
# Get the default API response
curl -X GET "https://your-deployment-url.vercel.app/api"

# Echo a message
curl -X GET "https://your-deployment-url.vercel.app/api/echo?message=hello-world"
```

## 🔧 Customization

### Adding New Routes

Modify the `handler` function in `src/lib.rs` to add new routes:

```rust
match (method.as_str(), path) {
    // Existing routes...
    
    // Add your new route here
    ("GET", "/api/new-endpoint") => {
        // Your handler code
        let response = ApiResponse {
            message: "Your new endpoint response".to_string(),
            status: "success".to_string(),
            timestamp: js_sys::Date::now() as u64,
            path: Some("/api/new-endpoint".to_string()),
            method: Some("GET".to_string()),
            params: None,
        };
        
        let json = serde_json::to_string(&response).unwrap();
        create_response(&json, "application/json", 200)
    },
    
    // Default 404 handler
    _ => { /* ... */ }
}
```

### Working with Environment Variables

To use environment variables in your Vercel Edge Function:

1. Add environment variables in the Vercel dashboard or through the CLI:
   ```
   vercel env add MY_ENV_VAR
   ```

2. Access environment variables in your JavaScript handler:
   ```javascript
   // api-handler.js
   export default async function(req) {
     console.log(process.env.MY_ENV_VAR);
     return handler(req);
   }
   ```

3. To pass environment variables to your Rust code, you need to pass them from JavaScript to WebAssembly using function parameters or global objects.

### Advanced Features

Vercel Edge Functions support several advanced features:

1. **Edge Middleware**: You can implement middleware logic directly in your handler function.

2. **Edge Config**: You can use Vercel's Edge Config for configuration:
   ```javascript
   import { handler } from '../pkg/{{project_name}}.js';
   import { getConfig } from '@vercel/edge-config';

   export default async function(req) {
     const config = await getConfig();
     // Pass config values to your Rust handler
     return handler(req);
   }
   ```

3. **Edge SQL**: You can use Vercel's Edge SQL to interact with databases directly from the edge.

## 📚 Resources

- [Vercel Edge Functions Documentation](https://vercel.com/docs/functions/edge-functions)
- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
