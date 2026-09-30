"""MNIST の手書き数字画像を返す MCP サーバ (stdio)。

ツール get_mnist_images は、1 件目にサマリ JSON、2 件目以降に PNG 画像を返す。
起動: uv run python server.py (普通は mnist_agent.py から子プロセスとして起動される)
"""

import json

from fastmcp import FastMCP
from fastmcp.exceptions import ToolError
from fastmcp.utilities.types import Image

import mnist_data

# 28x28 のままだと Claude にも画面にも小さすぎるので拡大して返す
PNG_SCALE = 4

mcp = FastMCP("mnist")


@mcp.tool
def get_mnist_images(
    count: int = 4, label: int | None = None, seed: int | None = None
) -> list:
    """MNIST の手書き数字画像を PNG で返す。

    Args:
        count: 返す枚数 (1〜16)。
        label: 0〜9 を指定すると、その数字の画像だけを返す。省略するとすべての数字から選ぶ。
        seed: 乱数シード。同じ値なら同じ画像が返る。

    1 件目は {"count", "filter_label", "images": [{"position", "index", "label"}]}
    の JSON で、2 件目以降が position の順に並んだ PNG 画像。
    """
    images, labels = mnist_data.load()
    try:
        samples = mnist_data.sample(images, labels, count, label, seed)
    except ValueError as exc:
        raise ToolError(str(exc)) from exc  # 引数の誤りは Claude にそのまま伝える
    summary = {
        "count": len(samples),
        "filter_label": label,
        "images": [
            {"position": i, "index": s.index, "label": s.label}
            for i, s in enumerate(samples, start=1)
        ],
    }
    pngs = [
        Image(data=mnist_data.to_png(s.pixels, PNG_SCALE), format="png") for s in samples
    ]
    return [json.dumps(summary), *pngs]


if __name__ == "__main__":
    mnist_data.load()  # ダウンロードを最初のツール呼び出しより前に済ませる
    mcp.run()
