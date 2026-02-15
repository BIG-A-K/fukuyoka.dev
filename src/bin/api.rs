use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::Uri,
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;
use tokio::fs;

use rust_app::admin;
use rust_app::embedding::EmbeddingModel;
use rust_app::search::search;

#[derive(Clone)]
struct AppState {
    model: Arc<EmbeddingModel>,
}

#[tokio::main]
async fn main() {
    let _ = fs::create_dir_all(admin::UPLOAD_DIR).await;

    // Load embedding model
    println!("Loading embedding model...");
    let model = Arc::new(
        EmbeddingModel::new("intfloat/multilingual-e5-base")
            .expect("Failed to load embedding model"),
    );
    println!("Embedding model loaded");

    let state = AppState { model };

    // Admin routes (protected by nginx Basic Auth at /api/akasha/*)
    let admin_routes = Router::new()
        .route("/upload", post(admin::upload_images))
        .route("/push", post(admin::push_storage))
        .route("/images", get(admin::list_images))
        .route("/local-image/{filename}", get(admin::serve_local_image))
        .route("/diff", get(admin::diff_images));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/search", post(search::<AppState>))
        .nest("/akasha", admin_routes)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .fallback(fallback)
        .with_state(state);

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
