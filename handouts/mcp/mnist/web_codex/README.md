# MNIST Viewer（Codex 版）

「label 9 のデータを 5 枚ください」と日本語で頼むと、**Codex が MCP ツールを自分で呼び出して** MNIST の手書き数字画像を表示するデモ。画面は Gradio で作っている。

このディレクトリだけで完結している。`../web_claude/`（Claude 版）と同じ MCP サーバ・画面で、エージェント部分だけを OpenAI 公式の Python SDK [`openai-codex`](https://github.com/openai/codex/tree/main/sdk/python) に置き換えた。

```
「label 9 のデータを 5 枚ください」
   ↓ Codex (gpt-6-luna) が数字と枚数を読み取り、ツールを選ぶ
get_mnist_images({"label": 9, "count": 5})     ← MCP サーバ (server.py)
   ↓ サマリ JSON + PNG 画像 5 枚
画像をチャットにギャラリーで表示し、「9 の画像を 5 枚取得しました」と答える
```

## 準備: ChatGPT のサブスクリプションでログイン

API キーは使わず、ChatGPT Plus/Pro などのサブスクリプションで動かす。

```sh
codex login          # ブラウザで ChatGPT アカウントにログイン
codex login status   # "Logged in using ChatGPT" になっていれば OK
```

`openai-codex` は codex 本体（`openai-codex-cli-bin`）を同梱しているので、`codex` コマンドを別途入れなくても動く。認証情報は `~/.codex/` のものを使う。

`OPENAI_API_KEY` / `CODEX_API_KEY` が環境にあると API キー課金になりうる。そのため `app.py` は起動時にこれらを取り除く。

## 起動

```sh
uv sync
uv run python download_data.py   # 初回だけ: MNIST を data/ にダウンロードする
uv run python app.py
```

ブラウザで http://localhost:7861 を開く（Claude 版は 7860 番）。

`app.py` を起動する前に、必ず `download_data.py` を実行して MNIST（訓練データ 60,000 枚、約 10 MB）を `data/` にダウンロードしておくこと。`data/` に無いまま起動すると、MCP サーバ `server.py` が起動時にダウンロードを始め、回線が遅いと接続確認に間に合わず MCP サーバに接続できなくなる。ダウンロード済みなら再実行しても何もしない。

## 試してみる

| 入力 | 動き |
|---|---|
| label 9 のデータを 5 枚ください | `get_mnist_images({"label": 9, "count": 5})` を 1 回呼ぶ |
| 5 と 7 を 3 枚ずつ | 数字ごとに 2 回呼ぶ |
| （続けて）同じ数字をあと 2 枚 | スレッドを引き継ぎ、直前の数字で呼ぶ |
| 今日の天気は？ | MNIST と関係ないので、画面にエラーを表示 |

## 構成

| ファイル | 役割 |
|---|---|
| `mnist_data.py` | MNIST のダウンロード、IDX 形式の読み込み、サンプリング、PNG 化（Claude 版と同じ） |
| `download_data.py` | MNIST を `data/` にダウンロードするスクリプト（`app.py` の前に実行。Claude 版と同じ） |
| `server.py` | MCP サーバ（FastMCP, stdio）。ツール `get_mnist_images`（Claude 版と同じ） |
| `mnist_agent.py` | codex の設定（`-c` の上書き）、スレッドの設定、システムプロンプト |
| `runner.py` | ターンの通知を画面用のイベントに変える |
| `app.py` | Gradio のチャット画面 |

### エージェントの設定

```python
async with AsyncCodex(CodexConfig(config_overrides=config_overrides())) as codex:
    thread = await codex.thread_start(          # 2 ターン目以降は thread_resume(thread_id, ...)
        model="gpt-6-luna",
        developer_instructions=SYSTEM_PROMPT,
        sandbox=Sandbox.read_only,
        approval_mode=ApprovalMode.deny_all,
    )
    turn = await thread.turn(prompt, effort="low")
    async for notification in turn.stream():  # item/started, item/completed, turn/completed ...
        ...
```

`config_overrides()` は `codex app-server` に `-c` で次の値を渡す。

| 上書き | 理由 |
|---|---|
| `mcp_servers.mnist.command` / `args` | `server.py` をこのプロジェクトの Python で起動する |
| `mcp_servers.mnist.default_tools_approval_mode="approve"` | MNIST のツールは読むだけなので確認なしで実行する。これがないと、承認できずにツール呼び出しが失敗する |
| `mcp_servers.<ユーザーのサーバ>.enabled=false` | `~/.codex/config.toml` の MCP サーバを混ぜない |
| `features.shell_tool=false` など | シェル・ブラウザ・画像生成・hooks などを使わせない |
| `web_search="disabled"` | Web 検索を使わせない |

Claude 版との違い:

- **設定の切り離し方:** Agent SDK には `strict_mcp_config` と `setting_sources=[]` がある。`codex app-server` にはユーザー設定を読まないオプションがないので、ユーザーの MCP サーバと機能を 1 つずつ無効にしている。
- **会話の引き継ぎ:** Claude 版は `resume=session_id`、Codex 版は `thread_resume(thread_id)`。スレッドは `~/.codex/sessions/` に保存される。
- **画像の受け取り方:** `McpToolCallThreadItem.result.content` に、MCP の形のまま `{"type": "image", "data": ..., "mimeType": "image/png"}` として届く。

## テスト

```sh
uv run pytest
```

テストは MNIST のダウンロードも Codex の呼び出しもしない（小さな偽データと、SDK の型で組み立てた通知を使う）。
