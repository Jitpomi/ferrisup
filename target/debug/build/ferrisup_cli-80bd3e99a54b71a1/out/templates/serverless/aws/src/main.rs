use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// Event input structure for the Lambda function
#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    name: String,
}

/// Response structure for the Lambda function
#[derive(Serialize)]
struct Response {
    message: String,
    request_id: String,
}

/// Main Lambda handler function
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let (event, context) = event.into_parts();
    let request_id = context.request_id;
    
    info!(
        message = "Processing serverless function request",
        request_id = %request_id,
        name = %event.name
    );
    
    // Create a response with a greeting using the name from the request or a default
    let name = if event.name.is_empty() {
        "World".to_string()
    } else {
        event.name
    };
    
    let message = format!("Hello, {}! Welcome to your FerrisUp serverless function.", name);
    
    // Return the formatted response
    Ok(Response {
        message,
        request_id,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize the tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("FerrisUp AWS Lambda function starting");
    
    // Start the Lambda runtime and register the handler
    lambda_runtime::run(service_fn(function_handler)).await?;
    
    Ok(())
}
