use fastly::http::{Method, StatusCode};
use fastly::{Error, Request, Response};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use fastly::http::header::CONTENT_TYPE;

/// API Response structure for JSON responses
#[derive(Serialize, Deserialize)]
struct ApiResponse {
    message: String,
    status: String,
    timestamp: u64,
    path: Option<String>,
    method: Option<String>,
}

/// Error response structure
#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
    status: u16,
}

/// The main entry point for your application.
///
/// This function is triggered when your service receives a client_old request.
/// It could be used for routing, or as a template for implementation logic.
#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    // Log the request details for debugging
    println!(
        "Handling request: {} {}",
        req.get_method(),
        req.get_url().path()
    );

    // Get the request method and path
    let method = req.get_method();
    let path = req.get_url().path();

    // Route the request based on method and path
    match (method, path) {
        // Root path handler - returns HTML page
        (&Method::GET, "/") => {
            let html = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>{{project_name}} | Fastly Compute@Edge</title>
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
                    h1 { color: #FF282D; margin-top: 0; }
                    pre {
                        background: #f0f0f0;
                        padding: 1rem;
                        border-radius: 4px;
                        overflow-x: auto;
                    }
                    a { color: #FF282D; }
                </style>
            </head>
            <body>
                <div class='container'>
                    <h1>🦀 {{project_name}} is running!</h1>
                    <p>Your Rust-powered Fastly Compute@Edge application is successfully running.</p>
                    <h2>API Endpoints:</h2>
                    <ul>
                        <li><code>/api</code> - Returns a JSON response</li>
                        <li><code>/api/echo?message=hello</code> - Echoes back your message</li>
                        <li><code>/api/cache</code> - Demonstrates cache control</li>
                    </ul>
                    <h2>Try it out:</h2>
                    <pre>curl -X GET "https://your-service.edgecompute.app/api"</pre>
                </div>
            </body>
            </html>
            "#;

            Ok(Response::from_body(html)
                .with_status(StatusCode::OK)
                .with_header(CONTENT_TYPE, "text/html"))
        }

        // API endpoint - returns JSON
        (&Method::GET, "/api") => {
            let response = ApiResponse {
                message: "Hello from Rust on Fastly Compute@Edge!".to_string(),
                status: "success".to_string(),
                timestamp: current_timestamp(),
                path: Some("/api".to_string()),
                method: Some("GET".to_string()),
            };

            Ok(Response::from_body(serde_json::to_string(&response)?)
                .with_status(StatusCode::OK)
                .with_header(CONTENT_TYPE, "application/json"))
        }

        // Echo API - returns message from query param
        (&Method::GET, path) if path.starts_with("/api/echo") => {
            // Parse the query parameters
            let params = req.get_url().query_pairs();
            
            // Look for a message parameter
            let message = params
                .into_iter()
                .find(|(key, _)| key == "message")
                .map(|(_, value)| value.to_string())
                .unwrap_or_else(|| "No message provided".to_string());

            let response = ApiResponse {
                message: format!("Echo: {}", message),
                status: "success".to_string(),
                timestamp: current_timestamp(),
                path: Some(path.to_string()),
                method: Some("GET".to_string()),
            };

            Ok(Response::from_body(serde_json::to_string(&response)?)
                .with_status(StatusCode::OK)
                .with_header(CONTENT_TYPE, "application/json"))
        }

        // Cache control example
        (&Method::GET, "/api/cache") => {
            let response = ApiResponse {
                message: "This response is cached for 60 seconds".to_string(),
                status: "success".to_string(),
                timestamp: current_timestamp(),
                path: Some("/api/cache".to_string()),
                method: Some("GET".to_string()),
            };

            Ok(Response::from_body(serde_json::to_string(&response)?)
                .with_status(StatusCode::OK)
                .with_header(CONTENT_TYPE, "application/json")
                .with_header("Cache-Control", "public, max-age=60")
                .with_header("Surrogate-Control", "max-age=60"))
        }

        // Catch all other routes with a 404
        _ => {
            let error = ErrorResponse {
                error: "Not Found".to_string(),
                status: 404,
            };

            Ok(Response::from_body(serde_json::to_string(&error)?)
                .with_status(StatusCode::NOT_FOUND)
                .with_header(CONTENT_TYPE, "application/json"))
        }
    }
}

/// Helper function to get the current timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
