use postgres::{Client, NoTls};
use std::env;

pub fn connect_db() -> Result<Client, Box<dyn std::error::Error>> {
    let client = Client::connect(
        &format!(
            "host=localhost user={} dbname={} password={}",
            env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string()),
            env::var("POSTGRES_DB").unwrap_or_else(|_| "postgres".to_string()),
            env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string())
        ),
        NoTls,
    )?;
    Ok(client)
}

pub fn initialize_db(posts_data: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect_db()?;
    enable_extensions(&mut client)?;
    create_table(&mut client)?;
    create_indexes(&mut client)?;
    let json_data = load_json(posts_data)?;
    for item in json_data {
        let title = item["title"].as_str().unwrap_or_default();
        let tokens = item["tokens"].as_str().unwrap_or_default();
        let embedding_str = item["embedding"].as_str().unwrap_or_default();
        insert_data(&mut client, title, tokens, embedding_str)?;
    }
    Ok(())
}

fn enable_extensions(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    client.batch_execute(
        "
        CREATE EXTENSION IF NOT EXISTS vector;
        CREATE EXTENSION IF NOT EXISTS pg_textsearch;
        ",
    )?;
    Ok(())
}
fn create_table(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    client.batch_execute(
        "
        CREATE TABLE documents (
            id SERIAL PRIMARY KEY,
            title TEXT,
            tokens TEXT, -- 形態素解析で単語をスペースで繋いだものを格納
            embeds vector(768)     -- ベクトルデータを格納
        );
        ",
    )?;
    Ok(())
}
fn create_indexes(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    client.batch_execute(
        "
        CREATE INDEX ON documents USING hnsw (embeds vector_cosine_ops);
        CREATE INDEX ON documents USING bm25 (tokens) WITH (text_config = 'simple');
        ",
    )?;
    Ok(())
}

fn insert_data(client: &mut Client, title: &str, tokens: &str, embedding_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.execute(
        "INSERT INTO documents (title, tokens, embeds) VALUES ($1, $2, $3::vector)",
        &[&title, &tokens, &embedding_str],
    )?;
    Ok(())
}

pub fn load_json(file_path: &str) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string(file_path)?;
    let json_data: Vec<serde_json::Value> = serde_json::from_str(&data)?;
    Ok(json_data)
}

