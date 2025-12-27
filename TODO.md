# TODO

## admin画面関連
- [ ] `/akasha`で入れるようにする(basic認証)(**nginx**)
- [ ] 画像投稿機能(スマホやPCからGUIであげたい)(**jsでpost**)
- [ ] R2と同期する機能(**ボタンポチ〜！**)

## 検索機能
- [ ] `/search`でfrontendに検索ページを描画(hugoを使う)
- [ ] `/result`にredirect。ここでapiの返り値を描画し、検索結果を出す

## embedding
- [x] jsonを作る機能
- [ ] json->parquet
- [ ] parquet->index

## API
- [ ] `/api/search`：textがpostされるので、embedding(Candle multilingual-e5-base)とvss(faiss)を実行する
- [ ] `/api/post`：画像投稿機能。`/tmp/post`に画像を配置。exiftoolでデータもクリーニングする。
- [ ] `/api/sync`：R2に画像を同期する(aws コマンドを実行する)
