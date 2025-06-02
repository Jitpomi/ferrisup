use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct InfoResponse {
    name: String,
    version: String,
    framework: String,
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

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from FerrisUp with Actix Web!")
}

#[get("/api/info")]
async fn info() -> impl Responder {
    let info = InfoResponse {
        name: "FerrisUp API".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        framework: "Actix Web".to_string(),
    };
    
    HttpResponse::Ok().json(info)
}

#[post("/api/items")]
async fn create_item(item: web::Json<CreateItemRequest>) -> impl Responder {
    // In a real app, you would save to a database
    let item_id = 42; // Placeholder
    
    let response = CreateItemResponse {
        id: item_id,
        name: item.name.clone(),
    };
    
    HttpResponse::Created().json(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let port = 3000;
    log::info!("Starting server at http://localhost:{}", port);
    
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(info)
            .service(create_item)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
