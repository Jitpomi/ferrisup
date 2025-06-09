use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router, serve,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tokio::net::TcpListener;

// Application state
struct AppState {
    app_name: String,
    request_count: Mutex<usize>,
}

// Response models
#[derive(Serialize)]
struct InfoResponse {
    app_name: String,
    version: String,
    request_count: usize,
}

#[derive(Deserialize)]
struct CreateItemRequest {
    name: String,
}

#[derive(Serialize)]
struct CreateItemResponse {
    id: u64,
    name: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create app state
    let state = Arc::new(AppState {
        app_name: "FerrisUp Axum Server".to_string(),
        request_count: Mutex::new(0),
    });

    // Build the router
    let app = Router::new()
        .route("/", get(root))
        .route("/api/info", get(info))
        .route("/api/items", post(create_item))
        .with_state(state);

    // Define the address to listen on
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on http://{}", addr);

    // Run the server
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}

// Route handlers
async fn root() -> &'static str {
    "Hello, FerrisUp with Axum!"
}

async fn info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Increment request count
    let mut count = state.request_count.lock().await;
    *count += 1;

    // Create response
    let response = InfoResponse {
        app_name: state.app_name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        request_count: *count,
    };

    Json(response)
}

async fn create_item(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateItemRequest>,
) -> impl IntoResponse {
    // Increment request count
    let mut count = state.request_count.lock().await;
    *count += 1;

    // In a real app, you'd save to a database
    let item_id = 42; // Placeholder

    // Create response
    let response = CreateItemResponse {
        id: item_id,
        name: request.name,
    };

    (StatusCode::CREATED, Json(response))
}
