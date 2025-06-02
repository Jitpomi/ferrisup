use http::{Request, Response, StatusCode};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Server,
};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, net::SocketAddr};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

/// Event input structure for the GCP Cloud Function
#[derive(Deserialize)]
struct RequestBody {
    #[serde(default)]
    name: String,
}

/// Response structure for the GCP Cloud Function
#[derive(Serialize)]
struct ResponseBody {
    message: String,
    request_id: String,
}

async fn function_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
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
    
    let message = format!("Hello, {}! Welcome to your FerrisUp serverless function.", name);
    
    // Return the formatted response
    let response_body = ResponseBody {
        message,
        request_id: Uuid::new_v4().to_string(),
    };
    
    let json = serde_json::to_string(&response_body).unwrap();
    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(json))
        .unwrap();
    
    Ok(response)
}

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    
    info!("FerrisUp GCP Cloud Function starting");
    
    // Get the port from the environment or use a default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    // Create a service to handle each incoming connection
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(function_handler)) }
    });
    
    // Create and run the server
    let server = Server::bind(&addr).serve(make_svc);
    
    info!("Server listening on {}", addr);
    
    // Run the server
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
