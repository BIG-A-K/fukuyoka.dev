use axum::{
    Json, Router,
    http::Uri,
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;

use rust_app::embedding::EmbeddingModel;
use rust_app::search::search;
use rust_app::db::initialize_db;

#[tokio::main]
async fn main() {
    // fileがあるかを確認
    let posts_data = "posts.json";
    if !std::path::Path::new(posts_data).exists() {
        eprintln!("Error: {} not found", posts_data);
        return;
    }
    println!("Found {}. Initializing database...", posts_data);
    // Initialize database
    if let Err(e) = initialize_db(posts_data).await {
        eprintln!("Error initializing database: {}", e);
        return;
    }

    // Load embedding model
    println!("Loading embedding model...");
    let model = Arc::new(
        EmbeddingModel::new("intfloat/multilingual-e5-base")
            .expect("Failed to load embedding model"),
    );
    println!("Embedding model loaded");

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/search", post(search))
        .fallback(fallback)
        .with_state(model);

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


async fn fallback(uri: Uri) -> String {
    println!("Fallback endpoint accessed: {uri}");
    format!("API : 404 Not Found - {uri}")
}
