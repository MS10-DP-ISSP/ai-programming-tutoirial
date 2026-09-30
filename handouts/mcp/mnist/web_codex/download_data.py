"""MNIST を data/ にダウンロードする (app.py を起動する前に一度だけ実行する)。

起動: uv run python download_data.py
既にあるファイルは再ダウンロードしない。
"""

import mnist_data


def main() -> None:
    for name in (mnist_data.IMAGES_FILE, mnist_data.LABELS_FILE):
        print(f"{name} を確認しています…", flush=True)
        path = mnist_data.download(name)
        print(f"  OK: {path}", flush=True)


if __name__ == "__main__":
    main()
