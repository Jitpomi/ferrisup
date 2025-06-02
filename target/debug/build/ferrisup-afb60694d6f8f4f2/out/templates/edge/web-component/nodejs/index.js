// JavaScript wrapper for the Rust WebAssembly component
const wasmModule = require('./pkg/{{project_name}}.js');

// Export the raw WebAssembly interface
exports.raw = wasmModule;

// Helper function to initialize the WebAssembly module
exports.initialize = async function() {
  await wasmModule.default();
  console.log('Rust WebAssembly component initialized successfully');
  return exports;
};

// Create a new component with the given options
exports.createComponent = function(options = {}) {
  return new wasmModule.NodejsComponent(options);
};

// Create a single data item
exports.createDataItem = function(id, value, tags) {
  return wasmModule.create_data_item(id, value, tags);
};

// Generate batch test data
exports.createBatchData = function(count, baseValue, tag) {
  return wasmModule.create_batch_data(count, baseValue, tag);
};

// Example usage function
exports.runExample = async function() {
  // Initialize the module
  await exports.initialize();
  
  // Create a component with debugging enabled
  const component = exports.createComponent({ 
    name: "ExampleComponent", 
    debug: true,
    cache_size: 50 
  });
  
  // Log component info
  console.log('Component info:', component.info());
  
  // Create some test data
  const batchData = exports.createBatchData(10, 5.0, "example");
  
  // Add data to the component
  const addedCount = component.add_data(batchData);
  console.log(`Added ${addedCount} items to the component`);
  
  // Process the data
  const result = component.process_data();
  console.log('Processing result:', result);
  
  // Filter data by tag
  const filtered = component.filter_by_tag("example");
  console.log(`Found ${filtered.length} items with tag "example"`);
  
  // Clear the component
  const clearedCount = component.clear();
  console.log(`Cleared ${clearedCount} items from the component`);
  
  return 'Example completed successfully';
};

// Run the example if this file is executed directly
if (require.main === module) {
  exports.runExample()
    .then(result => console.log(result))
    .catch(err => console.error('Error:', err));
}
