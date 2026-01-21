use serde::{Deserialize, Serialize};

pub const EMBEDDINGS_PATH: &str = "embeddings.json";

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct IndexedPost {
    pub filename: String,
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub embedding: Vec<f32>,
}

impl Default for IndexedPost {
    fn default() -> Self {
        Self {
            filename: String::new(),
            url: String::new(),
            title: String::new(),
            thumbnail: None,
            embedding: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct SearchIndex {
    posts: Vec<IndexedPost>,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
    pub score: f32,
}

impl SearchIndex {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let posts: Vec<IndexedPost> = serde_json::from_str(&content)?;
        println!("Loaded {} posts from {}", posts.len(), path);
        Ok(Self { posts })
    }

    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<SearchResult> {
        let mut scored: Vec<_> = self
            .posts
            .iter()
            .map(|post| {
                let score = cosine_similarity(query_embedding, &post.embedding);
                (post, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(top_k)
            .map(|(post, score)| SearchResult {
                title: post.title.clone(),
                url: post.url.clone(),
                thumbnail: post.thumbnail.clone(),
                score,
            })
            .collect()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
