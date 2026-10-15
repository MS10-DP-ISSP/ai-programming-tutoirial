# ai-programming-tutoirial

## Description

この Web サイトは [生成AIプログラミング講習会](https://indico.nifs.ac.jp/event/105/) に対応する講義資料をまとめたページである．

## Handouts

- [Slide](slides/slide.qmd)
  - [Appendix](slides/settings_json.qmd)
- [Preparation](handouts/preparation.qmd)

## ローカル環境で資料を閲覧

Quarto を [ここから](https://quarto.org/docs/download/) インストールします．
これにより `quarto` コマンドが使えるようになります．

ターミナルを開いて下記を実行します．

```sh
$ git clone https://github.com/MS10-DP-ISSP/ai-programming-tutoirial.git
$ cd ai-programming-tutoirial
$ quarto preview
```

静的ファイルとして書き出す場合は `quarto render` を実行します．出力は `_site/` に保存されます．

## DeckTape でスライドを PDF に出力する

[DeckTape](https://github.com/astefanutti/decktape) をインストールします。

```sh
npm install -g decktape
```

スライドのプレビューサーバーを `quarto preview --port 5952` で起動し、ブラウザーでスライドを開けることを確認してから、リポジトリのルートで次のコマンドを実行します。
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
