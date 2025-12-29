use axum::Json;
use axum_extra::extract::Multipart;
use serde_json::json;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

pub const UPLOAD_DIR: &str = "/tmp/data";

pub async fn upload_images(mut multipart: Multipart) -> Json<serde_json::Value> {
    // 画像をlocalに一時保存する処理
    println!("Upload endpoint accessed");
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.unwrap();

        let file_path = format!("{}/{}", UPLOAD_DIR, file_name);
        match fs::write(&file_path, &data).await {
            Ok(_) => {
                println!("Saved file: {}", file_path);
                return Json(json!({ "status": "ok", "message": "ファイルが正常にアップロードされました" }));
            }
            Err(e) => {
                println!("Failed to save file {}: {}", file_path, e);
                return Json(json!({ "status": "error", "error": format!("ファイルの保存に失敗しました: {}", e) }));
            }
        }
    }

    Json(json!({ "status": "error", "error": "ファイルが見つかりませんでした" }))
}

pub async fn sync_r2() -> Json<serde_json::Value> {
    println!("Sync endpoint accessed");

    let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();
    let bucket = std::env::var("R2_BUCKET").unwrap_or_else(|_| "fukuyoka-photo".to_string());

    let output = Command::new("aws")
        .args([
            "s3",
            "sync",
            UPLOAD_DIR,
            &format!("s3://{}", bucket),
            "--endpoint-url",
            &endpoint,
        ])
        .output()
        .await;

    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                println!("Sync completed: {}", stdout);
                Json(json!({ "status": "ok", "message": "R2への同期が完了しました" }))
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("Sync failed: {}", stderr);
                Json(json!({ "status": "error", "error": format!("同期に失敗しました: {}", stderr) }))
            }
        }
        Err(e) => {
            println!("Sync command failed: {}", e);
            Json(json!({ "status": "error", "error": format!("AWS CLIの実行に失敗しました: {}", e) }))
        }
    }
}

pub async fn list_images() -> Json<serde_json::Value> {
    println!("List images endpoint accessed");

    let mut images: Vec<String> = Vec::new();

    match fs::read_dir(UPLOAD_DIR).await {
        Ok(mut entries) => {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(filename) = entry.file_name().to_str() {
                    let ext = Path::new(filename)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    if ["jpg", "jpeg", "png"].contains(&ext.as_str()) {
                        images.push(filename.to_string());
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to read directory: {}", e);
            return Json(json!({ "status": "error", "error": format!("ディレクトリの読み込みに失敗しました: {}", e) }));
        }
    }

    images.sort();
    Json(json!({ "status": "ok", "images": images }))
}
