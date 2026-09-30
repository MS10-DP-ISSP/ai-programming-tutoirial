"""Codex のターンの通知を、画面で使いやすいイベントに変換する非同期ジェネレータ。

Gradio は async ジェネレータのハンドラをそのまま回せるので、スレッドは使わない。
イベントの形は web_claude/runner.py とそろえてある。

イベント:
    ("tool_use", name, input)                     Codex がツールを呼んだ
    ("tool_result", text, images, is_error)       ツールの戻り値。images は MNIST 画像のリスト
    ("text", str)                                 Codex の回答
    ("done", {session_id, duration_s})            session_id は Codex のスレッド ID
    ("error", message)
"""

import base64
import json
from dataclasses import dataclass

from openai_codex import AsyncCodex
from openai_codex.generated.v2_all import (
    AgentMessageThreadItem,
    ItemCompletedNotification,
    ItemStartedNotification,
    McpToolCallThreadItem,
    TurnCompletedNotification,
    TurnStatus,
)

from mnist_agent import EFFORT, build_config, thread_options


@dataclass(frozen=True)
class MnistImage:
    png: bytes
    label: int | None  # サマリ JSON から分かれば入る
    index: int | None


def parse_tool_result(content: list) -> tuple[str, list[MnistImage]]:
    """MCP ツールの content を (テキスト, 画像のリスト) に分ける。

    get_mnist_images は 1 件目にサマリ JSON、続いて PNG を position 順に返す。
    サマリの images[i] と i 番目の PNG を対応づけて label と index を付ける。
    """
    texts, pngs = [], []
    for block in content or []:
        if block.get("type") == "text":
            texts.append(block.get("text", ""))
        elif block.get("type") == "image":
            pngs.append(base64.b64decode(block["data"]))
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


async def to_events(notifications, thread_id: str):
    """ターンの通知 (Notification の async イテレータ) をイベントに変える。"""
    async for notification in notifications:
        payload = notification.payload
        item = getattr(getattr(payload, "item", None), "root", None)
        if isinstance(payload, ItemStartedNotification) and isinstance(item, McpToolCallThreadItem):
            yield ("tool_use", item.tool, item.arguments)
        elif isinstance(payload, ItemCompletedNotification):
            if isinstance(item, McpToolCallThreadItem):
                if item.error is not None:
                    yield ("tool_result", item.error.message, [], True)
                else:
                    text, images = parse_tool_result(item.result.content if item.result else [])
                    yield ("tool_result", text, images, item.status.value == "failed")
            elif isinstance(item, AgentMessageThreadItem) and item.text:
                yield ("text", item.text)
        elif isinstance(payload, TurnCompletedNotification):
            turn = payload.turn
            if turn.status == TurnStatus.failed:
                yield ("error", turn.error.message if turn.error else "ターンが失敗しました")
            yield (
                "done",
                {"session_id": thread_id, "duration_s": (turn.duration_ms or 0) / 1000},
            )


async def run_turn(prompt: str, thread_id: str | None, codex_factory=None):
    """1 ターン分のエージェント実行をイベントとして流す。例外も error イベントにする。

    thread_id を渡すと、そのスレッドの会話を引き継ぐ。
    """
    codex_factory = codex_factory or (lambda: AsyncCodex(build_config()))
    try:
        async with codex_factory() as codex:
            if thread_id:
                thread = await codex.thread_resume(thread_id, **thread_options())
            else:
                thread = await codex.thread_start(**thread_options())
            turn = await thread.turn(prompt, effort=EFFORT)
            async for event in to_events(turn.stream(), thread.id):
                yield event
    except Exception as exc:  # noqa: BLE001 - UI に必ず通知する
        yield ("error", str(exc))
