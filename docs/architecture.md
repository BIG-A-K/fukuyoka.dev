# Architecture

これは食事ブログをオンプレミスのサーバーでcloudflare tunnel経由で公開するリポジトリです。

## 構成

- `frontend/`: hugoで作成された静的サイトコンテンツを管理しています。
- `nginx/`: confファイルを配置しており、proxyを管理しています。
  - `https://www.fukuyoka.dev`: 通常のコンテンツ
  - `https://photo.fukuyoka.dev`: 画像を保存するR2ストレージ
  - `https://www.fukuyoka.dev/api`: Rust製のAPI
  - `https://www.fukuyoka.dev/akasha`: 管理者画面(basic認証)
  - `https://www.fukuyoka.dev/api/akasha/*`: 管理者用API(basic認証)
- `src/`: Axumで作成されるRust製のAPIを保存しています。
  - `lib.rs`: モジュールのエクスポート
  - `admin.rs`: アドミン用の実装
  - `search.rs`: 検索エンドポイントの実装（ハイブリッド検索）
  - `embedding.rs`: テキストの埋め込みの実装
  - `post.rs`: Markdown記事のパース処理
  - `db.rs`: PostgreSQL接続・初期化処理
  - `morphology.rs`: 形態素解析（Lindera）の実装
  - `bin/api.rs`: APIサーバーのエントリポイント
  - `bin/prepare.rs`: 記事の埋め込み・形態素解析を行うCLIツール
- `compose.yml`: Docker Compose設定（5サービス）

## データフロー

### 1. 記事投稿フロー

```
[admin画面] → POST /api/akasha/upload → 画像を/tmp/dataに保存
                                    ↓
                              exiftoolでEXIF削除
                                    ↓
                         POST /api/akasha/push → R2にアップロード
                                    ↓
                         Hugo記事を作成・編集 → frontend/content/posts/*.md
                                    ↓
                         cargo run --bin prepare -- --all
                                    ↓
                               posts.jsonを生成
                                    ↓
                         API起動時にPostgreSQLへロード
```

### 2. 検索フロー（ハイブリッド検索）

```
ユーザーが検索クエリを入力
         ↓
POST /api/search { "text": "query" }
         ↓
[Rust API] 並列実行:
  ├── "query: " + 入力テキスト をE5モデルでベクトル化
  │   └── PostgreSQL pgvectorでコサイン類似度検索
  └── Linderaで形態素解析 → トークン化
      └── PostgreSQL pg_textsearchでBM25検索
         ↓
RRF (Reciprocal Rank Fusion) で結果を統合
         ↓
上位5件を返却
```

### 3. 画像配信フロー

```
ブラウザが/photo/hoge.jpgをリクエスト
         ↓
nginxが/photo/*をphoto.fukuyoka.devにリバースプロキシ
         ↓
Cloudflare R2から画像を配信
```

## APIエンドポイント

### 公開API（認証不要）

| メソッド | パス | 説明 |
|---------|------|------|
| GET | `/` | ヘルスチェック代わりのメッセージ |
| GET | `/health` | ヘルスチェック（`{ "status": "ok" }`） |
| POST | `/search` | 記事を検索（ハイブリッド検索） |

#### POST /search レスポンス例

```json
{
  "status": true,
  "results": [
    {
      "title": "記事タイトル",
      "url": "/posts/filename/",
      "thumbnail": "画像URL"
    }
  ],
  "msg": null
}
```

### 管理者API（Basic認証が必要）

| メソッド | パス | 説明 |
|---------|------|------|
| POST | `/api/akasha/upload` | 画像をローカルにアップロード（multipart/form-data） |
| GET | `/api/akasha/images` | アップロード済み画像一覧を取得 |
| GET | `/api/akasha/local-image/{filename}` | ローカル画像を表示（プレビュー用） |
| GET | `/api/akasha/diff` | ローカルとR2の画像差分を確認 |
| POST | `/api/akasha/push` | ローカル画像をR2に同期 |

## 検索アーキテクチャ

### ハイブリッド検索

検索は2つの手法を組み合わせています：

1. **ベクトル類似度検索（pgvector）**: 意味的な類似性を検出
2. **全文検索（BM25）**: キーワードベースの検索

これらを **RRF (Reciprocal Rank Fusion)** で統合し、より精度の高い検索結果を提供します。

### 使用技術

| コンポーネント | 技術 |
|--------------|------|
| 埋め込みモデル | `intfloat/multilingual-e5-base` (768次元) |
| MLフレームワーク | Candle（Rust製） |
| 形態素解析 | Lindera（IPAdic辞書） |
| ベクトルDB | PostgreSQL + pgvector |
| 全文検索 | PostgreSQL + pg_textsearch (BM25) |

### E5モデルの特殊仕様

E5モデルは入力にプレフィックスが必要です：

- **記事埋め込み時**: `"passage: " + タイトル + " " + 本文`
- **検索クエリ時**: `"query: " + ユーザー入力`

### データベース構造

```sql
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title TEXT,
    url TEXT,
    thumbnail TEXT,
    tokens TEXT,           -- 形態素解析結果（スペース区切り）
    embeds vector(768)     -- 埋め込みベクトル
);

-- インデックス
CREATE INDEX posts_embeds_idx ON posts USING hnsw (embeds vector_cosine_ops);
CREATE INDEX posts_tokens_idx ON posts USING bm25 (tokens) WITH (text_config = 'simple');
```

## 環境変数

`.env`ファイルで以下を設定してください：

| 変数名 | 説明 | 必須 |
|--------|------|------|
| `TUNNEL_TOKEN` | Cloudflare Tunnelのトークン | ○ |
| `DOMAIN` | ドメイン名（デフォルト: www.fukuyoka.dev） | △ |
| `R2_ENDPOINT` | Cloudflare R2のエンドポイントURL | ○（画像機能を使う場合） |
| `R2_BUCKET` | R2バケット名（デフォルト: fukuyoka-photo） | △ |
| `AWS_ACCESS_KEY_ID` | R2アクセスキー | ○ |
| `AWS_SECRET_ACCESS_KEY` | R2シークレットキー | ○ |
| `POSTGRES_USER` | PostgreSQLユーザー名 | ○ |
| `POSTGRES_PASSWORD` | PostgreSQLパスワード | ○ |
| `POSTGRES_DB` | PostgreSQLデータベース名 | ○ |
| `POSTGRES_TABLE` | 検索用テーブル名 | ○ |

## 技術選定の理由

### Rust + Axum

- **高速性**: 検索APIの低レイテンシーが重要なため
- **メモリ安全性**: セグメンテーションフォルトがない
- **並行処理**: async/awaitによる非同期処理が簡潔

### Candle

- **Rust製**: Python（PyTorch等）への依存を排除
- **軽量**: 組み込みに適したサイズ
- **safetensors**: 安全なモデルフォーマットをサポート

### Lindera

- **Rust製**: ネイティブなパフォーマンス
- **IPAdic対応**: 日本語形態素解析の標準的な辞書

### PostgreSQL + pgvector

- **統合性**: リレーショナルデータとベクトル検索を1つのDBで管理
- **HNSWインデックス**: 高速な近似最近傍探索
- **pg_textsearch**: BM25による全文検索

### Hugo

- **ビルド速度**: 数百ページでも秒単位でビルド
- **静的出力**: セキュリティリスクが少ない
- **Markdown対応**: 記事作成が簡単

### Cloudflare Tunnel

- **固定IP不要**: 動的IPでも公開可能
- **自動SSL**: HTTPS化が設定不要
- **セキュリティ**: オリジンサーバーのIPを隠せる

### Cloudflare R2

- **S3互換**: AWS CLIで操作可能
- **低コスト**: 無料枠が大きい
- **CDN統合**: Cloudflareのエッジキャッシュを活用

## セキュリティ設計

### パストラバーサル対策

`admin.rs`でファイル名を検証：

```rust
if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
    return Err(StatusCode::BAD_REQUEST);
}
```

### Basic認証

nginxで以下に適用：

- `/akasha` - 管理者画面
- `/api/akasha/*` - 管理者用API

認証情報は `admin/.htpasswd` に保存されます。

### 攻撃パスブロック

nginxでWordPressなどの一般的な攻撃パスを418 I'm a teapot で返却：

```nginx
location ~* ^/(wp-admin|wp-includes|wordpress|xmlrpc\.php) {
    return 418;
}
```

### EXIF情報削除

アップロード時に自動的にexiftoolでEXIFメタデータを削除し、位置情報などの漏洩を防止。

## ディレクトリ構造

```
fukuyoka/
├── admin/                      # 管理者画面（HTML/CSS/JS）
│   ├── index.html
│   └── .htpasswd               # Basic認証（.gitignore対象）
├── compose.yml                 # Docker Compose設定（5サービス）
├── compose.override.yml        # 開発用オーバーライド
├── docker/
│   ├── Dockerfile             # Rustアプリのビルド環境
│   └── db/
│       └── Dockerfile         # PostgreSQL + pgvector
├── docs/
│   └── architecture.md        # このファイル
├── frontend/                   # Hugo静的サイト
│   ├── content/
│   │   └── posts/             # Markdown記事（TOML frontmatter）
│   ├── hugo.toml              # Hugo設定
│   └── themes/
│       └── fukuyoka/          # カスタムテーマ
├── nginx/
│   ├── nginx.conf             # リバースプロキシ設定
│   └── teapot.html            # 418エラーページ
├── scripts/
│   └── pg_vector/             # PostgreSQL初期化スクリプト
├── src/                        # Rustソースコード
│   ├── lib.rs                 # モジュールエクスポート
│   ├── admin.rs               # 画像アップロード・R2同期
│   ├── search.rs              # ハイブリッド検索ロジック
│   ├── embedding.rs           # E5モデルラッパー
│   ├── post.rs                # Markdownパース処理
│   ├── db.rs                  # PostgreSQL接続・初期化
│   ├── morphology.rs          # 形態素解析（Lindera）
│   └── bin/
│       ├── api.rs             # APIサーバー
│       └── prepare.rs         # 記事前処理CLI
├── .env                       # 環境変数（.gitignore対象）
├── Cargo.toml                 # Rust依存関係
├── posts.json                 # 前処理済み記事データ（.gitignore対象）
├── Makefile                   # 開発用コマンド
├── README.md                  # 基本セットアップ手順
└── template.env               # .envのテンプレート
```

## Docker仮想環境

本リポジトリではmakeコマンドによってdocker composeをラッパーしています。

```bash
# imageのビルド
make build

# 起動（デタッチモード）
make up

# 停止
make down

# APIコンテナにアクセス
make in

# ログ表示
make logs

# コンテナ一覧
make ps

# クリーンアップ（ボリューム含む）
make clean
```

### コンテナ構成

| コンテナ名 | 役割 | ポート | IPアドレス | 依存 |
|-----------|------|--------|-----------|------|
| fukuyoka_app | Rust API | 80（内部） | 動的 | db |
| fukuyoka_frontend | Hugoサーバー | 1313（内部） | 動的 | - |
| fukuyoka_proxy | nginx | 51841（ホスト） | 192.168.100.10 | app, frontend |
| cloudflared | Cloudflare Tunnel | - | 動的 | proxy |
| db | PostgreSQL + pgvector | 5432（内部） | 192.168.100.11 | - |

ネットワークは`192.168.100.0/24`のブリッジネットワークを使用。
