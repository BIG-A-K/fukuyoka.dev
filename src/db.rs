use tokio_postgres::{Client, NoTls};
use std::env;

pub async fn connect_db() -> Result<Client, Box<dyn std::error::Error>> {
    let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
    let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "postgres".to_string());
    let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string());

    let mut config = tokio_postgres::Config::new();
    config.host("db");
    config.user(&user);
    config.dbname(&db);
    config.password(&password);

    let (client, connection) = config.connect(NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {e}");
        }
    });

    Ok(client)
}

pub async fn initialize_db(posts_data: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = connect_db().await?;
    println!("Connected to database");
    enable_extensions(&client).await?;
    println!("Database extensions enabled");
    create_table(&client).await?;
    println!("Database table created");
    create_indexes(&client).await?;
    println!("Database indexes created");
    let json_data = load_json(posts_data)?;
    for item in json_data {
        let title = item["title"].as_str().unwrap_or_default();
        let url = item["url"].as_str().unwrap_or_default();
        let thumbnail = item["thumbnail"].as_str().unwrap_or_default();
        let tokens = item["tokens"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let embedding_str = item["embedding"]
            .as_array()
            .map(|arr| {
                format!(
                    "[{}]",
                    arr.iter()
                        .filter_map(|v| v.as_f64())
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .unwrap_or_default();
        insert_data(&client, title, url, thumbnail, &tokens, &embedding_str).await?;
    }
    Ok(())
}

async fn enable_extensions(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client
        .batch_execute(
            "
        CREATE EXTENSION IF NOT EXISTS vector;
        CREATE EXTENSION IF NOT EXISTS pg_textsearch;
        ",
        )
        .await?;
    Ok(())
}

async fn create_table(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client
        .batch_execute(
            "
        CREATE TABLE IF NOT EXISTS documents (
            id SERIAL PRIMARY KEY,
            title TEXT,
            url TEXT,
            thumbnail TEXT,
            tokens TEXT,
            embeds vector(768)
        );
        ",
        )
        .await?;
    Ok(())
}

async fn create_indexes(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    client
        .batch_execute(
            "
        CREATE INDEX IF NOT EXISTS documents_embeds_idx ON documents USING hnsw (embeds vector_cosine_ops);
        CREATE INDEX IF NOT EXISTS documents_tokens_idx ON documents USING bm25 (tokens) WITH (text_config = 'simple');
        ",
        )
        .await?;
    Ok(())
}

async fn insert_data(
    client: &Client,
    title: &str,
    url: &str,
    thumbnail: &str,
    tokens: &str,
    embedding_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "INSERT INTO documents (title, url, thumbnail, tokens, embeds) VALUES ($1, $2, $3, $4, $5::text::vector)",
            &[&title, &url, &thumbnail, &tokens, &embedding_str],
        )
        .await?;
    Ok(())
}

pub fn load_json(file_path: &str) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(file_path)?;
    let json_data: Vec<serde_json::Value> = serde_json::from_str(&data)?;
    Ok(json_data)
}
