# rustを用いた食事記録ブログサイトの構築
## 目的
- 食事記録を投稿。(instagramみたいに)
- RustでAPI
- cloudflare tunnelを用いたセルフホスティング

## 起動手順
`template.env`から`.env`を作成。
```sh
make build 
make up
```
でAPIの起動ができます。

起動するコンテナは全部で4つ。
1. cloudflare tunnel : cloudflare経由でインターネット接続
2. nginx : basic認証や`photo.fukuyoka.dev`へのプロキシなど
3. hugo(Frontend描画) : 記事(html)の描画
4. api(Rust) : 検索APIの提供(adminのAPIも共用)

## 記事の埋め込み
検索機能のためには事前に記事をベクトル空間に落とし込んでおく必要があります。

```sh
make in
```
をして`fukuyoka_app`に入れます。このコンテナはAPIを提供しています。
以下は暗黙的にコンテナの中で作業します。

使い方を見たければ`-h`オプションを使います
```sh
cargo run --bin embed -- -h #もしくは --help
```
以下のようなのが出てきます。
```sh
Usage: embed [OPTIONS]

Options:
      --src <SRC>              Process a single markdown file
      --all                    Process all posts in the posts directory
  -o, --output <OUTPUT>        Output file path (default: <src-filename>.json or embeddings.json for --all)
      --posts-dir <POSTS_DIR>  Posts directory (default: frontend/content/posts) [default: frontend/content/posts]
  -h, --help                   Print help
```

1. 単体記事の埋め込み
```sh
cargo run --bin embed -- --src frontend/contents/posts/hoge.md ( -o hoge.json)
```
出力はデフォルト時で`hoge.json`。`-o`オプションで指定もできます。

2. 全記事の埋め込み
```sh
cargo run --bin embed -- --all
```
出力はデフォルトで`embeddings.json`。こちらも`-o`オプションで指定もできます。
