# AGENTS.md

This file provides guidance for agentic coding assistants working on the Fukuyoka food diary blog.

## Build, Lint, and Test Commands

### Docker-based workflow (primary)
```bash
make build          # Build all Docker images
make up             # Start containers
make down           # Stop containers
make restart        # Restart app container
make restart-proxy  # Reload nginx config without full restart
make in             # Shell into fukuyoka_app container
make logs           # View container logs
make ps             # List running containers
make clean          # Remove containers, networks, volumes
```

### Inside the Rust container (run `make in` first)
```bash
cargo check                    # Type check without building
cargo build --release          # Build optimized binary
cargo run --release            # Run API server

# Testing
cargo test                     # Run all tests
cargo test <test_name>         # Run specific test (e.g., cargo test test_parse_frontmatter)
cargo test -- --nocapture      # Run tests with println! output visible
```

### CLI tools
- Use `clap` with `derive` feature for argument parsing
- Place binaries in `src/bin/<name>.rs` and register in Cargo.toml
- Follow `Args` struct pattern with `#[derive(Parser, Debug)]`
- Use `std::process::exit()` for CLI errors, never panic

### Hugo frontend
```bash
make hugo           # Build static site (cd frontend && hugo)
```

## Code Style Guidelines

### Imports and module organization
- Group imports: external crates → std → local modules
- Use `use crate_name::module::item;` format (not nested)
- Re-export public API through lib.rs when appropriate
- Example: `use axum::{routing::{get, post}, Json}; use std::path::Path;`

### Naming conventions
- Functions and variables: `snake_case` (e.g., `upload_images`, `file_path`)
- Types and structs: `PascalCase` (e.g., `Post`, `EmbeddingModel`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `UPLOAD_DIR`)
- Private internal items: no prefix (Rust default)
- Async functions: `async fn name(...)`

### Error handling
- Return `Result<T, Box<dyn std::error::Error>>` for fallible functions
- Use `?` operator for error propagation
- Prefer `map_err(|e| format!("...: {e}"))` for context over unwrap
- Match on errors with descriptive messages for user-facing endpoints
- Never panic in production code; return proper HTTP status codes instead

### Structs and types
- Use `pub` for all fields that need to be serialized/deserialized
- Derive `Debug` for all structs, `Clone` when useful
- Use `#[derive(Serialize, Deserialize)]` for JSON structures
- Keep structs in lib.rs if shared across modules, otherwise in module files

### Async and tokio
- Use `tokio::fs` for async file operations
- Mark async entry points with `#[tokio::main]`
- Always use `.await` on async calls
- Prefer async versions of operations in API handlers

### Axum handlers
- Route handlers take `Json<T>` for request bodies or path params via `AxumPath<T>`
- Return `Result<Response<Body>, StatusCode>` or `Json<serde_json::Value>`
- Use `axum::extract::{Path, Query, State}` for common extractors
- Example:
  ```rust
  pub async fn handler(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
      Json(json!({ "status": "ok" }))
  }
  ```

### External command execution
- Use `tokio::process::Command` for async command execution
- Check `.status.success()` for command results
- Use `String::from_utf8_lossy()` for stdout/stderr conversion
- Example: `Command::new("aws").args(["s3", "ls"]).output().await?`

### String formatting
- Use `format!()` for string interpolation (not concatenation)
- Use `{}` placeholder, only use `{:?}`/`{:#?}` for debug output
- For JSON responses, use `json!()` macro from serde_json

### Path handling
- Use `std::path::{Path, PathBuf}` for cross-platform paths
- Use `.to_string_lossy()` when converting to String (rarely needed)
- Always validate paths to prevent traversal attacks (check for "..", "/", "\\")

### Testing
- Place tests in module at bottom of file: `#[cfg(test)] mod tests { ... }`
- Use descriptive test names: `test_<function>_<scenario>`
- Test both success and error paths

### Comments and documentation
- Use `///` for public function documentation (rustdoc style)
- Prefer inline comments over separate lines when brief
- Keep comments in English unless context requires Japanese (consistency within file)
- Use `// TODO:` or `// FIXME:` for temporary markers
- Some files (e.g., admin.rs, embedding.rs) use Japanese comments for domain context

### Environment variables
- Use `std::env::var("KEY")` for required variables
- Use `.unwrap_or("default")` for optional variables
- Never log or commit sensitive values

### Code organization
- Main API logic in `src/main.rs`
- Feature modules: `src/admin.rs`, `src/embedding.rs`, `src/post.rs`
- CLI binaries: `src/bin/embed.rs`
- Shared types/exports in `src/lib.rs`
- Follow separation of concerns: parsing, business logic, and API layers distinct

### Security
- Validate all user inputs, especially file paths
- Use `unwrap()` only when you're certain it won't fail (test first)
- Never expose internal error details to API responses
- Set appropriate file size limits (currently 50MB via `DefaultBodyLimit`)
