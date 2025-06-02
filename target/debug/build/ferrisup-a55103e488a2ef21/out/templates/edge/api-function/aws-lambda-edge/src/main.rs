use lambda_http::{service_fn, Body, Error, Request, Response, http::StatusCode};
use lambda_runtime::{tracing, service_fn as lambda_service_fn};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// Configure the lambda tracing
tracing::init_default_subscriber();

/// API Response structure for JSON responses
#[derive(Serialize, Deserialize)]
struct ApiResponse {
    message: String,
    status: String,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_parameters: Option<serde_json::Value>,
}

/// Error response structure
#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
    status: u16,
}

/// The main Lambda entry point function for Lambda@Edge
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Start the Lambda Runtime
    lambda_http::run(lambda_service_fn(handler)).await?;
    Ok(())
}

/// Handler function for Lambda@Edge requests
async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Extract the path and method from the request
    let path = req.uri().path();
    let method = req.method().as_str();
    
    // Log the request details
    tracing::info!("Request received: {} {}", method, path);
    
    // Handle the request based on path and method
    match (method, path) {
        // Root path handler
        ("GET", "/") => {
            let html = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>{{project_name}} | AWS Lambda@Edge</title>
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
                    h1 { color: #FF9900; margin-top: 0; }
                    pre {
                        background: #f0f0f0;
                        padding: 1rem;
                        border-radius: 4px;
                        overflow-x: auto;
                    }
                    a { color: #FF9900; }
                </style>
            </head>
            <body>
                <div class='container'>
                    <h1>🦀 {{project_name}} is running!</h1>
                    <p>Your Rust-powered AWS Lambda@Edge function is successfully running.</p>
                    <h2>API Endpoints:</h2>
                    <ul>
                        <li><code>/api</code> - Returns a JSON response</li>
                        <li><code>/api/echo?message=hello</code> - Echoes back your message</li>
                        <li><code>/api/headers</code> - Returns request headers</li>
                    </ul>
                    <h2>Try it out:</h2>
                    <pre>curl -X GET "https://your-api-gateway-url.amazonaws.com/api"</pre>
                </div>
            </body>
            </html>
            "#;
            
            // Build and return the HTML response
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html")
                .body(Body::from(html))?;
            
            Ok(response)
        },
        
        // API endpoint
        ("GET", "/api") => {
            // Create the API response
            let response_data = ApiResponse {
                message: "Hello from Rust on AWS Lambda@Edge!".to_string(),
                status: "success".to_string(),
                timestamp: current_timestamp(),
                path: Some("/api".to_string()),
                method: Some("GET".to_string()),
                headers: None,
                query_parameters: None,
            };
            
            // Serialize and return the JSON response
            let json = serde_json::to_string(&response_data)?;
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(json))?;
            
            Ok(response)
        },
        
        // Echo endpoint
        ("GET", path) if path.starts_with("/api/echo") => {
            // Extract query parameters
            let query_params = extract_query_params(req.uri().query());
            
            // Get the message parameter or use a default
            let message = query_params.get("message")
                .cloned()
                .unwrap_or_else(|| "No message provided".to_string());
            
            // Create the API response
            let response_data = ApiResponse {
                message: format!("Echo: {}", message),
                status: "success".to_string(),
                timestamp: current_timestamp(),
                path: Some(path.to_string()),
                method: Some("GET".to_string()),
                headers: None,
                query_parameters: Some(serde_json::to_value(query_params)?),
            };
            
            // Serialize and return the JSON response
            let json = serde_json::to_string(&response_data)?;
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(json))?;
            
            Ok(response)
        },
        
        // Headers endpoint
        ("GET", "/api/headers") => {
            // Convert request headers to a map
            let mut headers_map = std::collections::HashMap::new();
            for (name, value) in req.headers() {
                headers_map.insert(
                    name.as_str().to_string(), 
                    value.to_str().unwrap_or("invalid header value").to_string()
                );
            }
            
            // Create the API response
            let response_data = ApiResponse {
                message: "Request headers".to_string(),
                status: "success".to_string(),
                timestamp: current_timestamp(),
                path: Some("/api/headers".to_string()),
                method: Some("GET".to_string()),
                headers: Some(serde_json::to_value(headers_map)?),
                query_parameters: None,
            };
            
            // Serialize and return the JSON response
            let json = serde_json::to_string(&response_data)?;
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(json))?;
            
            Ok(response)
        },
        
        // Not found (404) handler
        _ => {
            // Create the error response
            let error_data = ErrorResponse {
                error: "Not Found".to_string(),
                status: 404,
            };
            
            // Serialize and return the JSON response
            let json = serde_json::to_string(&error_data)?;
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "application/json")
                .body(Body::from(json))?;
            
            Ok(response)
        }
    }
}

/// Helper function to get the current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Helper function to extract query parameters
fn extract_query_params(query: Option<&str>) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some(index) = pair.find('=') {
                let key = &pair[..index];
                let value = &pair[(index + 1)..];
                params.insert(
                    key.to_string(),
                    urlencoding::decode(value).unwrap_or_default().to_string()
                );
            }
        }
    }
    
    params
}
