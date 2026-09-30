# MNIST Viewer（Claude 版）

「label 9 のデータを 5 枚ください」と日本語で頼むと、**Claude Haiku が MCP ツールを自分で呼び出して** MNIST の手書き数字画像を表示するデモ。画面は Gradio で作っている。

このディレクトリだけで完結している。MCP サーバも Python で書き直してあり、一つ上の Julia 版（`../server.jl`）は使わない。同じ構成の Codex 版が `../web_codex/` にある。

```
「label 9 のデータを 5 枚ください」
   ↓ Claude Haiku が数字と枚数を読み取り、ツールを選ぶ
get_mnist_images({"label": 9, "count": 5})     ← MCP サーバ (server.py)
   ↓ サマリ JSON + PNG 画像 5 枚
画像をチャットにギャラリーで表示し、「9 の画像を 5 枚取得しました…」と答える
```

## 準備: Claude のサブスクリプションでログイン

API キーは使わず、Claude Pro/Max のサブスクリプションで動かす。

```sh
claude               # 起動して /login でサブスクリプションアカウントにログイン
claude auth status   # "authMethod": "claude.ai" になっていれば OK
```

`ANTHROPIC_API_KEY` / `ANTHROPIC_AUTH_TOKEN` が環境にあると API 課金が優先される。そのため `app.py` は起動時にこれらを取り除く。

## 起動

```sh
uv sync
uv run python download_data.py   # 初回だけ: MNIST を data/ にダウンロードする
uv run python app.py
```

ブラウザで http://localhost:7860 を開く。Codex 版は 7861 番で起動するので、両方を並べて比べられる。

`app.py` を起動する前に、必ず `download_data.py` を実行して MNIST（訓練データ 60,000 枚、約 10 MB）を `data/` にダウンロードしておくこと。`data/` に無いまま起動すると、MCP サーバ `server.py` が起動時にダウンロードを始め、回線が遅いと接続確認に間に合わず「MCP サーバ mnist に接続できません (status=failed)」になる。ダウンロード済みなら再実行しても何もしない。

## 試してみる

| 入力 | 動き |
|---|---|
| label 9 のデータを 5 枚ください | `get_mnist_images({"label": 9, "count": 5})` を 1 回呼ぶ |
| 5 と 7 を 3 枚ずつ | 数字ごとに 2 回呼ぶ |
| （続けて）同じ数字をあと 2 枚 | 会話を引き継ぎ、直前の数字で呼ぶ |
| 0 を 30 枚 | 上限の 16 枚で呼び、上限があることを伝える |
| 今日の天気は？ | MNIST と関係ないので、画面にエラーを表示 |

チャットには、Claude が呼んだツールと引数、ツールの戻り値（サマリ JSON）、画像、回答が順に出る。

## 構成

| ファイル | 役割 |
|---|---|
| `mnist_data.py` | MNIST のダウンロード、IDX 形式の読み込み、サンプリング、PNG 化 |
| `download_data.py` | MNIST を `data/` にダウンロードするスクリプト（`app.py` の前に実行） |
| `server.py` | MCP サーバ（FastMCP, stdio）。ツール `get_mnist_images` |
| `mnist_agent.py` | `ClaudeAgentOptions` の組み立て、システムプロンプト |
| `runner.py` | `query()` のメッセージを画面用のイベントに変える |
| `app.py` | Gradio のチャット画面 |

### MCP サーバ `get_mnist_images`

| 引数 | 型 | 既定値 | 説明 |
|---|---|---|---|
| `count` | integer | 4 | 返す枚数（1〜16） |
| `label` | integer \| null | null | 0〜9 を指定すると、その数字だけから選ぶ |
| `seed` | integer \| null | null | 乱数シード（同じ値なら同じ画像） |

戻り値の 1 件目はサマリ JSON（`{"count", "filter_label", "images": [{"position", "index", "label"}]}`）、2 件目以降は `position` 順に並んだ PNG（112×112、28×28 を 4 倍に拡大）。重複なしで選ぶ。

### エージェントの設定

```python
ClaudeAgentOptions(
    model="claude-haiku-4-5",
    mcp_servers={"mnist": {"command": sys.executable, "args": ["server.py"]}},
    strict_mcp_config=True,                          # ~/.claude の MCP 設定は混ぜない
    allowed_tools=["mcp__mnist__get_mnist_images"],  # 確認なしで実行してよいツール
    disallowed_tools=["Bash", "Read", "Write", ..., "ToolSearch"],
    setting_sources=[],                              # ユーザー/プロジェクトの設定・CLAUDE.md を読まない
    system_prompt=SYSTEM_PROMPT,
)
```

- **画像の受け取り方:** MCP の画像は、Agent SDK の `ToolResultBlock.content` に `{"type": "image", "source": {"type": "base64", ...}}` として届く。`runner.py` がこれをデコードし、サマリ JSON の `label` と `index` を対応づけてキャプションにする。
- **MNIST と関係のない依頼:** システムプロンプトで、ツールを呼ばずに `OFF_TOPIC` とだけ返すよう指示している。画面はこの目印を見つけるとエラーを表示する。

## テスト

```sh
uv run pytest
```

テストは MNIST のダウンロードも Claude の呼び出しもしない（小さな偽データと偽の `query()` を使う）。
