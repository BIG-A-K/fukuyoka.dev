# Fukuyoka 技術仕様書

## プロジェクト概要

Fukuyoka（幸者のふくよか日記）は、フードダイアリーブログサイトです。

- **プロジェクト名**: Fukuyoka
- **サイトタイトル**: 幸者のふくよか日記
- **公開URL**: https://www.fukuyoka.dev/
- **著者**: kuroda yukijya
- **連絡先**: yukijya@big-a-k.com

## システムアーキテクチャ

### 技術スタック

| コンポーネント | 技術 | バージョン |
|--------------|------|----------|
| バックエンド | Rust (Axum) | 0.7 |
| ランタイム | Tokio | 1.x |
| フロントエンド | Hugo | latest |
| リバースプロキシ | nginx | latest |
| トンネル | Cloudflare Tunnel | latest |
| 画像ホスティング | Cloudflare R2 | - |
| コンテナ | Docker Compose | - |

### サービス構成

#### 1. fukuyoka_app (Rustバックエンド)

**役割**: APIサーバー

**技術詳細**:
- フレームワーク: Axum 0.7
- 非同期ランタイム: Tokio (マルチスレッド)
- ポート: 80 (コンテナ内部)
- イメージ: fukuyoka_app:latest (カスタムビルド)

**エンドポイント**:
- `GET /` - ヘルスチェック用メッセージを返す
  - レスポンス: `"Hello, I am Fukuyoka"`
- `GET /health` - ヘルスチェックJSON
  - レスポンス: `{"status": "ok"}`
- その他 - 404フォールバック
  - レスポンス: `"API : 404 Not Found"`

**Dockerfile構成**:
- ベースイメージ: `rust:latest`
- タイムゾーン: Asia/Tokyo
- 起動コマンド: `cargo run --release`

**依存関係** (Cargo.toml):
```toml
axum = "0.7"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

#### 2. fukuyoka_frontend (Hugoフロントエンド)

**役割**: 静的サイト生成・配信

**技術詳細**:
- イメージ: `docker.io/hugomods/hugo:latest`
- ポート: 1313 (コンテナ内部)
- 動作モード: サーバーモード
- 環境変数: `HUGO_ENV=production`

**Hugo設定** (hugo.toml):
```toml
baseURL = 'https://www.fukuyoka.dev/'
languageCode = 'ja'
title = '幸者のふくよか日記'
theme = 'fukuyoka'
```

**起動コマンド**:
```bash
server --bind 0.0.0.0 --port 1313 --baseURL https://${DOMAIN} --appendPort=false
```

**ディレクトリ構成**:
- `content/posts/` - ブログ記事（Markdown形式）
- `themes/fukuyoka/` - カスタムテーマ
- `static/` - 静的アセット
- `layouts/` - レイアウトテンプレート
- `public/` - ビルド成果物

#### 3. fukuyoka_proxy (nginxリバースプロキシ)

**役割**: ルーティング、画像プロキシ、セキュリティフィルタ

**技術詳細**:
- イメージ: `nginx:latest`
- ポート: 51841 (ホスト) → 80 (コンテナ)
- IPアドレス: 192.168.100.10 (固定)
- ヘルスチェックポート: 51841

**ルーティングルール**:

| パス | プロキシ先 | 説明 |
|------|----------|------|
| `/api/*` | `fukuyoka_app:80/` | Rust APIへのプロキシ（パス書き換え） |
| `/photo/*.{png,jpeg,jpg}` | `https://photo.fukuyoka.dev` | R2画像バケットへのプロキシ |
| `/*` | `fukuyoka_frontend:1313` | Hugoサイトへのプロキシ |

**セキュリティブロック**:
- WordPress関連パス: `/wp-admin`, `/wp-includes`, `/wp-content`, `/wordpress`, `/xmlrpc.php`, `/wp-login.php`
- 攻撃的パス: `/admin`, `/phpmyadmin`, `/pma`, `/mysql`, `/db`, `/database`, `/sql`, `/.env`, `/.git`
- ブロック時レスポンス: 444 (接続切断)

**gzip圧縮設定**:
- 有効化: ON
- 最小サイズ: 1000バイト
- 対象タイプ: text/plain, text/css, application/json, application/javascript, text/xml, application/xml

**画像プロキシ設定**:
```nginx
location ~* ^/photo/.*\.(png|jpe?g)$ {
  rewrite ^/photo/(.*)$ /$1 break;
  proxy_pass https://photo.fukuyoka.dev;
  proxy_set_header Host photo.fukuyoka.dev;
  proxy_ssl_server_name on;
}
```

**ヘルスチェック**:
- コマンド: `nginx -t`
- インターバル: 30秒
- タイムアウト: 10秒
- リトライ: 3回

#### 4. cloudflared (Cloudflare Tunnel)

**役割**: 外部アクセス提供

**技術詳細**:
- イメージ: `docker.io/cloudflare/cloudflared:latest`
- 認証: 環境変数 `TUNNEL_TOKEN`
- 起動コマンド: `tunnel --no-autoupdate run --token ${TUNNEL_TOKEN}`

**依存関係**:
- `fukuyoka_proxy` のヘルスチェック通過が必須

## ネットワーク構成

**Docker Network**: `fukuyoka`
- ドライバ: bridge
- サブネット: 192.168.100.0/24

**接続フロー**:
```
Internet
  ↓
Cloudflare Tunnel (cloudflared)
  ↓
nginx (192.168.100.10:80)
  ├→ /api/* → fukuyoka_app:80 (Rust)
  ├→ /photo/*.{png,jpg,jpeg} → https://photo.fukuyoka.dev (R2)
  └→ /* → fukuyoka_frontend:1313 (Hugo)
```

## ストレージ構成

**画像ストレージ**:
- ローカルNASマウント: `/ldisk/nas/fukuyoka-photo/photo` → `/photos` (読み取り専用)
- Cloudflare R2バケット: `photo.fukuyoka.dev`
- プロキシ経由でR2から配信（nginx経由）

**ボリュームマウント**:
- `../:/app` (fukuyoka_app) - Rustアプリケーション
- `../frontend:/src` (fukuyoka_frontend) - Hugoサイト
- `../nginx/nginx.conf:/etc/nginx/nginx.conf` (fukuyoka_proxy) - nginx設定

## 環境変数

**必須環境変数** (`.env`ファイル):
- `DOMAIN` - ドメイン名（例: `www.fukuyoka.dev`）
- `TUNNEL_TOKEN` - Cloudflare Tunnel認証トークン

## 開発・運用コマンド

### Docker操作

```bash
# ビルド
make build

# 起動（デタッチモード）
make up

# 停止
make down

# コンテナシェルアクセス
make in

# ログ表示
make logs

# コンテナ一覧
make ps

# クリーンアップ（ボリューム含む）
make clean
```

### Hugo操作

```bash
# 静的サイトビルド
make hugo
# または
cd frontend && hugo
```

## デプロイメント

**デプロイ方式**: Docker Compose + Cloudflare Tunnel

**起動順序**:
1. `fukuyoka_app` (Rustバックエンド)
2. `fukuyoka_frontend` (Hugoフロントエンド)
3. `fukuyoka_proxy` (nginx) - 上記2つのサービス起動後
4. `cloudflared` - nginx のヘルスチェック通過後

**再起動ポリシー**: `unless-stopped` (全サービス)

## ブログ記事追加フロー

1. `frontend/content/posts/` に Markdown ファイル作成
2. Hugo Front Matter を記述:
   ```yaml
   ---
   title: "記事タイトル"
   date: 2024-01-01T00:00:00+09:00
   tags: ["タグ1", "タグ2"]
   categories: ["カテゴリ"]
   thumbnail: "/photo/filename.jpg"
   ---
   ```
3. 画像を R2 バケットまたは NAS にアップロード
4. Hugo が自動的に再ビルド（サーバーモード）

## セキュリティ考慮事項

1. **攻撃パスのブロック**: nginx で WordPress や phpMyAdmin などの攻撃対象パスを事前ブロック
2. **読み取り専用マウント**: 画像ディレクトリを `:ro` でマウント
3. **ヘルスチェック**: nginx の設定検証を定期実行
4. **Cloudflare Tunnel**: 外部公開時にオリジンIPを隠蔽
5. **gzip圧縮**: 通信量削減とパフォーマンス向上

## パフォーマンス最適化

- **gzip圧縮**: テキストベースのコンテンツを圧縮
- **静的サイト生成**: Hugo による高速な静的HTML配信
- **CDN統合**: Cloudflare経由でのコンテンツ配信
- **画像プロキシ**: R2経由での効率的な画像配信

## 今後の拡張可能性

- Rust APIの機能拡張（コメント機能、検索APIなど）
- R2画像の最適化（WebP変換、リサイズなど）
- Hugo テーマのカスタマイズ
- 分析機能の追加
