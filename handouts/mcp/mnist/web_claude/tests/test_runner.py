import asyncio
import base64
import json

from claude_agent_sdk import (
    AssistantMessage,
    ResultMessage,
    SystemMessage,
    TextBlock,
    ToolResultBlock,
    ToolUseBlock,
    UserMessage,
)

from runner import MnistImage, parse_tool_result, run_turn

SUMMARY = json.dumps(
    {"count": 2, "filter_label": 9, "images": [
        {"position": 1, "index": 10, "label": 9},
        {"position": 2, "index": 20, "label": 9},
    ]}
)


def _image_block(data: bytes):
    return {"type": "image", "source": {"type": "base64", "media_type": "image/png",
                                        "data": base64.b64encode(data).decode()}}


def _result_message(session_id="sess-1"):
    return ResultMessage(subtype="success", duration_ms=2_500, duration_api_ms=1,
                         is_error=False, num_turns=2, session_id=session_id)


def _collect(fake_query):
    async def run():
        return [e async for e in run_turn("q", options=None, query_fn=fake_query)]
    return asyncio.run(run())


def test_parse_tool_result_pairs_images_with_summary():
    text, images = parse_tool_result(
        [{"type": "text", "text": SUMMARY}, _image_block(b"png1"), _image_block(b"png2")]
    )
    assert text == SUMMARY
    assert images == [MnistImage(b"png1", 9, 10), MnistImage(b"png2", 9, 20)]


def test_parse_tool_result_without_summary():
    text, images = parse_tool_result([_image_block(b"png")])
    assert text == ""
    assert images == [MnistImage(b"png", None, None)]
    assert parse_tool_result("boom") == ("boom", [])


def test_streams_tool_use_images_and_text():
    async def fake_query(*, prompt, options):
        yield AssistantMessage(content=[ToolUseBlock(
            id="t1", name="mcp__mnist__get_mnist_images", input={"label": 9, "count": 2})], model="m")
        yield UserMessage(content=[ToolResultBlock(
            tool_use_id="t1",
            content=[{"type": "text", "text": SUMMARY}, _image_block(b"a"), _image_block(b"b")],
            is_error=False)])
        yield AssistantMessage(content=[TextBlock(text="9 を 2 枚取得しました。")], model="m")
        yield _result_message("sess-9")

    events = _collect(fake_query)
    assert events[0] == ("tool_use", "mcp__mnist__get_mnist_images", {"label": 9, "count": 2})
    kind, text, images, is_error = events[1]
    assert (kind, text, is_error) == ("tool_result", SUMMARY, False)
    assert [img.png for img in images] == [b"a", b"b"]
    assert events[2:] == [
        ("text", "9 を 2 枚取得しました。"),
        ("done", {"session_id": "sess-9", "num_turns": 2, "duration_s": 2.5}),
    ]


def test_failed_mcp_server_is_reported():
    async def fake_query(*, prompt, options):
        yield SystemMessage(subtype="init", data={"mcp_servers": [{"name": "mnist", "status": "failed"}]})
        yield _result_message()

    kind, message = _collect(fake_query)[0]
    assert kind == "error" and "mnist" in message


def test_exception_is_reported_as_error_event():
    async def broken_query(*, prompt, options):
        raise RuntimeError("boom")
        yield  # ジェネレータにするためのダミー

    assert _collect(broken_query) == [("error", "boom")]
