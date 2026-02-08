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
  - `main.rs`: エンドポイントの管理
  - `admin.rs`: アドミン用の実装
  - `search.rs`: 検索エンドポイントの実装
  - `embedding.rs`: テキストの埋め込みの実装
  - `post.rs`: Markdown記事のパース処理
  - `bin/embed.rs`: ブログ記事を埋め込みするCLIツール
- `docker/`: dockerコンテナ関連のファイルディレクトリ
  - `compose.yml`: ここにあります。

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
                         cargo run --bin embed -- --all
                                    ↓
                              embeddings.jsonを生成
```

### 2. 検索フロー

```
ユーザーが検索クエリを入力
         ↓
POST /api/search { "text": "query" }
         ↓
[Rust API] "query: " + 入力テキスト をE5モデルでベクトル化
         ↓
embeddings.json内の記事ベクトルとコサイン類似度を計算
         ↓
上位10件をスコア付きで返却
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
| POST | `/embedding` | テキストをベクトル化（リクエスト: `{ "text": "..." }`） |
| POST | `/search` | 記事を検索（リクエスト: `{ "text": "..." }`） |

#### POST /search レスポンス例

```json
{
  "results": [
    {
      "title": "記事タイトル",
      "url": "/posts/filename/",
      "thumbnail": "画像URL",
      "score": 0.9234
    }
  ]
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

## 埋め込み・検索のアーキテクチャ

### モデル

- **使用モデル**: `intfloat/multilingual-e5-base`
- **フレームワーク**: Candle（Rust製MLフレームワーク）
- **ベクトル次元**: 768次元
- **類似度計算**: コサイン類似度

### E5モデルの特殊仕様

E5モデルは入力にプレフィックスが必要です：

- **記事埋め込み時**: `"passage: " + タイトル + " " + 本文`
- **検索クエリ時**: `"query: " + ユーザー入力`

このプレフィックスにより、検索意図と記事内容の意味的な距離を適切に測定できます。

### 検索インデックス

- **ファイル**: `embeddings.json`
- **構造**: `Vec<IndexedPost>` - 各記事のメタデータと768次元ベクトル
- **更新**: 記事追加・変更時に `embed --all` で再生成が必要

## 環境変数

`.env`ファイルで以下を設定してください：

| 変数名 | 説明 | 必須 |
|--------|------|------|
| `TUNNEL_TOKEN` | Cloudflare Tunnelのトークン | ○ |
| `R2_ENDPOINT` | Cloudflare R2のエンドポイントURL | ○（画像機能を使う場合） |
| `R2_BUCKET` | R2バケット名（デフォルト: fukuyoka-photo） | △ |
| `AWS_ACCESS_KEY_ID` | R2アクセスキー | ○ |
| `AWS_SECRET_ACCESS_KEY` | R2シークレットキー | ○ |
| `DOMAIN` | ドメイン名（デフォルト: www.fukuyoka.dev） | △ |

## 技術選定の理由

### Rust + Axum

- **高速性**: 検索APIの低レイテンシーが重要なため
- **メモリ安全性**: セグメンテーションフォルトがない
- **並行処理**: async/awaitによる非同期処理が簡潔

### Candle

- **Rust製**: Python（PyTorch等）への依存を排除
- **軽量**: 組み込みに適したサイズ
- **safetensors**: 安全なモデルフォーマットをサポート

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

認証情報は `.htpasswd` に保存されます。

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
│   └── index.html
├── docker/
│   ├── compose.yml            # Docker Compose設定（4サービス）
│   └── Dockerfile             # Rustアプリのビルド環境
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
├── src/                        # Rustソースコード
│   ├── main.rs                # APIサーバーエントリポイント
│   ├── admin.rs               # 画像アップロード・R2同期
│   ├── search.rs              # ベクトル検索ロジック
│   ├── embedding.rs           # E5モデルラッパー
│   ├── post.rs                # Markdownパース処理
│   ├── lib.rs                 # ライブラリエクスポート
│   └── bin/
│       └── embed.rs           # 埋め込み生成CLI
├── .env                       # 環境変数（.gitignore対象）
├── .htpasswd                  # Basic認証（.gitignore対象）
├── Cargo.toml                 # Rust依存関係
├── embeddings.json            # 検索インデックス（.gitignore対象）
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

| コンテナ名 | 役割 | ポート | 依存 |
|-----------|------|--------|------|
| fukuyoka_app | Rust API | 80（内部） | - |
| fukuyoka_frontend | Hugoサーバー | 1313（内部） | - |
| fukuyoka_proxy | nginx | 51841（ホスト） | app, frontend |
| cloudflared | Cloudflare Tunnel | - | proxy |

ネットワークは`192.168.100.0/24`のブリッジネットワークを使用。
