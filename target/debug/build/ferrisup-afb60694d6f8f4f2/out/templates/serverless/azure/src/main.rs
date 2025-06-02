use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

/// Event input structure for the Azure Function
#[derive(Deserialize)]
struct RequestBody {
    #[serde(default)]
    name: String,
}

/// Response structure for the Azure Function
#[derive(Serialize)]
struct ResponseBody {
    message: String,
    request_id: String,
}

/// Handler function for HTTP requests
async fn function_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Generate a unique request ID
    let request_id = Uuid::new_v4().to_string();
    
    info!("Processing request {}", request_id);
    
    // Parse the request body
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
    let request_body: RequestBody = serde_json::from_slice(&body_bytes)
        .unwrap_or(RequestBody { name: "".to_string() });
    
    // Create a response with a greeting using the name from the request or a default
    let name = if request_body.name.is_empty() {
        "World".to_string()
    } else {
        request_body.name
    };
    
    let message = format!("Hello, {}! Welcome to your FerrisUp Azure Function.", name);
    
    // Return the formatted response
    let response_body = ResponseBody {
        message,
        request_id,
    };
    
    let response_json = serde_json::to_string(&response_body).unwrap();
    
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(response_json))
        .unwrap())
}

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    
    info!("FerrisUp Azure Function starting");
    
    // Get the port from the environment variable or use a default
    let port = std::env::var("FUNCTIONS_CUSTOMHANDLER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    // Create a service to handle each incoming connection
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(function_handler)) }
    });
    
    let server = Server::bind(&addr).serve(make_svc);
    
    info!("Listening on http://{}", addr);
    
    // Run the server
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
