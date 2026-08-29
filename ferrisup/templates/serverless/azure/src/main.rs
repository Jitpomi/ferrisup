use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
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
async fn function_handler(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    // Generate a unique request ID
    let request_id = Uuid::new_v4().to_string();
    
    info!("Processing request {}", request_id);
    
    // Parse the request body
    let body_bytes = req
        .into_body()
        .collect()
        .await
        .map(|body| body.to_bytes())
        .unwrap_or_default();
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
        .body(Full::new(Bytes::from(response_json)))
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
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            if let Err(error) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service_fn(function_handler))
                .await
            {
                eprintln!("Server error: {error}");
            }
        });
    }
}
