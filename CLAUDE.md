# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fukuyoka is a food diary blog site (幸者のふくよか日記) built with:
- **Backend**: Rust (Axum framework) serving a simple API
- **Frontend**: Hugo static site generator with a custom theme
- **Deployment**: Self-hosted using Docker Compose with Cloudflare Tunnel
- **Image hosting**: Cloudflare R2 (proxied through nginx for image files)

## Architecture

### Service Stack
The application runs as a multi-container Docker setup orchestrated by docker-compose:

1. **fukuyoka_app** (Rust/Axum backend)
   - Listens on port 80 inside container
   - Simple API server with health check endpoint
   - Accessed via `/api/*` routes through nginx proxy

2. **fukuyoka_frontend** (Hugo)
   - Runs Hugo server on port 1313
   - Serves static site with food diary posts
   - All posts stored as markdown in `frontend/content/posts/`

3. **fukuyoka_proxy** (nginx)
   - Main entry point on host port 51841
   - Routes `/api/*` to backend, everything else to frontend
   - Proxies `/photo/*` image requests to Cloudflare R2 (photo.fukuyoka.dev)
   - Includes security blocks for common exploit paths

4. **cloudflared**
   - Cloudflare Tunnel for external access
   - Uses TUNNEL_TOKEN from `.env` file

### Nginx Routing Logic
- `/akasha/*` → proxies to Hugo frontend with Basic Auth (admin panel)
- `/api/*` → proxies to `fukuyoka_app:80` (Rust backend)
- `/photo/*.{png,jpeg,jpg}` → proxies to `https://photo.fukuyoka.dev` (R2 bucket)
- `/*` → proxies to `fukuyoka_frontend:1313` (Hugo frontend)

### Hugo Site Structure
- Custom theme located at `frontend/themes/fukuyoka/`
- Posts use Hugo front matter with fields: title, date, tags, categories, thumbnail
- Images referenced as `/photo/filename.jpg` (served from R2 via nginx proxy)
- Base URL: https://www.fukuyoka.dev/

## Common Commands

### Docker Operations
```bash
# Build and start all services
make build
make up

# View logs
make logs

# Access container shell
make in

# Stop services
make down

# Clean up everything (including volumes)
make clean

# Check running containers
make ps
```

### Hugo Frontend
```bash
# Build static site
make hugo
# or
cd frontend && hugo

# Hugo runs in server mode in Docker by default
# Access frontend development at http://localhost:1313 (when running locally)
```

### Rust Backend
```bash
# Run backend (inside container or locally)
cargo run --release

# The backend runs on port 80 and provides:
# GET / - Returns "Hello, I am Fukuyoka"
# GET /health - Returns {"status": "ok"}
# POST /upload - Image upload (multipart/form-data)
# POST /sync - Sync images to R2 using AWS CLI
# GET /images - List uploaded images
```

## Environment Setup

Required `.env` file at repository root must contain:
- `DOMAIN` - Your domain name for Hugo baseURL
- `TUNNEL_TOKEN` - Cloudflare Tunnel authentication token
- `R2_ENDPOINT` - Cloudflare R2 endpoint URL (e.g., `https://<account_id>.r2.cloudflarestorage.com`)
- `R2_BUCKET` - R2 bucket name (default: `fukuyoka-photo`)
- `AWS_ACCESS_KEY_ID` - R2 access key for AWS CLI
- `AWS_SECRET_ACCESS_KEY` - R2 secret key for AWS CLI

## Development Notes

### Adding New Blog Posts
1. Create new markdown file in `frontend/content/posts/`
2. Include Hugo front matter with thumbnail pointing to `/photo/filename.jpg`
3. Upload images to the NAS location or R2 bucket (configured at `/ldisk/nas/fukuyoka-photo/photo` in compose.yml)

### Rust Backend Extension
- Main application logic in `src/main.rs`
- Uses Axum router with tokio async runtime
- Add new routes to the Router in the main function
- Backend accessed via `/api/*` prefix through nginx

### Custom Hugo Theme
- Theme files in `frontend/themes/fukuyoka/`
- Layouts use partial templates (head, header, footer)
- Custom image rendering with modal support (`frontend/static/js/image-modal.js`)

### Admin Panel (Akasha)
- Access at `/akasha/` with Basic Auth
- Static files in `frontend/static/akasha/`
- Features:
  - Image upload to `/tmp/data`
  - R2 sync button (uses AWS CLI `s3 sync`)
  - View uploaded images
- Basic Auth credentials configured in `nginx/.htpasswd`
- Generate password: `htpasswd -nb admin yourpassword > nginx/.htpasswd`
