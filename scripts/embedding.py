#!/usr/bin/env python3
"""
Embedding script for fukuyoka blog posts.
Uses multilingual-e5-base model to create vector embeddings for semantic search.
"""

import glob
import json
import re
from pathlib import Path
import argparse

from sentence_transformers import SentenceTransformer


def parse_frontmatter(content: str) -> tuple[dict, str]:
    """Parse TOML frontmatter and return metadata and body."""
    pattern = r'^\+\+\+\n(.*?)\n\+\+\+\n(.*)$'
    match = re.match(pattern, content, re.DOTALL)

    if not match:
        return {}, content

    frontmatter_str = match.group(1)
    body = match.group(2).strip()

    # Simple TOML parsing for our use case
    metadata = {}
    for line in frontmatter_str.split('\n'):
        if '=' in line:
            key, value = line.split('=', 1)
            key = key.strip()
            value = value.strip()
            # Remove quotes
            if value.startswith("'") and value.endswith("'"):
                value = value[1:-1]
            elif value.startswith('"') and value.endswith('"'):
                value = value[1:-1]
            # Parse arrays
            elif value.startswith('[') and value.endswith(']'):
                value = [v.strip().strip("'\"") for v in value[1:-1].split(',')]
            metadata[key] = value

    return metadata, body


def clean_markdown(text: str) -> str:
    """Remove markdown syntax for cleaner embedding."""
    # Remove images
    text = re.sub(r'!\[.*?\]\(.*?\)', '', text)
    # Remove links but keep text
    text = re.sub(r'\[([^\]]+)\]\([^\)]+\)', r'\1', text)
    # Remove extra whitespace
    text = re.sub(r'\n+', '\n', text)
    return text.strip()


def load_posts(posts_dir: str) -> list[dict]:
    """Load all markdown posts from directory."""
    posts = []
    md_files = glob.glob(f"{posts_dir}/*.md")

    for filepath in md_files:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        metadata, body = parse_frontmatter(content)
        clean_body = clean_markdown(body)

        # Skip drafts
        if metadata.get('draft') == 'true':
            continue

        posts.append({
            'filename': Path(filepath).stem,
            'filepath': filepath,
            'title': metadata.get('title', ''),
            'date': metadata.get('date', ''),
            'tags': metadata.get('tags', []),
            'categories': metadata.get('categories', []),
            'body': clean_body,
            # For e5 models, prefix with "passage:" for documents
            'text_for_embedding': f"passage: {metadata.get('title', '')} {clean_body}"
        })

    return posts


def create_embeddings(posts: list[dict], model_name: str = "intfloat/multilingual-e5-base") -> list[dict]:
    """Create embeddings for all posts using multilingual-e5-base."""
    print(f"Loading model: {model_name}")
    model = SentenceTransformer(model_name)

    # Extract texts for embedding
    texts = [post['text_for_embedding'] for post in posts]

    print(f"Creating embeddings for {len(texts)} posts...")
    embeddings = model.encode(texts, show_progress_bar=True, convert_to_numpy=True)

    # Add embeddings to posts
    for post, embedding in zip(posts, embeddings):
        post['embedding'] = embedding.tolist()
        # Remove the text_for_embedding field from output
        del post['text_for_embedding']

    return posts


def save_embeddings(posts: list[dict], output_path: str):
    """Save posts with embeddings to JSON file."""
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(posts, f, ensure_ascii=False, indent=2)
    print(f"Saved embeddings to {output_path}")


def all_embedding():
    # Paths
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    posts_dir = project_root / "frontend" / "content" / "posts"
    output_path = script_dir / "embeddings.json"

    print(f"Loading posts from: {posts_dir}")
    posts = load_posts(str(posts_dir))
    print(f"Loaded {len(posts)} posts")

    for post in posts:
        print(f"  - {post['title']} ({post['filename']})")

    posts_with_embeddings = create_embeddings(posts)
    save_embeddings(posts_with_embeddings, str(output_path))

    # Print embedding dimensions
    if posts_with_embeddings:
        dim = len(posts_with_embeddings[0]['embedding'])
        print(f"Embedding dimensions: {dim}")


def main():
    args = argparse.ArgumentParser(description="Create embeddings for fukuyoka blog posts.")
    args.add_argument('--src', type=str, default=None, help="source markdown file path")
    args.add_argument('-all', action='store_true', help="process all posts")
    parsed_args = args.parse_args()
    if parsed_args.all:
        all_embedding()
    elif parsed_args.src:
        # Process single file
        filepath = parsed_args.src
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        metadata, body = parse_frontmatter(content)
        clean_body = clean_markdown(body)

        if metadata.get('draft') == 'true':
            print("The specified post is a draft. Skipping embedding.")
            return

        post = {
            'filename': Path(filepath).stem,
            'filepath': filepath,
            'title': metadata.get('title', ''),
            'date': metadata.get('date', ''),
            'tags': metadata.get('tags', []),
            'categories': metadata.get('categories', []),
            'body': clean_body,
            'text_for_embedding': f"passage: {metadata.get('title', '')} {clean_body}"
        }

        posts_with_embeddings = create_embeddings([post])
        output_path = Path(filepath).parent / f"{post['filename']}_embedding.json"
        save_embeddings(posts_with_embeddings, str(output_path))
    else:
        print("Please provide either --src for single file or -all to process all posts.")

if __name__ == "__main__":
    main()
