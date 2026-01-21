mod admin;

use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::Uri,
    routing::{get, post},
};
use serde_json::json;
use tokio::fs;

use rust_app::embedding::EmbeddingModel;

#[tokio::main]
async fn main() {
    let _ = fs::create_dir_all(admin::UPLOAD_DIR).await;

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

async fn embedding_post(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let text = payload["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return Json(json!({ "error": "text field is required" }));
    }

    match EmbeddingModel::new("intfloat/multilingual-e5-base") {
        Ok(model) => match model.embed(text) {
            Ok(embedding) => {
                println!("Generated embedding for text: {text}");
                Json(json!({ "embedding": embedding }))
            }
            Err(e) => Json(json!({ "error": format!("Failed to embed: {e}") })),
        },
        Err(e) => Json(json!({ "error": format!("Failed to load model: {e}") })),
    }
}

async fn search_post(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let text = payload["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return Json(json!({ "error": "text field is required" }));
    }
    let results = vec![
        json!({ "title": "幸楽苑　担々麺", "URL": "/posts/kourakuen/", "thumbnail": "/photo/20251005/IMG_6187.jpeg" }),
        json!({ "title": "幸楽苑", "URL": "/posts/kourakuen2/", "thumbnail": "/photo/20251005/IMG_6187.jpeg" }),
    ];
    Json(json!({ "results": results }))
}

async fn fallback(uri: Uri) -> String {
    println!("Fallback endpoint accessed: {uri}");
    format!("API : 404 Not Found - {uri}")
}
