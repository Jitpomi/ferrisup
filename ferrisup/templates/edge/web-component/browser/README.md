# {{project_name}} - Rust WebAssembly Component

This project is a reusable web component built with Rust and WebAssembly, designed for browsers. It provides a foundation for building interactive UI components that can be embedded in any web application.

## 📋 Features

- ⚡️ High-performance WebAssembly compiled from Rust
- 🎨 Customizable properties and styling
- 🔒 Encapsulated with Shadow DOM for style isolation
- 🔄 Interactive with JavaScript interoperability
- 📦 Easily embeddable in any web project

## 🚀 Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Development

1. Build the WebAssembly binary:
   ```
   wasm-pack build --target web
   ```

2. Serve the project (using any static file server):
   ```
   # If you have Python installed
   python -m http.server
   
   # Or using Node.js
   npx serve
   ```

3. Open your browser and navigate to the local server (typically http://localhost:8000)

### Integration

To use this component in other projects:

1. Build the WebAssembly binary:
   ```
   wasm-pack build --target bundler
   ```

2. Copy the generated `pkg` directory to your project

3. Import and use the component:
   ```javascript
   import init, { RustComponent, ComponentProperties } from './pkg/{{project_name}}.js';
   
   async function initComponent() {
     await init();
     
     // Create a new component instance
     const component = new RustComponent();
     
     // Customize properties
     const props = new ComponentProperties();
     props.set_title("My Custom Title");
     props.set_description("Custom description here");
     props.set_theme("dark");
     
     // Update the component with new properties
     component.update(props);
     
     // Add a click handler
     component.set_click_handler(() => {
       console.log("Component clicked!");
     });
     
     // Add the component to the DOM
     document.getElementById("container").appendChild(component.element());
   }
   
   initComponent();
   ```

## 🔧 Customization

### Component Properties

The component supports the following properties:

- **title**: The main heading of the component
- **description**: Descriptive text displayed in the component
- **theme**: Visual theme ('light' or 'dark')

### Custom Styling

You can customize the component's appearance by modifying the CSS in the `create_style` method in `src/lib.rs`.

### Adding New Features

To add new features to the component:

1. Add new properties to the `ComponentProperties` struct in `src/lib.rs`
2. Implement getters and setters for the new properties
3. Update the `render` method to use the new properties
4. Rebuild with `wasm-pack build --target web`

## 📚 Advanced Usage

### Registering as a Custom Element

The component includes a helper function to register it as a custom element:

```javascript
import init, { register_rust_component } from './pkg/{{project_name}}.js';

async function registerComponent() {
  await init();
  
  // Register as <rust-component>
  register_rust_component("rust-component");
  
  // Now you can use it in HTML
  // <rust-component></rust-component>
}

registerComponent();
```

### Framework Integration

This component can be integrated with popular frameworks:

#### React

```jsx
import React, { useEffect, useRef } from 'react';
import init, { RustComponent } from './pkg/{{project_name}}.js';

function RustComponentWrapper(props) {
  const containerRef = useRef(null);
  const componentRef = useRef(null);
  
  useEffect(() => {
    let mounted = true;
    
    async function initializeComponent() {
      await init();
      if (!mounted) return;
      
      componentRef.current = new RustComponent();
      containerRef.current.appendChild(componentRef.current.element());
    }
    
    initializeComponent();
    
    return () => {
      mounted = false;
    };
  }, []);
  
  return <div ref={containerRef}></div>;
}

export default RustComponentWrapper;
```

## 📖 Resources

- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [Web Components Introduction](https://developer.mozilla.org/en-US/docs/Web/Web_Components)
