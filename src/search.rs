use axum::Json;
use axum::extract::State;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::db::connect_db;
use crate::embedding::EmbeddingModel;
use crate::morphology::morphology;

#[derive(Deserialize)]
pub struct SearchRequest {
    pub text: String,
}

struct SearchResult {
    title: String,
    url: String,
    thumbnail: Option<String>,
    score: f64,
}

#[derive(Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub status: bool,
    pub results: Vec<SearchResultItem>,
    pub msg: Option<String>,
}

pub async fn search(State(model): State<Arc<EmbeddingModel>>, Json(payload): Json<SearchRequest>) -> Json<SearchResponse> {
    let text = payload.text.trim();
    if text.is_empty() {
        return Json(SearchResponse { status: false, results: vec![], msg: Some("Query is empty".into()) });
    }
    println!("Received search query: {text}");
    let client = match connect_db().await {
        Ok(connection) => connection,
        Err(e) => {
            println!("Error connecting to database: {e}");
            return Json(SearchResponse { status: false, results: vec![], msg: Some(format!("Error connecting to database: {e}")) });
        }
    };

    // Generate embedding for the query
    let embedding = match model.embed(&format!("query: {text}")) {
        Ok(emb) => emb,
        Err(e) => {
            println!("Error generating embedding: {e}");
            return Json(SearchResponse { status: false, results: vec![], msg: Some(format!("Error generating embedding: {e}")) });
        }
    };

    // Morphological analysis for BM25
    let morph_tokens = match morphology(text) {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Error generating tokens: {e}");
            return Json(SearchResponse { status: false, results: vec![], msg: Some(format!("Error generating tokens: {e}")) });
        }
    };
    let token_str = morph_tokens.join(" ");

    let vector_results = vector_similarity_search(&client, embedding, 10).await;
    let token_results = bm25(&client, token_str, 10).await;
    let combined_results = rrf_rerank(vector_results, token_results);

    Json(SearchResponse {
        status: true,
        results: combined_results.into_iter().map(|r| SearchResultItem {
            title: r.title,
            url: r.url,
            thumbnail: r.thumbnail,
        }).collect(),
        msg: None,
    })
}


/// Reciprocal Rank Fusion: combines two ranked lists by title key
fn rrf_rerank(embed_results: Vec<SearchResult>, token_results: Vec<SearchResult>) -> Vec<SearchResult> {
    let k = 60.0_f64; // RRF constant
    let mut scores: HashMap<String, (f64, SearchResult)> = HashMap::new();

    for (rank, result) in embed_results.into_iter().enumerate() {
        let rrf_score = 1.0 / (k + rank as f64 + 1.0);
        scores.insert(result.title.clone(), (rrf_score, result));
    }

    for (rank, result) in token_results.into_iter().enumerate() {
        let rrf_score = 1.0 / (k + rank as f64 + 1.0);
        scores
            .entry(result.title.clone())
            .and_modify(|(s, _)| *s += rrf_score)
            .or_insert((rrf_score, result));
    }

    let mut combined: Vec<(f64, SearchResult)> = scores.into_values().collect();
    combined.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    combined.into_iter().take(5).map(|(_, r)| r).collect()
}

async fn vector_similarity_search(client: &Client, embedding: Vec<f32>, top_k: usize) -> Vec<SearchResult> {
    let table_name = match env::var("POSTGRES_TABLE") {
        Ok(v) => v,
        Err(_) => { println!("POSTGRES_TABLEを設定してください"); return Vec::new(); }
    };
    let sql = format!(
        "SELECT title, url, thumbnail, embeds <=> $1::text::vector AS score \
         FROM {table_name} ORDER BY score LIMIT {top_k}"
    );
    let embedding_str = format!("[{}]", embedding.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
    let mut results = Vec::new();
    match client.query(&sql, &[&embedding_str]).await {
        Ok(rows) => {
            for row in rows {
                let title: String = row.get(0);
                let url: String = row.get(1);
                let thumbnail: Option<String> = row.get(2);
                let score: f64 = row.get(3);
                results.push(SearchResult { title, url, thumbnail, score });
            }
        }
        Err(e) => {
            println!("Error in vector search: {e}");
        }
    }
    results
}

async fn bm25(client: &Client, tokens: String, top_k: usize) -> Vec<SearchResult> {
    let table_name = match env::var("POSTGRES_TABLE") {
        Ok(v) => v,
        Err(_) => { println!("POSTGRES_TABLEを設定してください"); return Vec::new(); }
    };
    let sql = format!(
        "SELECT title, url, thumbnail, tokens <@> to_bm25query($1, '{table_name}_tokens_idx') AS score \
         FROM {table_name} ORDER BY score ASC LIMIT {top_k}"
    );
    let mut results = Vec::new();
    match client.query(&sql, &[&tokens]).await {
        Ok(rows) => {
            for row in rows {
                let title: String = row.get(0);
                let url: String = row.get(1);
                let thumbnail: Option<String> = row.get(2);
                let score: f64 = row.get(3);
                results.push(SearchResult { title, url, thumbnail, score });
            }
        }
        Err(e) => {
            println!("Error in BM25 search: {e}");
        }
    }
    results
}
