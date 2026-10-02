# ai-programming-tutoirial

## DeckTape でスライドを PDF に出力する

[DeckTape](https://github.com/astefanutti/decktape) をインストールします。

```sh
npm install -g decktape
```

スライドのプレビューサーバーを起動し、ブラウザーでスライドを開けることを確認してから、リポジトリのルートで次のコマンドを実行します。
URL のポート番号は、プレビューサーバーが表示するものに合わせてください。

```sh
decktape http://localhost:5952/slides/slide.html output.pdf
```

生成した `output.pdf` は、コマンドを実行したディレクトリに保存されます。

### Chrome が見つからない場合

`Could not find Chrome` と表示される場合は、インストール済みの Google Chrome を `--chrome-path` で指定します。
macOS では次のコマンドで出力できます。

```sh
decktape \
  --chrome-path "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  http://localhost:5952/slides/slide.html output.pdf
```
