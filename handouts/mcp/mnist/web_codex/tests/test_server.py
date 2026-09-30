import asyncio
import json

import numpy as np
from fastmcp import Client

import mnist_data
import server


def _call(monkeypatch, args):
    labels = np.repeat(np.arange(10, dtype=np.uint8), 3)
    images = np.zeros((30, 28, 28), dtype=np.uint8)
    # 実データのダウンロードをせずに済むよう、小さな偽データを使う
    monkeypatch.setattr(mnist_data, "load", lambda: (images, labels))

    async def run():
        async with Client(server.mcp) as client:
            return await client.call_tool("get_mnist_images", args, raise_on_error=False)

    return asyncio.run(run())


def test_returns_summary_then_png_images(monkeypatch):
    result = _call(monkeypatch, {"count": 3, "label": 9, "seed": 1})
    assert not result.is_error
    summary = json.loads(result.content[0].text)
    assert summary["count"] == 3
    assert summary["filter_label"] == 9
    assert [img["label"] for img in summary["images"]] == [9, 9, 9]
    assert [img["position"] for img in summary["images"]] == [1, 2, 3]
    assert [c.type for c in result.content[1:]] == ["image"] * 3
    assert result.content[1].mime_type == "image/png"


def test_bad_arguments_are_returned_as_tool_error(monkeypatch):
    result = _call(monkeypatch, {"count": 50})
    assert result.is_error
    assert "1〜16" in result.content[0].text
