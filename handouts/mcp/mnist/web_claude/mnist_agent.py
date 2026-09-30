"""MNIST ビューアのエージェント (Claude Haiku + server.py の MCP サーバ) の設定。

認証は Claude Code と同じサブスクリプション (claude で /login 済み) を使う。
API キーはここでは一切扱わない。
"""

import os
import sys
from pathlib import Path
from typing import MutableMapping

from claude_agent_sdk import ClaudeAgentOptions

MODEL = "claude-haiku-4-5"

# 同じディレクトリの MCP サーバ
SERVER_PATH = Path(__file__).resolve().parent / "server.py"
TOOL_NAME = "mcp__mnist__get_mnist_images"

# これらが環境にあると API キー課金が優先されるため、起動時に取り除く
API_CREDENTIAL_VARS = ("ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN")

# MNIST ツール以外は使わせない
BANNED_TOOLS = [
    "Bash",
    "Read",
    "Write",
    "Edit",
    "NotebookEdit",
    "Glob",
    "Grep",
    "WebFetch",
    "WebSearch",
    # ツールが1つだけなので、ToolSearch で探す往復を省く (定義を最初から読み込ませる)
    "ToolSearch",
]

# MNIST と関係のない依頼のときに Claude に返させる目印。UI はこれを見てエラーを出す
OFF_TOPIC_MARKER = "OFF_TOPIC"

SYSTEM_PROMPT = f"""\
あなたは MNIST (手書き数字 0〜9 のデータセット) の画像を見せるアシスタントです。
画像を求められたら、必ず get_mnist_images ツールで取得してから答えてください。

- 「9 の画像を 5 枚」のように数字と枚数を指定されたら label=9, count=5 で呼ぶこと。
- 数字の指定がなければ label は省略する。枚数の指定がなければ count は省略する (4 枚)。
- count は 1〜16。17 枚以上を求められたら 16 枚にして、上限が 16 枚であることを伝える。
- 「5 と 7 を 2 枚ずつ」のように複数の数字を求められたら、数字ごとにツールを呼ぶこと。
- 画像は画面に左から position の順で表示されるので、回答では画像を描写し直さない。
  何の数字を何枚取得したかを日本語で 1〜2 文で伝え、気づいたこと (字の癖など) があれば短く添える。
- MNIST の画像と関係のない依頼には、ツールを呼ばず、{OFF_TOPIC_MARKER} とだけ返すこと (他の文字は付けない)。
"""


def is_off_topic(reply: str) -> bool:
    """Claude の回答が「MNIST と関係ない」の目印かどうか。"""
    return reply.strip().startswith(OFF_TOPIC_MARKER)


def use_subscription_auth(environ: MutableMapping[str, str] = os.environ) -> list[str]:
    """API キー系の環境変数を取り除き、サブスクリプション認証を使わせる。

    取り除いた変数名のリストを返す。
    """
    removed = [name for name in API_CREDENTIAL_VARS if name in environ]
    for name in removed:
        del environ[name]
    return removed


def build_options(resume: str | None = None) -> ClaudeAgentOptions:
    """MNIST エージェントの設定を返す。resume でセッションを継続する。"""
    return ClaudeAgentOptions(
        system_prompt=SYSTEM_PROMPT,
        model=MODEL,
        # server.py はこのプロジェクトの Python (fastmcp, numpy 入り) で起動する
        mcp_servers={"mnist": {"command": sys.executable, "args": [str(SERVER_PATH)]}},
        strict_mcp_config=True,
        allowed_tools=[TOOL_NAME],
        disallowed_tools=list(BANNED_TOOLS),
        cwd=Path(__file__).parent,
        resume=resume,
        setting_sources=[],
    )
