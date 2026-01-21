use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Post {
    pub filename: String,
    pub filepath: String,
    pub title: String,
    pub date: String,
    pub thumbnail: Option<String>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub body: String,
}

impl Post {
    /// Create text for embedding with e5 model prefix
    pub fn text_for_embedding(&self) -> String {
        format!("passage: {} {}", self.title, self.body)
    }
}

/// Parse TOML frontmatter from markdown content
pub fn parse_frontmatter(
    content: &str,
) -> (HashMap<String, String>, Vec<String>, Vec<String>, String) {
    let pattern = Regex::new(r"(?s)^\+\+\+\n(.*?)\n\+\+\+\n(.*)$").unwrap();

    let Some(caps) = pattern.captures(content) else {
        return (HashMap::new(), vec![], vec![], content.to_string());
    };

    let frontmatter_str = &caps[1];
    let body = caps[2].trim().to_string();

    let mut metadata = HashMap::new();
    let mut tags = Vec::new();
    let mut categories = Vec::new();

    for line in frontmatter_str.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            // Parse arrays
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                let items: Vec<String> = inner
                    .split(',')
                    .map(|s| s.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                match key {
                    "tags" => tags = items,
                    "categories" => categories = items,
                    _ => {}
                }
            } else {
                // Remove quotes from string values
                let cleaned = value
                    .trim_start_matches(|c| c == '\'' || c == '"')
                    .trim_end_matches(|c| c == '\'' || c == '"')
                    .to_string();
                metadata.insert(key.to_string(), cleaned);
            }
        }
    }

    (metadata, tags, categories, body)
}

/// Remove markdown syntax for cleaner embedding
pub fn clean_markdown(text: &str) -> String {
    // Remove images: ![alt](url)
    let re_images = Regex::new(r"!\[.*?\]\(.*?\)").unwrap();
    let text = re_images.replace_all(text, "");

    // Remove links but keep text: [text](url) -> text
    let re_links = Regex::new(r"\[([^\]]+)\]\([^\)]+\)").unwrap();
    let text = re_links.replace_all(&text, "$1");

    // Remove extra newlines
    let re_newlines = Regex::new(r"\n+").unwrap();
    let text = re_newlines.replace_all(&text, "\n");

    text.trim().to_string()
}

/// Load a single post from a markdown file
pub fn load_post(filepath: &Path) -> Option<Post> {
    let content = fs::read_to_string(filepath).ok()?;
    let (metadata, tags, categories, body) = parse_frontmatter(&content);
    let clean_body = clean_markdown(&body);

    // Skip drafts
    if metadata.get("draft").map(|s| s.as_str()) == Some("true") {
        return None;
    }

    Some(Post {
        filename: filepath.file_stem()?.to_string_lossy().to_string(),
        filepath: filepath.to_string_lossy().to_string(),
        title: metadata.get("title").cloned().unwrap_or_default(),
        date: metadata.get("date").cloned().unwrap_or_default(),
        thumbnail: metadata.get("thumbnail").cloned(),
        tags,
        categories,
        body: clean_body,
    })
}

/// Load all posts from a directory
pub fn load_posts(posts_dir: &Path) -> Vec<Post> {
    let pattern = posts_dir.join("*.md");
    let pattern_str = pattern.to_string_lossy();

    glob::glob(&pattern_str)
        .expect("Failed to read glob pattern")
        .filter_map(|entry| entry.ok())
        .filter_map(|path| load_post(&path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"+++
title = "Test Post"
date = "2024-01-01"
tags = ['tag1', 'tag2']
+++
This is the body."#;

        let (metadata, tags, _, body) = parse_frontmatter(content);
        assert_eq!(metadata.get("title").unwrap(), "Test Post");
        assert_eq!(tags, vec!["tag1", "tag2"]);
        assert_eq!(body, "This is the body.");
    }

    #[test]
    fn test_clean_markdown() {
        let text = "Hello ![img](url) and [link](http://example.com)";
        let cleaned = clean_markdown(text);
        assert_eq!(cleaned, "Hello  and link");
    }
}
