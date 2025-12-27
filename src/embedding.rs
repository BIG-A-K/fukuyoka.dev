async fn embedding_post(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    // ここに埋め込み生成のロジックを実装します
    // 例として768次元の1,2,3,...,768のベクトルを返す
    let text = payload["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return Json(json!({ "error": "text field is required" }));
    }
    let embedding_vector = embedding(text);
    Json(json!({ "embedding": embedding_vector }))
}

fn embedding(text: &str) -> Vec<f32> {
    // ここに埋め込み生成のロジックを実装します
    // 例として768次元の1,2,3,...,768のベクトルを返す
    let embedding_vector: Vec<f32> = (1..=768).map(|x| x as f32).collect();
    // 正規化する
    let norm: f32 = embedding_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    let embedding_vector: Vec<f32> = embedding_vector.iter().map(|x| x / norm).collect();
    embedding_vector
}