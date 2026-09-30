# MNIST Viewer（MCP デモ集）

「label 9 のデータを 5 枚ください」と日本語で頼むと、**AI エージェントが MCP ツールを自分で呼び出して** MNIST の手書き数字画像を表示するデモ。

同じデモを、エージェント（Claude / Codex）と実装言語（Python / Julia）を変えて 4 種類用意している。各ディレクトリだけで完結していて、互いに依存しない。

```
「label 9 のデータを 5 枚ください」
   ↓ エージェントが数字と枚数を読み取り、ツールを選ぶ
get_mnist_images({"label": 9, "count": 5})     ← MCP サーバ
   ↓ サマリ JSON + PNG 画像 5 枚
画像をチャットにギャラリーで表示し、回答する
```

## 4 つの版

| ディレクトリ | エージェント | 言語 | 画面 | ポート |
|---|---|---|---|---|
| [`web_claude/`](web_claude/) | Claude Haiku（Claude Agent SDK） | Python | Gradio | 7860 |
| [`web_codex/`](web_codex/) | Codex `gpt-6-luna`（`openai-codex` SDK） | Python | Gradio | 7861 |
| [`web_claude_julia/`](web_claude_julia/) | Claude Haiku（`claude` CLI） | Julia | HTML + JavaScript | 7862 |
| [`web_codex_julia/`](web_codex_julia/) | Codex `gpt-6-luna`（`codex exec` CLI） | Julia | HTML + JavaScript | 7863 |

ポートがすべて違うので、4 つを同時に起動して比べられる。

- **Python 版**は公式 SDK からエージェントを呼び、MCP サーバは FastMCP で書いてある。
- **Julia 版**は公式 SDK がないので CLI（`claude` / `codex exec`）を子プロセスとして起動し、出力の JSON を 1 行ずつ読む。MCP サーバは ModelContextProtocol.jl、Web サーバは HTTP.jl。
- Claude 版と Codex 版は、エージェント部分以外（MCP サーバ・画面）が同じ。

## 共通の準備

API キーは使わず、各社のサブスクリプションでログインして動かす。

| エージェント | ログイン | 確認 |
|---|---|---|
| Claude | `claude` を起動して `/login` | `claude auth status` が `"authMethod": "claude.ai"` |
| Codex | `codex login` | `codex login status` が `Logged in using ChatGPT` |

`ANTHROPIC_API_KEY` / `OPENAI_API_KEY` などが環境にあると API 課金が優先されうる。そのため、どの版も起動時にこれらを取り除いている。

必要なツール:

- Python 版: [uv](https://docs.astral.sh/uv/)
- Julia 版: [Julia](https://julialang.org/)、および `claude` または `codex` コマンド

## 起動

Python 版（`web_claude/` または `web_codex/` で実行）:

```sh
uv sync
uv run python app.py
```

Julia 版（`web_claude_julia/` または `web_codex_julia/` で実行）:

```sh
julia --project=. -e 'using Pkg; Pkg.instantiate()'   # 初回のみ
julia --project=. app.jl
```

ブラウザで上の表のポートを開く。初回だけ MNIST（訓練データ 60,000 枚、約 10 MB）を各ディレクトリの `data/` にダウンロードする。

## 試してみる

| 入力 | 動き |
|---|---|
| label 9 のデータを 5 枚ください | `get_mnist_images({"label": 9, "count": 5})` を 1 回呼ぶ |
| 5 と 7 を 3 枚ずつ | 数字ごとに 2 回呼ぶ |
| （続けて）同じ数字をあと 2 枚 | 会話を引き継ぎ、直前の数字で呼ぶ |
| 0 を 30 枚 | 上限の 16 枚で呼び、上限があることを伝える |
| 今日の天気は？ | MNIST と関係ないので、画面にエラーを表示 |

チャットには、エージェントが呼んだツールと引数、ツールの戻り値（サマリ JSON）、画像、回答が順に出る。

## MCP ツール `get_mnist_images`

4 つの版すべてで同じ仕様。

| 引数 | 型 | 既定値 | 説明 |
|---|---|---|---|
| `count` | integer | 4 | 返す枚数（1〜16） |
| `label` | integer \| null | null | 0〜9 を指定すると、その数字だけから選ぶ |
| `seed` | integer \| null | null | 乱数シード（同じ値なら同じ画像） |

戻り値の 1 件目はサマリ JSON、2 件目以降は PNG 画像（112×112、28×28 を 4 倍に拡大）。重複なしで選ぶ。

## テスト

```sh
uv run pytest                       # Python 版
julia --project=. test/runtests.jl  # Julia 版
```

どの版のテストも、MNIST のダウンロードやエージェントの呼び出しはしない（小さな偽データと偽のエージェントを使う）。

## 詳しくは

各版の構成やエージェントの設定は、それぞれの `README.md` を参照。
