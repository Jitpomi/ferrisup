use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

/// Event input structure for the Vercel Function
#[derive(Deserialize)]
struct RequestBody {
    #[serde(default)]
    name: String,
}

/// Response structure for the Vercel Function
#[derive(Serialize)]
struct ResponseBody {
    message: String,
    request_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // Initialize the tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("FerrisUp Vercel Function starting");
    
    // Parse the request body
    let request_body = match req.body() {
        Body::Text(text) => serde_json::from_str::<RequestBody>(text).unwrap_or(RequestBody { name: "".to_string() }),
        _ => RequestBody { name: "".to_string() },
    };
    
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
        request_id: req.headers().get("x-vercel-id").map_or("unknown", |v| v.to_str().unwrap_or("unknown")).to_string(),
    };
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&response_body)?))?)
}
