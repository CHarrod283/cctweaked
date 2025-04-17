use std::fmt::Debug;
use axum::Router;
use axum::routing::{get, post};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(|| async {"hello world"}))
        .route("/", post(print_json));
    
    let listener =  tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap_or_else(|e| {
        error!("Failed to bind to address: {}", e);
        std::process::exit(1);
    });
    info!("Starting server on {}", listener.local_addr().expect("Failed to get local address we bound to"));
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        error!("Failed to start server: {}", e);
    });
}

async fn print_json(json: axum::Json<serde_json::Value>) -> String {
    info!("Received JSON: {:?}", json);
    format!("Received JSON: {:?}", json)
}