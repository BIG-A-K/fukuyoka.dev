mod admin;
mod search;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::Uri,
    routing::{get, post},
};
use serde_json::json;
use std::sync::Arc;
use tokio::fs;

use rust_app::embedding::EmbeddingModel;
use search::{SearchIndex, EMBEDDINGS_PATH};

#[derive(Clone)]
struct AppState {
    model: Arc<EmbeddingModel>,
    index: Arc<SearchIndex>,
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

    // Load search index
    println!("Loading search index from {EMBEDDINGS_PATH}...");
    let index = Arc::new(
        SearchIndex::load(EMBEDDINGS_PATH).expect("Failed to load search index"),
    );

    let state = AppState { model, index };

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
        .route("/embedding", post(embedding_post))
        .route("/search", post(search_post))
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

async fn embedding_post(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let text = payload["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return Json(json!({ "error": "text field is required" }));
    }

    match state.model.embed(text) {
        Ok(embedding) => {
            println!("Generated embedding for text: {text}");
            Json(json!({ "embedding": embedding }))
        }
        Err(e) => Json(json!({ "error": format!("Failed to embed: {e}") })),
    }
}

async fn search_post(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let text = payload["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return Json(json!({ "error": "text field is required" }));
    }

    // e5モデルはクエリに "query: " プレフィックスが必要
    let query_text = format!("query: {}", text);

    match state.model.embed(&query_text) {
        Ok(query_embedding) => {
            let results = state.index.search(&query_embedding, 10);
            println!("Search for '{}': {} results", text, results.len());
            Json(json!({ "results": results }))
        }
        Err(e) => Json(json!({ "error": format!("Failed to embed query: {e}") })),
    }
}

async fn fallback(uri: Uri) -> String {
    println!("Fallback endpoint accessed: {uri}");
    format!("API : 404 Not Found - {uri}")
}
