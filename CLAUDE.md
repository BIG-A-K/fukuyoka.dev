# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fukuyoka is a food diary blog site (幸者のふくよか日記) built with:
- **Backend**: Rust (Axum framework) serving API + embedding model
- **Frontend**: Hugo static site generator with custom theme
- **Deployment**: Self-hosted using Docker Compose with Cloudflare Tunnel
- **Image hosting**: Cloudflare R2 (proxied through nginx)

## Architecture

### Service Stack (docker-compose)

1. **fukuyoka_app** - Rust/Axum backend on port 80, accessed via `/api/*`
2. **fukuyoka_frontend** - Hugo server on port 1313
3. **fukuyoka_proxy** - nginx entry point, handles routing and R2 proxy
4. **cloudflared** - Cloudflare Tunnel for external access

### Nginx Routing
- `/api/*` → Rust public API (no auth)
- `/photo/*.{png,jpeg,jpg}` → R2 bucket (`photo.fukuyoka.dev`)
- `/*` → Hugo frontend

### Rust Code Structure
- `src/bin/api.rs` - Axum router, API endpoints
- `src/embedding.rs` - ML embedding model (multilingual-e5-base via candle)
- `src/post.rs` - Hugo post parsing
- `src/bin/embed.rs` - CLI tool for batch embedding generation

## Common Commands

### Docker
```bash
make build          # Build images
make up             # Start containers
make down           # Stop containers
make in             # Shell into fukuyoka_app container
make logs           # View logs
make restart-proxy  # Reload nginx config
```

### Rust (inside container via `make in`)
```bash
cargo check                    # Type check
cargo build --release          # Build
cargo run --release            # Run API server

# Embedding CLI for search indexing
cargo run --bin embed -- --all                    # Embed all posts → embeddings.json
cargo run --bin embed -- --src path/to/post.md    # Embed single post
cargo run --bin embed -- -h                       # Show help
```

### Hugo
```bash
make hugo           # Build static site
cd frontend && hugo # Same as above
```

## API Endpoints

### Public API (`/api/*`)
- `GET /` - Hello message
- `GET /health` - Health check
- `POST /search` - Search posts

## Environment Setup

Required `.env` file (copy from `template.env`):
- `DOMAIN` - Domain for Hugo baseURL
- `TUNNEL_TOKEN` - Cloudflare Tunnel token
- `R2_ENDPOINT` - R2 endpoint URL
- `R2_BUCKET` - R2 bucket name
- `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` - R2 credentials

## Development Notes

### Adding Blog Posts
1. Create markdown in `frontend/content/posts/`
2. Use front matter: title, date, tags, categories, thumbnail
3. Reference images as `/photo/filename.jpg`
