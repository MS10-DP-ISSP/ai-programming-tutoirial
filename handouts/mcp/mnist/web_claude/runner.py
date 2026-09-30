"""query() のメッセージを、画面で使いやすいイベントに変換する非同期ジェネレータ。

Gradio は async ジェネレータのハンドラをそのまま回せるので、スレッドは使わない。

イベント:
    ("tool_use", name, input)                     Claude がツールを呼んだ
    ("tool_result", text, images, is_error)       ツールの戻り値。images は MNIST 画像のリスト
    ("text", str)                                 Claude の回答
    ("done", {session_id, num_turns, duration_s})
    ("error", message)
"""

import base64
import json
from dataclasses import dataclass

from claude_agent_sdk import (
    AssistantMessage,
    ResultMessage,
    SystemMessage,
    TextBlock,
    ToolResultBlock,
    ToolUseBlock,
    UserMessage,
    query,
)


@dataclass(frozen=True)
class MnistImage:
    png: bytes
    label: int | None  # サマリ JSON から分かれば入る
    index: int | None


def parse_tool_result(content) -> tuple[str, list[MnistImage]]:
    """ToolResultBlock.content を (テキスト, 画像のリスト) に分ける。

    get_mnist_images は 1 件目にサマリ JSON、続いて PNG を position 順に返す。
    サマリの images[i] と i 番目の PNG を対応づけて label と index を付ける。
    """
    if isinstance(content, str):
        return content, []
    texts, pngs = [], []
    for block in content or []:
        if block.get("type") == "text":
            texts.append(block.get("text", ""))
        elif block.get("type") == "image":
            pngs.append(base64.b64decode(block["source"]["data"]))
    text = "\n".join(texts)
    try:
        meta = json.loads(text)["images"]
    except (json.JSONDecodeError, KeyError, TypeError):
        meta = []
    images = [
        MnistImage(
            png=png,
            label=meta[i].get("label") if i < len(meta) else None,
            index=meta[i].get("index") if i < len(meta) else None,
        )
        for i, png in enumerate(pngs)
    ]
    return text, images


async def run_turn(prompt, options, query_fn=query):
    """1 ターン分のエージェント実行をイベントとして流す。例外も error イベントにする。"""
    try:
        async for message in query_fn(prompt=prompt, options=options):
            if isinstance(message, SystemMessage) and message.subtype == "init":
                for server in message.data.get("mcp_servers", []):
                    if server.get("status") in ("failed", "needs-auth"):
                        yield (
                            "error",
                            f"MCP サーバ {server.get('name')} に接続できません "
                            f"(status={server.get('status')})",
                        )
            elif isinstance(message, AssistantMessage):
                for block in message.content:
                    if isinstance(block, TextBlock):
                        yield ("text", block.text)
                    elif isinstance(block, ToolUseBlock):
                        yield ("tool_use", block.name, block.input)
            elif isinstance(message, UserMessage) and isinstance(message.content, list):
                for block in message.content:
                    if isinstance(block, ToolResultBlock):
                        text, images = parse_tool_result(block.content)
                        yield ("tool_result", text, images, bool(block.is_error))
            elif isinstance(message, ResultMessage):
                yield (
                    "done",
                    {
                        "session_id": message.session_id,
                        "num_turns": message.num_turns,
                        "duration_s": message.duration_ms / 1000,
                    },
                )
    except Exception as exc:  # noqa: BLE001 - UI に必ず通知する
        yield ("error", str(exc))
