use axum::Json;
use postgres::Client;
use serde::{Deserialize, Serialize};

use crate::db::connect_db;
use crate::morphology::morphology;

#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
}

struct SearchResult {
    content: String,
    score: f32,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub status: bool,
    pub results: Vec<String>,
    pub msg: Option<String>,
}

pub async fn search<S>(Json(payload): Json<SearchRequest>) -> Json<SearchResponse> {
    let text = payload.query.trim();
    if text.is_empty() {
        return Json(SearchResponse { status: false, results: vec![], msg: Some("Query is empty".into()) });
    }

    let morph_tokens = match morphology(text) {
        Ok(tokens) => tokens,
        Err(e) => {
            println!("Error generating tokens: {e}");
            return Json(SearchResponse { status: false, results: vec![], msg: Some(format!("Error generating tokens: {e}")) });
        }
    };
    let token_str = morph_tokens.join(" ");

    let mut con = match connect_db() {
        Ok(connection) => connection,
        Err(e) => {
            println!("Error connecting to database: {e}");
            return Json(SearchResponse { status: false, results: vec![], msg: Some(format!("Error connecting to database: {e}")) });
        }
    };

    let vector_results = vector_similarity_search(&mut con, vec![], 5);
    let token_results = bm25(&mut con, token_str, 5);
    let combined_results = rff_rerank(vector_results, token_results);

    Json(SearchResponse {
        status: true,
        results: combined_results.into_iter().map(|r| r.content).collect(),
        msg: None,
    })
}


fn rff_rerank(embed_results: Vec<SearchResult>, token_results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut combined_results = Vec::new();
    for (embed_result, token_result) in embed_results.iter().zip(token_results.iter()) {
        let combined_score = 0.7 * embed_result.score + 0.3 * token_result.score;
        combined_results.push(SearchResult {
            content: embed_result.content.clone(),
            score: combined_score,
        });
    }
    combined_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    combined_results
}

fn vector_similarity_search(connection: &mut Client, embedding: Vec<f32>, top_k: usize) -> Vec<SearchResult> {
    let sql = format!(
        "SELECT content, embedding <=> $1::vector AS score \
         FROM documents ORDER BY score LIMIT {top_k}"
    );
    let embedding_str = format!("[{}]", embedding.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
    let mut results = Vec::new();
    match connection.query(&sql, &[&embedding_str]) {
        Ok(rows) => {
            for row in rows {
                let content: String = row.get(0);
                let score: f32 = row.get(1);
                results.push(SearchResult { content, score });
            }
        }
        Err(e) => {
            println!("Error in vector search: {e}");
        }
    }
    results
}

fn bm25(connection: &mut Client, tokens: String, top_k: usize) -> Vec<SearchResult> {
    let sql = format!(
        "SELECT content, jp_tokenized_content <@> to_bm25query($1, 'bm25_idx') AS score \
         FROM documents ORDER BY score ASC LIMIT {top_k}"
    );
    let mut results = Vec::new();
    match connection.query(&sql, &[&tokens]) {
        Ok(rows) => {
            for row in rows {
                let content: String = row.get(0);
                let score: f32 = row.get(1);
                results.push(SearchResult { content, score });
            }
        }
        Err(e) => {
            println!("Error in BM25 search: {e}");
        }
    }
    results
}
