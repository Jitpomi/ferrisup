use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Simple macro for logging to the browser console
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

#[wasm_bindgen]
pub struct APP_NAME {
    name: String,
}

#[wasm_bindgen]
impl APP_NAME {
    /// Create a new instance of the APP_NAME
    pub fn new(name: &str) -> Self {
        console_log!("Creating new APP_NAME instance: {}", name);
        Self {
            name: name.to_string(),
        }
    }

    /// Get the name of the app
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Process some data with the edge app
    pub fn process(&self, data: &str) -> String {
        console_log!("Processing data with {}: {}", self.name, data);
        format!("Processed by {}: {}", self.name, data)
    }
}

// This is like the "main" function for our WASM module
#[wasm_bindgen(start)]
pub fn start() {
    // Print a message to the browser console
    console_log!("APP_NAME edge application initialized");
    
    // In a real app, you might set up event listeners or initialize resources here
}

// Regular Rust code (not exported to JavaScript)
mod utils {
    /// An internal helper function (not exposed to JavaScript)
    pub fn transform_data(input: &str) -> String {
        format!("Transformed: {}", input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = APP_NAME::new("test-app");
        assert_eq!(app.name(), "test-app");
    }

    #[test]
    fn test_processing() {
        let app = APP_NAME::new("test-app");
        let result = app.process("test data");
        assert!(result.contains("test data"));
    }
}
