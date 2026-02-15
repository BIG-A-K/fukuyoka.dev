-- 拡張機能の有効化
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_textsearch;

-- テーブル作成
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    content TEXT,
    jp_tokenized_content TEXT, -- Sudachiで分割した単語をスペースで繋いだものを格納
    embedding vector(768)     -- ベクトルデータを格納
);

-- ベクトル検索用インデックス (HNSW)
CREATE INDEX ON documents USING hnsw (embedding vector_cosine_ops);

-- キーワード検索用インデックス (BM25)
CREATE INDEX ON documents USING bm25 (jp_tokenized_content) WITH (text_config = 'simple');