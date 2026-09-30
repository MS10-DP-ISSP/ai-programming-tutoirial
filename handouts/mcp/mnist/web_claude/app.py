"""MNIST ビューア (Claude Haiku + MNIST MCP サーバ) の Gradio 画面。

「label 9 のデータを 5 枚ください」→ Haiku が get_mnist_images(label=9, count=5) を呼ぶ
→ 画像をチャットに表示、という流れを画面で追えるようにしている。
起動: uv run python app.py
"""

import io
import json

import gradio as gr
from PIL import Image

from mnist_agent import MODEL, build_options, is_off_topic, use_subscription_auth
from runner import run_turn

# API キーが環境にあっても使わず、claude の /login 済みサブスクリプションで動かす
removed_credentials = use_subscription_auth()

EXAMPLES = [
    "label 9 のデータを 5 枚ください",
    "3 の手書き数字を 8 枚見せて",
    "5 と 7 を 3 枚ずつ",
    "ランダムに 16 枚",
    "今日の天気は？",  # MNIST と関係ない依頼 → エラー表示
]

OFF_TOPIC_ERROR = "MNIST と関係のない依頼には答えられません。見たい数字と枚数を指定してください。"


def caption(image, position: int) -> str:
    if image.label is None:
        return f"#{position}"
    return f"#{position}  label {image.label}  (index {image.index})"


def gallery(images) -> gr.Gallery:
    """MNIST 画像のリストをチャット内に表示するギャラリーにする。"""
    return gr.Gallery(
        [(Image.open(io.BytesIO(img.png)), caption(img, i)) for i, img in enumerate(images, 1)],
        columns=min(len(images), 8),
        height="auto",
        object_fit="contain",
        format="png",  # webp だと線がにじむ
        show_label=False,
    )


def tool_use_message(name: str, args: dict) -> dict:
    call = f"{name.split('__')[-1]}({json.dumps(args, ensure_ascii=False)})"
    return {
        "role": "assistant",
        "content": f"```python\n{call}\n```",
        "metadata": {"title": "🤖 Claude がツールを呼び出し", "status": "pending"},
    }


def tool_result_message(text: str, n_images: int, is_error: bool) -> dict:
    try:
        body = f"```json\n{json.dumps(json.loads(text), ensure_ascii=False, indent=2)}\n```"
    except (json.JSONDecodeError, TypeError):
        body = f"```\n{text}\n```"
    if n_images:
        body += f"\n\nPNG 画像 {n_images} 枚"
    title = "❌ ツールがエラーを返却" if is_error else "🔧 ツールの戻り値"
    return {"role": "assistant", "content": body, "metadata": {"title": title}}


def mark_first_pending_done(history: list) -> None:
    """戻り値が届いたツール呼び出しの表示を「実行中」から「完了」にする (呼び出し順に対応)。"""
    for m in history:
        if m.get("metadata", {}).get("status") == "pending":
            m["metadata"]["status"] = "done"
            return


def render(history: list) -> list:
    """履歴をチャット表示用のメッセージにする。画像のリストはギャラリーに変える。"""
    return [
        {**m, "content": gallery(m["content"])} if isinstance(m["content"], list) else m
        for m in history
    ]


async def respond(prompt: str, history: list, session_id: str | None):
    """1 ターン分を実行し、チャット表示を少しずつ更新する。

    history (gr.State) には画像を PNG のまま持ち、表示のたびに render() でギャラリーにする。
    """
    prompt = prompt.strip()
    if not prompt:
        yield render(history), history, session_id, ""
        return
    history = history + [{"role": "user", "content": prompt}]
    yield render(history), history, session_id, ""

    error = None
    async for event in run_turn(prompt, build_options(resume=session_id)):
        kind = event[0]
        if kind == "tool_use":
            history.append(tool_use_message(event[1], event[2]))
        elif kind == "tool_result":
            _, text, images, is_error = event
            mark_first_pending_done(history)
            history.append(tool_result_message(text, len(images), is_error))
            if images:
                history.append({"role": "assistant", "content": images})
        elif kind == "text":
            if is_off_topic(event[1]):
                error = OFF_TOPIC_ERROR  # 目印は表示せず、エラーにする
            else:
                history.append({"role": "assistant", "content": event[1]})
        elif kind == "done":
            session_id = event[1]["session_id"]
        else:  # error
            error = event[1]
        yield render(history), history, session_id, ""

    if error:
        history.append({"role": "assistant", "content": f"⚠️ {error}"})
    yield render(history), history, session_id, ""


with gr.Blocks(title="MNIST Viewer Claude") as demo:
    session_id = gr.State(None)
    history = gr.State([])

    with gr.Sidebar():
        gr.Markdown("## 設定")
        gr.Markdown(f"モデル: `{MODEL}`\n\nMCP サーバ: `server.py`\n\n使えるツール: `get_mnist_images` のみ")
        if removed_credentials:
            gr.Markdown(
                f"ℹ️ {', '.join(removed_credentials)} を無視して、"
                "Claude のサブスクリプション (claude /login) で実行します。"
            )
        new_chat = gr.Button("新しい会話")

    gr.Markdown(
        "# 🔢 MNIST Viewer (Claude 版)\n"
        "見たい手書き数字と枚数を日本語で伝えると、Claude Haiku が MCP ツール "
        "`get_mnist_images` を呼び、取得した画像を表示します。"
    )
    chatbot = gr.Chatbot(height=600, show_label=False, buttons=["copy"])
    prompt = gr.Textbox(placeholder="例: label 9 のデータを 5 枚ください", show_label=False, submit_btn=True)
    gr.Examples(EXAMPLES, inputs=prompt)

    prompt.submit(respond, [prompt, history, session_id], [chatbot, history, session_id, prompt])
    new_chat.click(lambda: ([], [], None), outputs=[chatbot, history, session_id])


if __name__ == "__main__":
    demo.launch(server_port=7860)
