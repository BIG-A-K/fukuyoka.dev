use axum::{
    Json, Router,
    routing::{get, post},
};
use serde_json::json;
include!("embedding.rs");

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/embedding", post(embedding_post))
        .fallback(fallback);
    // サーバ起動
    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap(),
        app,
    )
    .await
    .unwrap();
}

async fn root() -> &'static str {
    println!("Root endpoint accessed");
    "Hello, I am Fukuyoka"
}

async fn health_check() -> Json<serde_json::Value> {
    println!("Health check OK");
    Json(json!({ "status": "ok" }))
}

async fn fallback() -> &'static str {
    println!("Fallback endpoint accessed");
    "API : 404 Not Found"
}

