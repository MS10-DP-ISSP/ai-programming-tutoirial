import download_data
import mnist_data


def test_main_downloads_images_and_labels(monkeypatch, capsys):
    requested = []
    monkeypatch.setattr(mnist_data, "download", lambda name: requested.append(name))

    download_data.main()

    assert requested == [mnist_data.IMAGES_FILE, mnist_data.LABELS_FILE]
    assert mnist_data.IMAGES_FILE in capsys.readouterr().out
