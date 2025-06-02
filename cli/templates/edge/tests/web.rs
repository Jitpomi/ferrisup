//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use {{project_name}}::{{project_name_pascal_case}};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_app_creation() {
    // Create a new instance of our app
    let app = {{project_name_pascal_case}}::new("test-app");
    assert_eq!(app.name(), "test-app");
}

#[wasm_bindgen_test]
fn test_processing() {
    // Create a new instance of our app
    let app = {{project_name_pascal_case}}::new("test-app");
    
    // Process some test data
    let result = app.process("test data");
    
    // Verify the result contains our test data
    assert!(result.contains("test data"));
}

#[wasm_bindgen_test]
fn test_console_log() {
    // This test just demonstrates that console.log works
    web_sys::console::log_1(&"Testing console.log from WebAssembly".into());
    
    // No assertion needed, this is just to verify the console.log binding works
    assert!(true);
}
