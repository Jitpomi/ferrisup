# Rust Edge Computing Applications

This template provides a foundation for building Rust applications that run at the edge using WebAssembly. It's designed to help you quickly get started with edge computing platforms like Cloudflare Workers, Vercel Edge Functions, Fastly Compute@Edge, AWS Lambda@Edge, and Netlify Edge Functions.

## Features

- **Multi-platform support**: Build once, deploy anywhere with platform-specific optimizations
- **Three application types**: Static Sites, API/Functions, and Web Components
- **Provider-specific templates**: Optimized for Cloudflare, Vercel, Fastly, AWS, and Netlify
- **Seamless WebAssembly integration**: Rust compiled to high-performance WebAssembly
- **Modern, responsive designs**: For static site templates
- **Framework adapters**: For web components (React, Vue, Svelte)
- **Production-ready configuration**: With proper caching and security settings

## Application Types

### Static Sites

Static site templates provide a foundation for building modern, responsive websites using Rust and WebAssembly. They include:

- Interactive UI elements powered by Rust/WebAssembly
- Responsive layouts with mobile-first design
- Optimized for deployment on global CDN networks

Supported providers:
- **Cloudflare Pages**: Global CDN with auto-minification and performance optimization
- **Vercel**: Edge network with automatic preview deployments and analytics
- **Netlify**: Complete hosting solution with form handling and serverless functions

### API Functions

API function templates enable you to build high-performance serverless APIs using Rust. They include:

- Request routing with path and method matching
- JSON serialization/deserialization
- Error handling and proper HTTP status codes
- Platform-specific optimizations

Supported providers:
- **Cloudflare Workers**: Globally distributed serverless functions with minimal cold starts
- **Vercel Edge Functions**: Serverless functions that execute at the edge
- **Fastly Compute@Edge**: Ultra-low latency edge computing platform
- **AWS Lambda@Edge**: Functions that run alongside CloudFront CDN

### Web Components

Web component templates allow you to build reusable UI components in Rust that can be embedded in any web application. They include:

- Shadow DOM encapsulation for style isolation
- Custom element lifecycle hooks
- State management and event handling
- Framework adapters for popular JavaScript frameworks

Supported environments:
- **Browser**: Pure WebAssembly components using the Web Components standard
- **Node.js**: Server-side rendering and module bundling
- **Framework-specific**: Adapters for React, Vue, and Svelte

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

3. Follow the provider-specific instructions in the README.md file of your generated project:
   - For Cloudflare Workers: Deploy using Wrangler CLI
   - For Vercel: Deploy using Vercel CLI or GitHub integration
   - For Fastly: Deploy using Fastly CLI
   - For AWS Lambda@Edge: Deploy using AWS SAM CLI
   - For Netlify: Deploy using Netlify CLI or GitHub integration

## Customization

Each template includes detailed customization instructions in its README.md file, covering:

- Project structure and organization
- Available API endpoints (for API functions)
- Component props and state (for web components)
- Styling and layout (for static sites)
- Deployment configuration
- Environment variables

## Resources

- [Rust WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [Cloudflare Workers Documentation](https://developers.cloudflare.com/workers/)
- [Vercel Edge Functions Documentation](https://vercel.com/docs/functions/edge-functions)
- [Fastly Compute@Edge Documentation](https://developer.fastly.com/learning/compute/)
- [AWS Lambda@Edge Documentation](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/lambda-at-the-edge.html)
- [Netlify Edge Functions Documentation](https://docs.netlify.com/edge-functions/overview/)
- [Web Components Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Web_components)
