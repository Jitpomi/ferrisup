# {{project_name}} - Rust WebAssembly Component for Node.js

This project is a high-performance Rust WebAssembly component specifically designed for Node.js environments. It provides an optimized data processing library that can be seamlessly integrated into Node.js applications.

## 📋 Features

- ⚡️ High-performance data processing powered by Rust and WebAssembly
- 🔄 Bidirectional data exchange between JavaScript and Rust
- 🧠 Efficient in-memory cache with configurable size
- 🔍 Data filtering and aggregation capabilities
- 📊 Statistical analysis of numerical data
- 🧩 Callback support for reactive programming patterns

## 🚀 Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (v14 or later recommended)
- npm or yarn

### Installation

1. Build the WebAssembly module:
   ```
   npm run build
   ```
   This will compile the Rust code to WebAssembly and generate the necessary JavaScript bindings in the `pkg` directory.

2. Use in your project:
   ```javascript
   const component = require('./');
   
   async function main() {
     await component.initialize();
     
     // Create a component instance
     const processor = component.createComponent({
       name: "MyProcessor",
       debug: true
     });
     
     console.log(processor.info());
   }
   
   main().catch(console.error);
   ```

### Running the Example

The package includes a simple example that demonstrates the component's capabilities:

```
node index.js
```

This will run a demonstration that creates a component, adds data, processes it, and displays the results.

## 📖 API Documentation

### Component Creation

```javascript
// Initialize the module
await component.initialize();

// Create a component with options
const processor = component.createComponent({
  name: "ProcessorName",        // Component name
  version: "1.0.0",             // Version string
  debug: true,                  // Enable debug logging
  cache_size: 100               // Maximum items in cache
});
```

### Working with Data

```javascript
// Create a single data item
const item = component.createDataItem("item-1", 42.5, ["tag1", "tag2"]);

// Generate batch test data (count, baseValue, commonTag)
const batchData = component.createBatchData(10, 5.0, "example");

// Add data to the component
const addedCount = processor.add_data(batchData);

// Process data and get statistics
const stats = processor.process_data();
// Returns: { success, count, total, average, min, max, processed_at }

// Filter data by tag
const filtered = processor.filter_by_tag("example");

// Clear the component cache
const clearedCount = processor.clear();
```

### Working with Callbacks

```javascript
// Set a callback function for add_data operations
processor.set_callback(function(count) {
  console.log(`Added ${count} items to the component`);
});

// Remove the callback
processor.remove_callback();
```

### Component Information

```javascript
// Get component information
const info = processor.info();
// Returns: { name, version, debug, cache_size, current_items, engine }
```

## 🔧 Customization

### Modifying the Rust Code

The core functionality is implemented in `src/lib.rs`. You can extend this in several ways:

1. **Add new data processing methods** to the `NodejsComponent` implementation:
   ```rust
   #[wasm_bindgen]
   pub fn calculate_moving_average(&self, window_size: usize) -> Result<JsValue, JsValue> {
       // Implementation here
   }
   ```

2. **Add new utility functions** outside the component:
   ```rust
   #[wasm_bindgen]
   pub fn transform_data(data: JsValue, factor: f64) -> Result<JsValue, JsValue> {
       // Implementation here
   }
   ```

3. **Extend the data structures** to support additional fields or processing requirements.

### Optimizing for Performance

For maximum performance:

1. Build in release mode:
   ```
   wasm-pack build --target nodejs --release
   ```

2. Consider adjusting the memory allocator settings or turning off `wee_alloc` for better performance (at the cost of slightly larger binary size).

3. Process data in batches rather than individual items for best throughput.

## 📊 Performance Considerations

### Memory Management

WebAssembly operates with a linear memory model. The component includes:

- Configurable cache size to limit memory usage
- Explicit `clear()` method to free memory when needed

### Data Transfer Overhead

When passing large data structures between JavaScript and Rust, there is serialization overhead. To minimize this:

1. Process data in batches rather than individual calls
2. Keep data in the WebAssembly module as long as possible
3. Only return the results you need, not entire datasets

## 🧪 Testing

The project includes a basic test script:

```
npm test
```

To write additional tests, consider using Jest with the provided WebAssembly module:

```javascript
const component = require('./');

beforeAll(async () => {
  await component.initialize();
});

test('should process data correctly', async () => {
  const processor = component.createComponent();
  const data = component.createBatchData(10, 1.0, "test");
  processor.add_data(data);
  
  const result = processor.process_data();
  expect(result.success).toBe(true);
  expect(result.count).toBe(10);
  expect(result.average).toBe(5.5); // Average of 1..10
});
```

## 📚 Resources

- [Rust and WebAssembly](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [Node.js WebAssembly Documentation](https://nodejs.org/api/wasm.html)
- [Serde for JavaScript Interop](https://docs.rs/serde-wasm-bindgen/)
