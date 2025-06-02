use worker::*;
use serde::{Deserialize, Serialize};

// Define your API response data structure
#[derive(Serialize, Deserialize)]
struct ApiResponse {
    message: String,
    status: String,
    timestamp: u64,
}

#[event(fetch)]
pub async fn main(mut req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    // Set up better logging for debugging
    console_log!("Worker started processing request");
    console_error_panic_hook::set_once();
    
    // Get the URL and method for routing
    let url = req.url()?;
    let method = req.method();
    
    match (url.path(), method) {
        ("/", Method::Get) => {
            // Serve the landing page HTML
            let html = get_index_html();
            Response::from_html(html)
        },

        ("/api", Method::Get) => {
            // Simple API response
            let response_data = ApiResponse {
                message: "Hello from Rust and WebAssembly!".to_string(),
                status: "success".to_string(),
                timestamp: Date::now().as_millis() / 1000, // Current time in seconds
            };
            
            Response::from_json(&response_data)
        },
        
        ("/api", Method::Post) => {
            // Echo back JSON data
            let data = req.json().await?;
            Response::from_json(&data)
        },
        
        // Example of accessing KV storage
        ("/api/kv-example", Method::Get) => {
            let kv_example = r#"
// Example of using KV storage:
// 1. Add KV namespace to wrangler.toml:
//    [[kv_namespaces]]
//    binding = "MY_KV"
//    id = "xxxx"
//
// 2. Usage example:
//    let kv = env.kv("MY_KV")?;
//    kv.put("key", "value")?.execute().await?;
//    let value = kv.get("key").text().await?;
"#;
            Response::ok(kv_example)
        },
        
        // Example of accessing env variables
        ("/api/env-example", Method::Get) => {
            let env_example = r#"
// Access environment variables:
// 1. Define in wrangler.toml:
//    [vars]
//    MY_VAR = "example"
//
// 2. Usage example:
//    let my_var = env.var("MY_VAR")?.to_string();
"#;
            Response::ok(env_example)
        },
        
        // Handle 404 for unmatched routes
        _ => {
            let not_found = ApiResponse {
                message: "Not Found".to_string(),
                status: "error".to_string(),
                timestamp: Date::now().as_millis() / 1000,
            };
            
            Ok(Response::from_json(&not_found)?.with_status(404))
        }
    }
}

// Helper function to generate the index HTML page
fn get_index_html() -> String {
    let mut html = String::new();
    
    // Opening tags
    html.push_str("<html>");
    html.push_str("<head>");
    html.push_str("<title>{{project_name}} | Cloudflare Worker</title>");
    
    // CSS styles with proper escaping
    html.push_str("<style>");
    html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; line-height: 1.6; }");
    html.push_str("h1 { color: #f48120; border-bottom: 2px solid #f48120; padding-bottom: 10px; }");
    html.push_str(".powered-by { margin-top: 3rem; font-size: 0.8rem; color: #666; }");
    html.push_str("code { background-color: #f6f8fa; padding: 0.2em 0.4em; border-radius: 3px; }");
    html.push_str(".endpoint { background-color: #f6f8fa; padding: 1rem; border-radius: 5px; margin: 1rem 0; }");
    html.push_str("</style>");
    html.push_str("</head>");
    
    // Page content
    html.push_str("<body>");
    html.push_str("<h1>🦀 {{project_name}} API</h1>");
    html.push_str("<p>Your Rust-powered Cloudflare Worker is up and running!</p>");
    
    html.push_str("<h2>Available Endpoints:</h2>");
    html.push_str("<div class=\"endpoint\">");
    html.push_str("<p><strong>GET /api</strong> - Returns a JSON response</p>");
    html.push_str("<code>curl -X GET https://your-worker.your-subdomain.workers.dev/api</code>");
    html.push_str("</div>");
    
    html.push_str("<div class=\"endpoint\">");
    html.push_str("<p><strong>POST /api</strong> - Accepts JSON data and returns it</p>");
    html.push_str("<code>curl -X POST -H \"Content-Type: application/json\" -d '{\"name\": \"FerrisUp\"}' https://your-worker.your-subdomain.workers.dev/api</code>");
    html.push_str("</div>");
    
    html.push_str("<div class=\"endpoint\">");
    html.push_str("<p><strong>GET /api/kv-example</strong> - Shows KV usage example</p>");
    html.push_str("<code>curl -X GET https://your-worker.your-subdomain.workers.dev/api/kv-example</code>");
    html.push_str("</div>");
    
    html.push_str("<div class=\"endpoint\">");
    html.push_str("<p><strong>GET /api/env-example</strong> - Shows environment variables example</p>");
    html.push_str("<code>curl -X GET https://your-worker.your-subdomain.workers.dev/api/env-example</code>");
    html.push_str("</div>");
    
    html.push_str("<p>Check out your <code>src/lib.rs</code> file to see how this worker is implemented and to add more endpoints.</p>");
    
    html.push_str("<div class=\"powered-by\">");
    html.push_str("Powered by Rust, WebAssembly, and Cloudflare Workers<br>");
    html.push_str("Generated with <a href=\"https://github.com/Jitpomi/ferrisup\">FerrisUp</a>");
    html.push_str("</div>");
    
    html.push_str("</body>");
    html.push_str("</html>");
    
    html
}
