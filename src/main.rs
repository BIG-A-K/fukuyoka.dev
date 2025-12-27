use axum::{Json, Router, routing::{get,post}};
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
    "Hello, I am Fukuyoka"
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn fallback() -> &'static str {
    "API : 404 Not Found"
}

async fn example_post(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(payload)
}