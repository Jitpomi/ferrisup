use serde::{Deserialize, Serialize};
use http_body_util::BodyExt;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use vercel_runtime::{run, service_fn, Error, Request, Response, ResponseBody};

/// Event input structure for the Vercel Function
#[derive(Deserialize)]
struct RequestBody {
    #[serde(default)]
    name: String,
}

/// Response structure for the Vercel Function
#[derive(Serialize)]
struct ApiResponse {
    message: String,
    request_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}

async fn handler(req: Request) -> Result<Response<ResponseBody>, Error> {
    // Initialize the tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("FerrisUp Vercel Function starting");
    
    // Parse the request body
    let (parts, body) = req.into_parts();
    let body = body.collect().await?.to_bytes();
    let request_body = serde_json::from_slice::<RequestBody>(&body)
        .unwrap_or(RequestBody { name: String::new() });
    
    // Create a response with a greeting using the name from the request or a default
    let name = if request_body.name.is_empty() {
        "World".to_string()
    } else {
        request_body.name
    };
    
    let message = format!("Hello, {}! Welcome to your FerrisUp serverless function.", name);
    
    // Return the formatted response
    let response_body = ApiResponse {
        message,
        request_id: parts.headers.get("x-vercel-id").map_or("unknown", |v| v.to_str().unwrap_or("unknown")).to_string(),
    };
    
    Ok(Response::builder()
        .status(http::StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(ResponseBody::from(serde_json::to_string(&response_body)?))?)
}
