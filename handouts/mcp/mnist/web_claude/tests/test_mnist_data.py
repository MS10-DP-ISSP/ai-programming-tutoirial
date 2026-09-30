import io

import numpy as np
import pytest
from PIL import Image

import mnist_data


def _fake_dataset():
    # 各数字 3 枚ずつ、画素値 = ラベル の 28x28 画像
    labels = np.repeat(np.arange(10, dtype=np.uint8), 3)
    images = np.broadcast_to(labels[:, None, None], (30, 28, 28)).copy()
    return images, labels


def test_parse_idx_reads_images_and_labels():
    header = bytes([0, 0, 0x08, 3]) + (2).to_bytes(4, "big") + (28).to_bytes(4, "big") * 2
    body = np.arange(2 * 28 * 28, dtype=np.uint32).astype(np.uint8).tobytes()
    arr = mnist_data.parse_idx(header + body)
    assert arr.shape == (2, 28, 28)
    assert arr[0, 0, 1] == 1  # 行優先: arr[n, y, x]

    labels = mnist_data.parse_idx(bytes([0, 0, 0x08, 1]) + (3).to_bytes(4, "big") + b"\x07\x02\x09")
    assert labels.tolist() == [7, 2, 9]


def test_parse_idx_rejects_non_uint8():
    with pytest.raises(ValueError):
        mnist_data.parse_idx(bytes([0, 0, 0x0D, 1]) + (1).to_bytes(4, "big") + b"\x00" * 4)


def test_sample_filters_by_label_without_duplicates():
    images, labels = _fake_dataset()
    samples = mnist_data.sample(images, labels, count=3, label=9, seed=0)
    assert [s.label for s in samples] == [9, 9, 9]
    assert len({s.index for s in samples}) == 3
    assert all((s.pixels == 9).all() for s in samples)


def test_sample_is_reproducible_with_seed():
    images, labels = _fake_dataset()
    a = mnist_data.sample(images, labels, count=5, seed=42)
    b = mnist_data.sample(images, labels, count=5, seed=42)
    assert [s.index for s in a] == [s.index for s in b]


@pytest.mark.parametrize("count,label", [(0, None), (17, None), (1, 10), (1, -1)])
def test_sample_rejects_out_of_range(count, label):
    images, labels = _fake_dataset()
    with pytest.raises(ValueError):
        mnist_data.sample(images, labels, count=count, label=label)


def test_to_png_scales_without_blurring():
    pixels = np.zeros((28, 28), dtype=np.uint8)
    pixels[0, 0] = 255
    img = Image.open(io.BytesIO(mnist_data.to_png(pixels, scale=4)))
    assert img.size == (112, 112)
    arr = np.asarray(img)
    assert (arr[:4, :4] == 255).all() and arr[4, 4] == 0


def test_download_tries_gcs_mirror_first(monkeypatch, tmp_path):
    # S3 ミラーは遅い (約 130 KB/s) ので、速い GCS を先に試す
    requested = []

    class FakeResponse:
        def __enter__(self):
            return self

        def __exit__(self, *exc):
            return False

        def read(self):
            return b"data"

    def fake_urlopen(url, timeout):
        requested.append(url)
        return FakeResponse()

    monkeypatch.setattr(mnist_data.urllib.request, "urlopen", fake_urlopen)

    mnist_data.download("train-labels-idx1-ubyte.gz", tmp_path)

    assert requested == ["https://storage.googleapis.com/cvdf-datasets/mnist/train-labels-idx1-ubyte.gz"]
