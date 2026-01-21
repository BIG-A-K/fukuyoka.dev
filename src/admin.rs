use axum::Json;
use axum::body::Body;
use axum::extract::Path as AxumPath;
use axum::http::{Response, StatusCode, header};
use axum_extra::extract::Multipart;
use serde_json::json;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

pub const UPLOAD_DIR: &str = "/tmp/data";

pub async fn upload_images(mut multipart: Multipart) -> Json<serde_json::Value> {
    // 画像をlocalに一時保存する処理
    println!("ADMIN : Upload endpoint accessed");
    let mut uploaded_count = 0;
    let mut errors: Vec<String> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("{}: {}", file_name, e));
                continue;
            }
        };

        let file_path = format!("{}/{}", UPLOAD_DIR, file_name);
        match fs::write(&file_path, &data).await {
            Ok(_) => {
                println!("Saved file: {}", file_path);
                uploaded_count += 1;
            }
            Err(e) => {
                println!("Failed to save file {}: {}", file_path, e);
                errors.push(format!("{}: {}", file_name, e));
            }
        }
    }

    if uploaded_count > 0 {
        Json(json!({
            "status": "ok",
            "uploaded": uploaded_count,
            "message": format!("{}件のファイルをアップロードしました", uploaded_count)
        }))
    } else if !errors.is_empty() {
        Json(
            json!({ "status": "error", "error": format!("アップロード失敗: {}", errors.join(", ")) }),
        )
    } else {
        Json(json!({ "status": "error", "error": "ファイルが見つかりませんでした" }))
    }
}

/// ローカルからR2にアップロード
pub async fn push_storage() -> Json<serde_json::Value> {
    println!("ADMIN : Push to storage endpoint accessed");

    let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();
    let bucket = std::env::var("R2_BUCKET").unwrap_or_else(|_| "fukuyoka-photo".to_string());

    if endpoint.is_empty() {
        return Json(json!({ "status": "error", "error": "R2_ENDPOINTが設定されていません" }));
    }

    // exiftoolでJPGのメタデータを削除
    let exif_output = Command::new("exiftool")
        .args([
            "-overwrite_original",
            "-all=",
            "-ext",
            "jpg",
            "-ext",
            "jpeg",
            UPLOAD_DIR,
        ])
        .output()
        .await;

    match &exif_output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            println!("exiftool completed: {} {}", stdout, stderr);
        }
        Err(e) => {
            println!("exiftool warning: {}", e);
            // exiftoolが失敗したらアップロードは中止
            return Json(
                json!({ "status": "error", "error": format!("exiftoolの実行に失敗しました: {}", e) }),
            );
        }
    }

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
                println!("Push completed: {}", stdout);
                Json(json!({ "status": "ok", "message": "R2へのアップロードが完了しました" }))
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("Push failed: {}", stderr);
                Json(
                    json!({ "status": "error", "error": format!("アップロードに失敗しました: {}", stderr) }),
                )
            }
        }
        Err(e) => {
            println!("Push command failed: {}", e);
            Json(
                json!({ "status": "error", "error": format!("AWS CLIの実行に失敗しました: {}", e) }),
            )
        }
    }
}

pub async fn list_images() -> Json<serde_json::Value> {
    println!("ADMIN : List images endpoint accessed");

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
            return Json(
                json!({ "status": "error", "error": format!("ディレクトリの読み込みに失敗しました: {}", e) }),
            );
        }
    }

    images.sort();
    Json(json!({ "status": "ok", "images": images }))
}

/// ローカル画像を配信
pub async fn serve_local_image(
    AxumPath(filename): AxumPath<String>,
) -> Result<Response<Body>, StatusCode> {
    // パストラバーサル対策
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let file_path = format!("{}/{}", UPLOAD_DIR, filename);
    let data = fs::read(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let content_type = match Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// R2とローカルの差分を取得
pub async fn diff_images() -> Json<serde_json::Value> {
    println!("ADMIN : Diff images endpoint accessed");

    let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();
    let bucket = std::env::var("R2_BUCKET").unwrap_or_else(|_| "fukuyoka-photo".to_string());

    if endpoint.is_empty() {
        return Json(json!({ "status": "error", "error": "R2_ENDPOINTが設定されていません" }));
    }

    // ローカルの画像一覧を取得
    let mut local_images: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(mut entries) = fs::read_dir(UPLOAD_DIR).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(filename) = entry.file_name().to_str() {
                let ext = Path::new(filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if ["jpg", "jpeg", "png"].contains(&ext.as_str()) {
                    local_images.insert(filename.to_string());
                }
            }
        }
    }

    // R2の画像一覧を取得
    let output = Command::new("aws")
        .args([
            "s3",
            "ls",
            &format!("s3://{}/", bucket),
            "--endpoint-url",
            &endpoint,
        ])
        .output()
        .await;

    let mut r2_images: std::collections::HashSet<String> = std::collections::HashSet::new();
    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                for line in stdout.lines() {
                    // 出力形式: "2024-01-01 12:00:00     12345 filename.jpg"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let filename = parts[3];
                        let ext = Path::new(filename)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();
                        if ["jpg", "jpeg", "png"].contains(&ext.as_str()) {
                            r2_images.insert(filename.to_string());
                        }
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                return Json(
                    json!({ "status": "error", "error": format!("R2一覧取得失敗: {}", stderr) }),
                );
            }
        }
        Err(e) => {
            return Json(json!({ "status": "error", "error": format!("AWS CLI実行失敗: {}", e) }));
        }
    }

    // 差分を計算
    let mut only_local: Vec<String> = local_images.difference(&r2_images).cloned().collect();
    let mut only_r2: Vec<String> = r2_images.difference(&local_images).cloned().collect();
    let mut synced: Vec<String> = local_images.intersection(&r2_images).cloned().collect();

    only_local.sort();
    only_r2.sort();
    synced.sort();

    Json(json!({
        "status": "ok",
        "only_local": only_local,
        "only_r2": only_r2,
        "synced": synced
    }))
}
