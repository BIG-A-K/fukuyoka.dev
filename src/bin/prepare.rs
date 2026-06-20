use clap::Parser;
use serde::Serialize;
use std::path::PathBuf;

use rust_app::embedding::EmbeddingModel;
use rust_app::morphology::morphology;
use rust_app::post::{load_post, load_posts, Post};

#[derive(Parser, Debug)]
#[command(name = "prepare")]
#[command(about = "Prepare posts with embeddings and morphology for PostgreSQL")]
struct Args {
    /// Process a single markdown file
    #[arg(short, long)]
    src: Option<PathBuf>,

    /// Process all posts in the posts directory
    #[arg(short, long)]
    all: bool,

    /// Output file path (default: posts.json)
    #[arg(short, long, default_value = "posts.json")]
    output: PathBuf,

    /// Posts directory (default: frontend/content/posts)
    #[arg(short, long, default_value = "frontend/content/posts")]
    posts_dir: PathBuf,
}

#[derive(Serialize)]
struct PreparedPost {
    filename: String,
    filepath: String,
    url: String,
    title: String,
    date: String,
    thumbnail: Option<String>,
    menus: Vec<String>,
    genres: Vec<String>,
    embedding: Vec<f32>,
    tokens: Vec<String>,
    search_text: String,
}

/// Create text for morphology: title + body + alt texts (without `passage:` prefix)
fn create_morphology_text(post: &Post) -> String {
    let mut text = format!("{} {}", post.title, post.body);

    if !post.alt_texts.is_empty() {
        text.push(' ');
        text.push_str(&post.alt_texts.join(" "));
    }

    text
}

fn process_posts(posts: Vec<Post>, model: &EmbeddingModel) -> Vec<PreparedPost> {
    posts
        .into_iter()
        .filter_map(|post| {
            let embedding_text = post.text_for_embedding();
            let morphology_text = create_morphology_text(&post);

            // Generate embedding
            let embedding = match model.embed(&embedding_text) {
                Ok(emb) => emb,
                Err(e) => {
                    eprintln!("  Failed to embed {}: {}", post.filename, e);
                    return None;
                }
            };

            // Generate tokens via morphology
            let tokens = match morphology(&morphology_text) {
                Ok(toks) => toks,
                Err(e) => {
                    eprintln!("  Failed to parse morphology for {}: {}", post.filename, e);
                    vec![]
                }
            };

            // Create search_text from tokens (space-separated for PostgreSQL BM25)
            let search_text = tokens.join(" ");

            let url = format!("/posts/{}/", post.filename);

            println!(
                "  Processed: {} ({} tokens, {} dim)",
                post.title,
                tokens.len(),
                embedding.len()
            );

            Some(PreparedPost {
                filename: post.filename,
                filepath: post.filepath,
                url,
                title: post.title,
                date: post.date,
                thumbnail: post.thumbnail,
                menus: post.menus,
                genres: post.genres,
                embedding,
                tokens,
                search_text,
            })
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.all && args.src.is_none() {
        eprintln!("Please provide either --src for single file or --all to process all posts.");
        std::process::exit(1);
    }

    println!("Loading model: intfloat/multilingual-e5-base");
    let model = EmbeddingModel::new("intfloat/multilingual-e5-base")?;

    let posts = if args.all {
        println!("Loading posts from: {:?}", args.posts_dir);
        let posts = load_posts(&args.posts_dir);
        println!("Loaded {} posts", posts.len());
        posts
    } else if let Some(src) = &args.src {
        match load_post(src) {
            Some(post) => {
                println!("Loaded post: {}", post.title);
                vec![post]
            }
            None => {
                eprintln!("Failed to load post or post is a draft: {:?}", src);
                std::process::exit(1);
            }
        }
    } else {
        vec![]
    };

    if posts.is_empty() {
        println!("No posts to process.");
        return Ok(());
    }

    println!("Processing posts...");
    let prepared_posts = process_posts(posts, &model);

    // Save to JSON
    let json = serde_json::to_string_pretty(&prepared_posts)?;
    std::fs::write(&args.output, json)?;
    println!("Saved {} posts to {:?}", prepared_posts.len(), args.output);

    // Print statistics
    if let Some(first) = prepared_posts.first() {
        println!("Embedding dimensions: {}", first.embedding.len());
        println!(
            "Sample tokens: {:?}",
            &first.tokens[..first.tokens.len().min(10)]
        );
    }

    Ok(())
}
