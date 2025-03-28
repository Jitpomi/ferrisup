use poem::{
    get, handler, Route, Server, EndpointExt, 
    web::Json, IntoResponse, Response, Result,
    http::StatusCode
};
use serde::{Deserialize, Serialize};
use std::env;

#[handler]
fn hello() -> &'static str {
    "Hello, FerrisUp!"
}

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    message: String,
    status: String,
}

#[handler]
fn api_info() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Welcome to the FerrisUp API".to_string(),
        status: "OK".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Initialize logger
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt::init();

    let app = Route::new()
        .at("/", get(hello))
        .at("/api/info", get(api_info))
        .catch_error(|err| async move {
            tracing::error!("Error: {:?}", err);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("text/plain")
                .body("Internal server error")
                .into_response()
        });

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Listening on {}", addr);

    Server::new(addr.parse()?).run(app).await
}
