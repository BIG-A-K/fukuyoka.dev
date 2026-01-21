use clap::Parser;
use serde::Serialize;
use std::path::PathBuf;

use rust_app::embedding::EmbeddingModel;
use rust_app::post::{Post, load_post, load_posts};

#[derive(Parser, Debug)]
#[command(name = "embed")]
#[command(about = "Create embeddings for fukuyoka blog posts")]
struct Args {
    /// Process a single markdown file
    #[arg(short, long)]
    src: Option<PathBuf>,

    /// Process all posts in the posts directory
    #[arg(short, long)]
    all: bool,

    /// Output file path (default: <src-filename>.json or embeddings.json for --all)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Posts directory (default: frontend/content/posts)
    #[arg(short, long, default_value = "frontend/content/posts")]
    posts_dir: PathBuf,
}

#[derive(Serialize)]
struct PostWithEmbedding {
    filename: String,
    filepath: String,
    title: String,
    date: String,
    thumbnail: Option<String>,
    tags: Vec<String>,
    categories: Vec<String>,
    body: String,
    embedding: Vec<f32>,
}

fn create_embeddings(posts: Vec<Post>, model: &EmbeddingModel) -> Vec<PostWithEmbedding> {
    posts
        .into_iter()
        .filter_map(|post| {
            let text = post.text_for_embedding();
            match model.embed(&text) {
                Ok(embedding) => {
                    println!("  Embedded: {} ({})", post.title, post.filename);
                    Some(PostWithEmbedding {
                        filename: post.filename,
                        filepath: post.filepath,
                        title: post.title,
                        date: post.date,
                        thumbnail: post.thumbnail,
                        tags: post.tags,
                        categories: post.categories,
                        body: post.body,
                        embedding,
                    })
                }
                Err(e) => {
                    eprintln!("  Failed to embed {}: {}", post.filename, e);
                    None
                }
            }
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

    // Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        if args.all {
            PathBuf::from("embeddings.json")
        } else if let Some(src) = &args.src {
            let stem = src.file_stem().unwrap_or_default();
            PathBuf::from(format!("{}.json", stem.to_string_lossy()))
        } else {
            PathBuf::from("error.json")
        }
    });

    if posts.is_empty() {
        println!("No posts to process.");
        return Ok(());
    }

    println!("Creating embeddings...");
    let posts_with_embeddings = create_embeddings(posts, &model);

    // Save to JSON
    let json = serde_json::to_string_pretty(&posts_with_embeddings)?;
    std::fs::write(&output_path, json)?;
    println!("Saved embeddings to {:?}", output_path);

    // Print embedding dimensions
    if let Some(first) = posts_with_embeddings.first() {
        println!("Embedding dimensions: {}", first.embedding.len());
    }

    Ok(())
}
