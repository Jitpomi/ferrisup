use wasm_bindgen::prelude::*;
use web_sys::{Request, Response, ResponseInit, Headers};
use wasm_bindgen_futures::future_to_promise;
use serde::{Serialize, Deserialize};
use js_sys::{Promise, Object, Reflect, JSON};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // Define a binding to fetch Request properties
    type RequestInfo;
    
    #[wasm_bindgen(method, getter)]
    fn url(this: &RequestInfo) -> String;
    
    #[wasm_bindgen(method, getter)]
    fn method(this: &RequestInfo) -> String;
    
    #[wasm_bindgen(method, getter)]
    fn headers(this: &RequestInfo) -> Headers;
}

// Custom error type for API responses
#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
    status: u16,
}

// Success response for API
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    message: String,
    status: String,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

// Main handler function for Vercel Edge Functions
#[wasm_bindgen]
pub fn handler(req: JsValue) -> Promise {
    // Use console_error_panic_hook for better debugging
    console_error_panic_hook::set_once();
    
    // Convert the JsValue to a web_sys::Request
    let request = req.dyn_into::<RequestInfo>().unwrap();
    let url = request.url();
    let method = request.method();
    
    // Extract the path from the URL
    let path = url.split('?').next().unwrap_or("/");
    
    // Create a Future that handles the request
    future_to_promise(async move {
        match (method.as_str(), path) {
            // Root path handler
            ("GET", "/") => {
                let html = r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>{{project_name}} | Vercel Edge Function</title>
                    <style>
                        body {
                            font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            margin: 0;
                            padding: 2rem;
                            color: #333;
                            background: #f5f5f5;
                        }
                        .container {
                            max-width: 800px;
                            margin: 0 auto;
                            background: white;
                            padding: 2rem;
                            border-radius: 8px;
                            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                        }
                        h1 { color: #FF0080; margin-top: 0; }
                        pre {
                            background: #f0f0f0;
                            padding: 1rem;
                            border-radius: 4px;
                            overflow-x: auto;
                        }
                        a { color: #FF0080; }
                    </style>
                </head>
                <body>
                    <div class='container'>
                        <h1>🦀 {{project_name}} is running!</h1>
                        <p>Your Rust-powered Vercel Edge Function is successfully running.</p>
                        <h2>API Endpoints:</h2>
                        <ul>
                            <li><code>/api</code> - Returns a JSON response</li>
                            <li><code>/api/echo?message=hello</code> - Echoes back your message</li>
                        </ul>
                        <h2>Try it out:</h2>
                        <pre>curl -X GET &quot;https://your-deployment-url.vercel.app/api&quot;</pre>
                    </div>
                </body>
                </html>
                "#;
                
                create_response(html, "text/html", 200)
            },
            
            // API path handler
            ("GET", "/api") => {
                let response = ApiResponse {
                    message: "Hello from Rust on Vercel Edge Functions!".to_string(),
                    status: "success".to_string(),
                    timestamp: js_sys::Date::now() as u64,
                    path: Some("/api".to_string()),
                    method: Some("GET".to_string()),
                    params: None,
                };
                
                let json = serde_json::to_string(&response).unwrap();
                create_response(&json, "application/json", 200)
            },
            
            // Echo API handler
            ("GET", path) if path.starts_with("/api/echo") => {
                // Parse URL to extract query parameters
                let query = url.split('?').nth(1).unwrap_or("");
                let params = parse_query_string(query);
                
                let message = params.get("message")
                    .cloned()
                    .unwrap_or_else(|| "No message provided".to_string());
                
                let response = ApiResponse {
                    message: format!("Echo: {}", message),
                    status: "success".to_string(),
                    timestamp: js_sys::Date::now() as u64,
                    path: Some(path.to_string()),
                    method: Some("GET".to_string()),
                    params: Some(serde_json::to_value(params).unwrap()),
                };
                
                let json = serde_json::to_string(&response).unwrap();
                create_response(&json, "application/json", 200)
            },
            
            // 404 Not Found handler
            _ => {
                let error = ErrorResponse {
                    error: "Not Found".to_string(),
                    status: 404,
                };
                
                let json = serde_json::to_string(&error).unwrap();
                create_response(&json, "application/json", 404)
            }
        }
    })
}

// Helper function to create a Response object
fn create_response(body: &str, content_type: &str, status: u16) -> Result<JsValue, JsValue> {
    let mut init = ResponseInit::new();
    init.status(status);
    
    let headers = Headers::new()?;
    headers.set("Content-Type", content_type)?;
    headers.set("X-Powered-By", "Rust + WebAssembly")?;
    init.headers(&headers);
    
    let response = Response::new_with_opt_str_and_init(Some(body), &init)?;
    Ok(response.into())
}

// Helper function to parse URL query parameters
fn parse_query_string(query: &str) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    
    if query.is_empty() {
        return params;
    }
    
    for param in query.split('&') {
        if let Some(index) = param.find('=') {
            let key = &param[..index];
            let value = &param[(index + 1)..];
            params.insert(
                urlencoding::decode(key).unwrap_or_default().to_string(),
                urlencoding::decode(value).unwrap_or_default().to_string()
            );
        }
    }
    
    params
}
