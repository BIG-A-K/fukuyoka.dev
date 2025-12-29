# TODO

## admin画面関連
### front
- [ ] `/akasha`で入れるようにする(basic認証)(**nginx**)
- [ ] 画像投稿機能(スマホやPCからGUIであげたい)(**jsでpost**)
- [ ] R2と同期する機能(**ボタンポチ〜！**)
### API
- [ ] `/api/post`：画像投稿機能。`/tmp/post`に画像を配置。exiftoolでデータもクリーニングする。
- [ ] `/api/sync`：R2に画像を同期する(aws コマンドを実行する)

## 検索機能
### front
- [ ] `/search`でfrontendに検索ページを描画(hugoを使う)
- [ ] `/result`にredirect。ここでapiの返り値を描画し、検索結果を出す
### API
- [ ] `/api/embedding`で埋め込み
  - [ ] `/api/search`で検索結果のJsonを取得


## embedding(事前準備)
- [ ] scripts以下で実装しているPythonコードをRustに置き換える。