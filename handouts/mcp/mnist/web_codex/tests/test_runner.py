import asyncio
import base64
import json
from types import SimpleNamespace

from openai_codex.generated.v2_all import (
    ItemCompletedNotification,
    ItemStartedNotification,
    TurnCompletedNotification,
)

from runner import MnistImage, parse_tool_result, run_turn, to_events

SUMMARY = json.dumps(
    {"count": 2, "filter_label": 9, "images": [
        {"position": 1, "index": 10, "label": 9},
        {"position": 2, "index": 20, "label": 9},
    ]}
)
IDS = {"threadId": "th-1", "turnId": "tu-1"}


def _image_block(data: bytes):
    return {"type": "image", "data": base64.b64encode(data).decode(), "mimeType": "image/png"}


def _mcp_item(status, result=None, error=None):
    return {"type": "mcpToolCall", "id": "c1", "server": "mnist", "tool": "get_mnist_images",
            "arguments": {"label": 9, "count": 2}, "status": status, "result": result, "error": error}


def _started(item):
    return SimpleNamespace(method="item/started", payload=ItemStartedNotification.model_validate(
        {"item": item, "startedAtMs": 0, **IDS}))


def _completed(item):
    return SimpleNamespace(method="item/completed", payload=ItemCompletedNotification.model_validate(
        {"item": item, "completedAtMs": 0, **IDS}))


def _turn_completed(status="completed", error=None):
    return SimpleNamespace(method="turn/completed", payload=TurnCompletedNotification.model_validate(
        {"threadId": "th-1", "turn": {"id": "tu-1", "items": [], "status": status,
                                      "durationMs": 2500, "error": error}}))


def _collect(notifications):
    async def source():
        for n in notifications:
            yield n

    async def run():
        return [e async for e in to_events(source(), "th-1")]

    return asyncio.run(run())


def test_parse_tool_result_pairs_images_with_summary():
    text, images = parse_tool_result(
        [{"type": "text", "text": SUMMARY}, _image_block(b"png1"), _image_block(b"png2")]
    )
    assert text == SUMMARY
    assert images == [MnistImage(b"png1", 9, 10), MnistImage(b"png2", 9, 20)]


def test_parse_tool_result_without_summary():
    assert parse_tool_result([_image_block(b"png")]) == ("", [MnistImage(b"png", None, None)])


def test_streams_tool_use_images_and_text():
    result = {"content": [{"type": "text", "text": SUMMARY}, _image_block(b"a"), _image_block(b"b")]}
    events = _collect([
        _started(_mcp_item("inProgress")),
        _completed(_mcp_item("completed", result=result)),
        _completed({"type": "agentMessage", "id": "m1", "text": "9 を 2 枚取得しました。"}),
        _turn_completed(),
    ])
    assert events[0] == ("tool_use", "get_mnist_images", {"label": 9, "count": 2})
    kind, text, images, is_error = events[1]
    assert (kind, text, is_error) == ("tool_result", SUMMARY, False)
    assert [img.png for img in images] == [b"a", b"b"]
    assert events[2:] == [
        ("text", "9 を 2 枚取得しました。"),
        ("done", {"session_id": "th-1", "duration_s": 2.5}),
    ]


def test_tool_error_is_reported():
    events = _collect([_completed(_mcp_item("failed", error={"message": "count は 1〜16"}))])
    assert events == [("tool_result", "count は 1〜16", [], True)]


def test_failed_turn_is_reported():
    events = _collect([_turn_completed("failed", {"message": "usage limit"})])
    assert events[0] == ("error", "usage limit")
    assert events[1][0] == "done"


def test_exception_is_reported_as_error_event():
    class BrokenCodex:
        async def __aenter__(self):
            raise RuntimeError("codex login が必要です")

        async def __aexit__(self, *exc):
            return False

    async def run():
        return [e async for e in run_turn("q", None, codex_factory=BrokenCodex)]

    assert asyncio.run(run()) == [("error", "codex login が必要です")]
