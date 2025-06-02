use netlify_lambda_http::{
    lambda::{self, Context},
    IntoResponse, Request, Response
};
use aws_lambda_events::encodings::Body;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// Event input structure for the Netlify Function
#[derive(Deserialize)]
struct RequestBody {
    #[serde(default)]
    name: String,
}

/// Response structure for the Netlify Function
#[derive(Serialize)]
struct ResponseBody {
    message: String,
    request_id: String,
}

#[lambda::lambda(http)]
#[tokio::main]
async fn main(req: Request, ctx: Context) -> Result<impl IntoResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Initialize the tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("FerrisUp Netlify Function starting");
    
    // Parse the request body
    let body = req.body();
    let request_body = match body {
        Body::Text(text) => {
            serde_json::from_str::<RequestBody>(text).unwrap_or(RequestBody { name: "".to_string() })
        },
        Body::Binary(binary) => {
            serde_json::from_slice::<RequestBody>(binary).unwrap_or(RequestBody { name: "".to_string() })
        },
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
        request_id: ctx.request_id,
    };
    
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&response_body)?)?)
}
