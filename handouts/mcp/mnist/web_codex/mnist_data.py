"""MNIST (訓練データ 60,000 枚) の読み込みとサンプリング。

初回だけ公開ミラーから IDX 形式のファイルを data/ にダウンロードする。
画像は (28, 28) の uint8 配列で、arr[y, x] (行優先) で画素値 0-255 を持つ。
"""

import gzip
import io
import urllib.request
from dataclasses import dataclass
from functools import cache
from pathlib import Path

import numpy as np
from PIL import Image

DATA_DIR = Path(__file__).resolve().parent / "data"

# 先頭から順に試す。S3 は遅いことがある (約 130 KB/s) ので GCS を先にする
MIRRORS = [
    "https://storage.googleapis.com/cvdf-datasets/mnist/",
    "https://ossci-datasets.s3.amazonaws.com/mnist/",
]
IMAGES_FILE = "train-images-idx3-ubyte.gz"
LABELS_FILE = "train-labels-idx1-ubyte.gz"

MAX_COUNT = 16


@dataclass(frozen=True)
class Sample:
    index: int  # データセット内の通し番号 (0 始まり)
    label: int
    pixels: np.ndarray  # (28, 28) uint8


def download(name: str, data_dir: Path = DATA_DIR) -> Path:
    """name を data_dir にダウンロードする (既にあれば何もしない)。"""
    path = data_dir / name
    if path.exists():
        return path
    data_dir.mkdir(parents=True, exist_ok=True)
    errors = []
    for mirror in MIRRORS:
        try:
            with urllib.request.urlopen(mirror + name, timeout=60) as response:
                body = response.read()
        except OSError as exc:
            errors.append(f"{mirror}: {exc}")
            continue
        tmp = path.with_suffix(".part")
        tmp.write_bytes(body)
        tmp.rename(path)
        return path
    raise RuntimeError(f"MNIST の {name} をダウンロードできません: {errors}")


def parse_idx(raw: bytes) -> np.ndarray:
    """IDX 形式 (先頭 4 バイトが magic、続いて各次元の大きさ) を numpy 配列にする。"""
    if raw[:2] != b"\x00\x00" or raw[2] != 0x08:
        raise ValueError("uint8 の IDX ファイルではありません")
    ndim = raw[3]
    shape = tuple(int.from_bytes(raw[4 + 4 * i : 8 + 4 * i], "big") for i in range(ndim))
    return np.frombuffer(raw, dtype=np.uint8, offset=4 + 4 * ndim).reshape(shape)


@cache
def load() -> tuple[np.ndarray, np.ndarray]:
    """(images (60000, 28, 28), labels (60000,)) を返す。"""
    images = parse_idx(gzip.decompress(download(IMAGES_FILE).read_bytes()))
    labels = parse_idx(gzip.decompress(download(LABELS_FILE).read_bytes()))
    return images, labels


def validate(count: int, label: int | None) -> None:
    if not 1 <= count <= MAX_COUNT:
        raise ValueError(f"count は 1〜{MAX_COUNT} で指定してください (count={count})")
    if label is not None and not 0 <= label <= 9:
        raise ValueError(f"label は 0〜9 で指定してください (label={label})")


def sample(
    images: np.ndarray,
    labels: np.ndarray,
    count: int,
    label: int | None = None,
    seed: int | None = None,
) -> list[Sample]:
    """count 枚を重複なしで選ぶ。label を指定するとその数字だけから選ぶ。"""
    validate(count, label)
    pool = np.arange(len(labels)) if label is None else np.flatnonzero(labels == label)
    rng = np.random.default_rng(seed)
    picked = rng.choice(pool, size=min(count, len(pool)), replace=False)
    return [Sample(int(i), int(labels[i]), images[i]) for i in picked]


def to_png(pixels: np.ndarray, scale: int = 1) -> bytes:
    """28x28 の画素を PNG にする。scale 倍に拡大 (ぼかさない)。"""
    img = Image.fromarray(pixels)
    if scale != 1:
        img = img.resize((img.width * scale, img.height * scale), Image.NEAREST)
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    return buf.getvalue()
