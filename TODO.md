# TODO
## admin画面関連
以下は全てRust(axum)で実装する
- [ ] `/akasha`で入れるようにする(basic認証)
- [ ] 画像投稿機能(スマホやPCからGUIであげたい)
- [ ] R2と同期する機能(aws コマンド)

## 検索機能
以下もRustで。
- [ ] `/search`でfrontendに検索ページを描画(hugoを使う)
- [ ] `/api/search`にtextがpostされるので、embedding(Candle multilingual-e5-base)とvss(faiss)を実行する
- [ ] `/result`にredirect。ここでapiの返り値を描画し、検索結果を出す

## embedding
- [x] jsonを作る機能
- [ ] json->parquet
- [ ] parquet->index

