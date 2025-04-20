use wasm_bindgen::prelude::*;
use js_sys::{Array, Function, Object, Reflect};
use serde::{Serialize, Deserialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

// A macro to provide `println!(..)` style syntax for `console.log` logging.
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

// Error macro for console.error
macro_rules! console_error {
    ($($t:tt)*) => (error(&format!($($t)*)))
}

// Configuration options for the component
#[derive(Serialize, Deserialize)]
pub struct ComponentOptions {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub cache_size: usize,
}

impl Default for ComponentOptions {
    fn default() -> Self {
        Self {
            name: "RustComponent".to_string(),
            version: "1.0.0".to_string(),
            debug: false,
            cache_size: 100,
        }
    }
}

// Data structure for processing in the component
#[derive(Serialize, Deserialize)]
pub struct DataItem {
    pub id: String,
    pub value: f64,
    pub tags: Vec<String>,
}

// Result structure for component operations
#[derive(Serialize, Deserialize)]
pub struct ProcessingResult {
    pub success: bool,
    pub count: usize,
    pub total: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub processed_at: u64,
}

// The main component class for Node.js
#[wasm_bindgen]
pub struct NodejsComponent {
    options: ComponentOptions,
    cache: Vec<DataItem>,
    callback: Option<Function>,
}

#[wasm_bindgen]
impl NodejsComponent {
    #[wasm_bindgen(constructor)]
    pub fn new(opts: JsValue) -> Result<NodejsComponent, JsValue> {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();
        
        // Parse options or use defaults
        let options: ComponentOptions = match opts.is_undefined() || opts.is_null() {
            true => ComponentOptions::default(),
            false => match serde_wasm_bindgen::from_value(opts) {
                Ok(parsed) => parsed,
                Err(err) => {
                    console_error!("Failed to parse options: {}", err);
                    ComponentOptions::default()
                }
            }
        };
        
        // Log initialization in debug mode
        if options.debug {
            console_log!("Initializing Rust NodejsComponent with options: {:?}", 
                serde_json::to_string(&options).unwrap_or_default());
        }
        
        // Create and return the component instance
        Ok(NodejsComponent {
            options,
            cache: Vec::with_capacity(options.cache_size),
            callback: None,
        })
    }
    
    // Get component information
    #[wasm_bindgen]
    pub fn info(&self) -> JsValue {
        let info_obj = Object::new();
        
        Reflect::set(&info_obj, &"name".into(), &self.options.name.into()).unwrap();
        Reflect::set(&info_obj, &"version".into(), &self.options.version.into()).unwrap();
        Reflect::set(&info_obj, &"debug".into(), &self.options.debug.into()).unwrap();
        Reflect::set(&info_obj, &"cache_size".into(), &self.options.cache_size.into()).unwrap();
        Reflect::set(&info_obj, &"current_items".into(), &self.cache.len().into()).unwrap();
        Reflect::set(&info_obj, &"engine".into(), &"Rust WebAssembly".into()).unwrap();
        
        info_obj.into()
    }
    
    // Add data items to the component cache
    #[wasm_bindgen]
    pub fn add_data(&mut self, data: JsValue) -> Result<usize, JsValue> {
        // Parse the incoming data
        let items: Vec<DataItem> = match serde_wasm_bindgen::from_value(data) {
            Ok(parsed) => parsed,
            Err(err) => {
                let error_msg = format!("Failed to parse data: {}", err);
                return Err(JsValue::from_str(&error_msg));
            }
        };
        
        // Debug log
        if self.options.debug {
            console_log!("Adding {} items to the component", items.len());
        }
        
        // Add items to the cache, respecting the cache size limit
        let available_space = self.options.cache_size - self.cache.len();
        let items_to_add = std::cmp::min(items.len(), available_space);
        
        // Add items to the cache
        for i in 0..items_to_add {
            self.cache.push(items[i].clone());
        }
        
        // Trigger callback if defined
        if let Some(callback) = &self.callback {
            if let Ok(count_val) = JsValue::from_serde(&items_to_add) {
                let _ = callback.call1(&JsValue::NULL, &count_val);
            }
        }
        
        Ok(items_to_add)
    }
    
    // Process the data in the cache and return statistics
    #[wasm_bindgen]
    pub fn process_data(&self) -> Result<JsValue, JsValue> {
        // Check if we have data to process
        if self.cache.is_empty() {
            return Err(JsValue::from_str("No data to process"));
        }
        
        // Debug log
        if self.options.debug {
            console_log!("Processing {} items in the component", self.cache.len());
        }
        
        // Calculate statistics
        let count = self.cache.len();
        let mut total = 0.0;
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        
        for item in &self.cache {
            total += item.value;
            min = min.min(item.value);
            max = max.max(item.value);
        }
        
        let average = if count > 0 { total / count as f64 } else { 0.0 };
        
        // Create the result
        let result = ProcessingResult {
            success: true,
            count,
            total,
            average,
            min,
            max,
            processed_at: js_sys::Date::now() as u64,
        };
        
        // Convert to JsValue and return
        match serde_wasm_bindgen::to_value(&result) {
            Ok(js_result) => Ok(js_result),
            Err(err) => Err(JsValue::from_str(&format!("Failed to serialize result: {}", err))),
        }
    }
    
    // Filter data by tag
    #[wasm_bindgen]
    pub fn filter_by_tag(&self, tag: String) -> Result<JsValue, JsValue> {
        // Debug log
        if self.options.debug {
            console_log!("Filtering items by tag: {}", tag);
        }
        
        // Filter items
        let filtered: Vec<&DataItem> = self.cache.iter()
            .filter(|item| item.tags.contains(&tag))
            .collect();
        
        // Convert to JsValue and return
        match serde_wasm_bindgen::to_value(&filtered) {
            Ok(js_result) => Ok(js_result),
            Err(err) => Err(JsValue::from_str(&format!("Failed to serialize filtered data: {}", err))),
        }
    }
    
    // Clear the component cache
    #[wasm_bindgen]
    pub fn clear(&mut self) -> usize {
        let count = self.cache.len();
        self.cache.clear();
        
        // Debug log
        if self.options.debug {
            console_log!("Cleared {} items from the component", count);
        }
        
        count
    }
    
    // Set a callback function to be called when data is added
    #[wasm_bindgen]
    pub fn set_callback(&mut self, callback: Function) {
        self.callback = Some(callback);
        
        // Debug log
        if self.options.debug {
            console_log!("Callback function set");
        }
    }
    
    // Remove the callback function
    #[wasm_bindgen]
    pub fn remove_callback(&mut self) {
        self.callback = None;
        
        // Debug log
        if self.options.debug {
            console_log!("Callback function removed");
        }
    }
}

// Utility functions that don't require a component instance
#[wasm_bindgen]
pub fn create_data_item(id: String, value: f64, tags: Box<[JsValue]>) -> Result<JsValue, JsValue> {
    // Convert JsValue tags to Rust strings
    let mut rust_tags = Vec::with_capacity(tags.len());
    for tag in tags.iter() {
        match tag.as_string() {
            Some(t) => rust_tags.push(t),
            None => return Err(JsValue::from_str("Tags must be strings")),
        }
    }
    
    // Create the data item
    let item = DataItem {
        id,
        value,
        tags: rust_tags,
    };
    
    // Convert to JsValue and return
    match serde_wasm_bindgen::to_value(&item) {
        Ok(js_item) => Ok(js_item),
        Err(err) => Err(JsValue::from_str(&format!("Failed to serialize data item: {}", err))),
    }
}

// Batch create data items
#[wasm_bindgen]
pub fn create_batch_data(count: usize, base_value: f64, tag: String) -> Result<JsValue, JsValue> {
    let mut items = Vec::with_capacity(count);
    
    for i in 0..count {
        items.push(DataItem {
            id: format!("item-{}", i),
            value: base_value + (i as f64),
            tags: vec![tag.clone()],
        });
    }
    
    // Convert to JsValue and return
    match serde_wasm_bindgen::to_value(&items) {
        Ok(js_items) => Ok(js_items),
        Err(err) => Err(JsValue::from_str(&format!("Failed to serialize batch data: {}", err))),
    }
}
