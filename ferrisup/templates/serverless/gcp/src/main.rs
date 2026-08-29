use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
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

async fn function_handler(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
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
        .body(Full::new(Bytes::from(json)))
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
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Server listening on {}", addr);

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
