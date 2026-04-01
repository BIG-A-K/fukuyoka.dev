# 食事記録ブログサイトの構築
本リポジトリは検索機能付きのブログリポジトリです。[こちら](https://www.fukuyoka.dev)にて公開しています。
以下の機能を提供します。

1. cloudflare tunnel : cloudflareを経由したインターネットのゲート(グローバルIPを非公開のままホスティングできます)
2. nginx : APIサーバへのプロキシ、もしくはフロントエンドへのプロキシ
3. hugo(Frontend描画) : 記事(html)の描画
4. api(Rust) : 検索APIの提供
5. PostgreSQL : 文章検索用のDB


## 起動手順
本リポジトリはMakefileによってDockerや形態素解析などの操作をラッパーしています。

1. `template.env`から`.env`を作成: cloudflare tunnelとPostgreSQLの接続に必要です
```sh
cp template.env .env
```
を実行した後、`.env`の中身を適切に編集してください。

2. 文章の埋め込み・形態素解析を実行: 検索に使えるように文章を処理してjson(`./posts.json`)で保管します。
```sh
make dev-build
# make build # 本番環境の場合
make prepare
```
3. コンテナの立ち上げ
```sh
make dev
# make up #本番環境の場合
```
4. コンテナの終了
```sh
make down
```
5. (optional) 他にも開発向けのターゲットを用意しています
```sh
make (help)
```

# ネットワーク構成図

```mermaid
graph LR
    subgraph Internet
        User["👤 ユーザー"]
        CF["☁️ Cloudflare CDN<br/>www.fukuyoka.dev"]
        R2["📦 Cloudflare R2<br/>photo.fukuyoka.dev"]
    end

    subgraph Docker Network - fukuyoka 192.168.100.0/24
        cloudflared["cloudflared<br/>(Cloudflare Tunnel)"]

        subgraph fukuyoka_proxy - nginx :80
            direction TB
            route_api["/api/* → Public API"]
            route_photo["/photo/*.jpg → R2 proxy"]
            route_front["/* → Hugo"]
        end

        app["fukuyoka_app<br/>Rust/Axum<br/>:80"]
        frontend["fukuyoka_frontend<br/>Hugo Server<br/>:1313"]
        db["fukuyoka_db<br/>PostgreSQL + pgvector<br/>192.168.100.11"]
    end

    User -->|HTTPS| CF
    CF -->|Tunnel| cloudflared
    cloudflared -->|HTTP :80| fukuyoka_proxy

    route_api -->|"proxy_pass /"| app
    route_photo -->|"proxy_pass HTTPS"| R2
    route_front -->|"proxy_pass :1313"| frontend

    app -.->|embedding/search| db

    style CF fill:#f6a821,color:#000
    style R2 fill:#f6a821,color:#000
    style cloudflared fill:#f48120,color:#fff
    style app fill:#b7410e,color:#fff
    style frontend fill:#ff75a0,color:#000
    style db fill:#336791,color:#fff
```

## Nginxのルーティング詳細

| パス | 認証 | 転送先 | 説明 |
|------|------|--------|------|
| `/api/*` | なし | `fukuyoka_app /` | 公開API (embedding, search) |
| `/photo/*.{png,jpg}` | なし | `photo.fukuyoka.dev` (R2) | 画像プロキシ |
| `/*` | なし | `fukuyoka_frontend:1313` | ブログフロントエンド |
| `/wp-*` 等 | - | 418 I'm a teapot | 攻撃パスブロック: 攻撃者にジョークで対抗 |
