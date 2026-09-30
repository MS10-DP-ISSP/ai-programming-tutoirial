"""MNIST ビューアのエージェント (Codex + server.py の MCP サーバ) の設定。

認証は Codex CLI と同じ ChatGPT サブスクリプション (codex login 済み) を使う。
API キーはここでは一切扱わない。
"""

import json
import os
import sys
import tomllib
from pathlib import Path
from typing import MutableMapping

from openai_codex import ApprovalMode, CodexConfig, Sandbox

MODEL = "gpt-6-luna"  # 速くて安いモデル (Claude 版の Haiku に相当)
EFFORT = "low"

HERE = Path(__file__).resolve().parent
# 同じディレクトリの MCP サーバ
SERVER_PATH = HERE / "server.py"
SERVER_NAME = "mnist"
TOOL_NAME = "get_mnist_images"

USER_CONFIG_PATH = Path(os.environ.get("CODEX_HOME", Path.home() / ".codex")) / "config.toml"

# これらが環境にあると API キー課金になりうるため、起動時に取り除く
API_CREDENTIAL_VARS = ("OPENAI_API_KEY", "CODEX_API_KEY")

# MNIST ツール以外は使わせない。シェル・ブラウザ・画像生成などの機能を切り、
# ~/.codex/hooks.json などユーザーの hooks も動かさない
DISABLED_FEATURES = [
    "shell_tool",
    "unified_exec",
    "apps",
    "plugins",
    "browser_use",
    "in_app_browser",
    "computer_use",
    "image_generation",
    "multi_agent",
    "goals",
    "hooks",
]

# MNIST と関係のない依頼のときに Codex に返させる目印。UI はこれを見てエラーを出す
OFF_TOPIC_MARKER = "OFF_TOPIC"

SYSTEM_PROMPT = f"""\
あなたは MNIST (手書き数字 0〜9 のデータセット) の画像を見せるアシスタントです。
画像を求められたら、必ず {TOOL_NAME} ツールで取得してから答えてください。

- 「9 の画像を 5 枚」のように数字と枚数を指定されたら label=9, count=5 で呼ぶこと。
- 数字の指定がなければ label は省略する。枚数の指定がなければ count は省略する (4 枚)。
- count は 1〜16。17 枚以上を求められたら 16 枚にして、上限が 16 枚であることを伝える。
- 「5 と 7 を 2 枚ずつ」のように複数の数字を求められたら、数字ごとにツールを呼ぶこと。
- 画像は画面に左から position の順で表示されるので、回答では画像を描写し直さない。
  何の数字を何枚取得したかを日本語で 1〜2 文で伝え、気づいたこと (字の癖など) があれば短く添える。
- MNIST の画像と関係のない依頼には、ツールを呼ばず、{OFF_TOPIC_MARKER} とだけ返すこと (他の文字は付けない)。
"""


def is_off_topic(reply: str) -> bool:
    """Codex の回答が「MNIST と関係ない」の目印かどうか。"""
    return reply.strip().startswith(OFF_TOPIC_MARKER)


def use_subscription_auth(environ: MutableMapping[str, str] = os.environ) -> list[str]:
    """API キー系の環境変数を取り除き、サブスクリプション認証を使わせる。

    取り除いた変数名のリストを返す。
    """
    removed = [name for name in API_CREDENTIAL_VARS if name in environ]
    for name in removed:
        del environ[name]
    return removed


def user_mcp_servers(config_path: Path = USER_CONFIG_PATH) -> list[str]:
    """~/.codex/config.toml に登録されている MCP サーバ名。"""
    try:
        config = tomllib.loads(config_path.read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return []
    return list(config.get("mcp_servers", {}))


def config_overrides(config_path: Path = USER_CONFIG_PATH) -> tuple[str, ...]:
    """codex app-server に渡す -c の値。

    Claude 版の strict_mcp_config に当たる設定が codex には無いので、
    ユーザーの MCP サーバを 1 つずつ無効にしてから mnist サーバだけを足す。
    """
    others = [name for name in user_mcp_servers(config_path) if name != SERVER_NAME]
    return (
        # -c のパスは "." で区切るだけでクォートを解釈しないので、名前はそのまま書く
        *(f"mcp_servers.{name}.enabled=false" for name in others),
        # server.py はこのプロジェクトの Python (fastmcp, numpy 入り) で起動する
        f"mcp_servers.{SERVER_NAME}.command={json.dumps(sys.executable)}",
        f"mcp_servers.{SERVER_NAME}.args={json.dumps([str(SERVER_PATH)])}",
        # mnist のツールは読むだけなので、確認なしで実行してよい
        f'mcp_servers.{SERVER_NAME}.default_tools_approval_mode="approve"',
        'web_search="disabled"',
        *(f"features.{name}=false" for name in DISABLED_FEATURES),
    )


def build_config() -> CodexConfig:
    """codex app-server の起動設定を返す。"""
    return CodexConfig(config_overrides=config_overrides(), cwd=str(HERE))


def thread_options() -> dict:
    """thread_start / thread_resume に渡す引数。"""
    return {
        "model": MODEL,
        "developer_instructions": SYSTEM_PROMPT,
        "sandbox": Sandbox.read_only,
        # MCP ツールは上で承認済み。それ以外の権限の要求はすべて断る
        "approval_mode": ApprovalMode.deny_all,
        "cwd": str(HERE),
    }
