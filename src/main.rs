mod admin;

use axum::{
    Json, Router,
    routing::{get, post},
    extract::DefaultBodyLimit,
};
use serde_json::json;
use tokio::fs;

include!("embedding.rs");

#[tokio::main]
async fn main() {
    let _ = fs::create_dir_all(admin::UPLOAD_DIR).await;

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/embedding", post(embedding_post))
        .route("/search", post(search_post))
        // .route("/upload", post(admin::upload_images))
        // .route("/sync", post(admin::sync_r2))
        // .route("/images", get(admin::list_images))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .fallback(fallback);

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

async fn search_post(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let text = payload["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return Json(json!({ "error": "text field is required" }));
    }
    let results = vec![
        json!({ "title": "幸楽苑　担々麺", "URL": "/posts/kourakuen/" }),
        json!({ "title": "幸楽苑", "URL": "/posts/kourakuen2/" }),
    ];
    Json(json!({ "results": results }))
}

async fn fallback() -> &'static str {
    println!("Fallback endpoint accessed");
    "API : 404 Not Found"
}
