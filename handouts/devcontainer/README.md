# Devcontainer per harness

コーディングエージェントを `bypass permissions` 系のモードで動かすと、`rm -rf /` のような破壊的コマンドが実行されるリスクがあります。
Dev Container の中で動かし、必要なコードとデータだけをマウントすることで被害範囲を限定できます。

各ディレクトリに、ハーネスごとの最小構成の `.devcontainer/devcontainer.json` があります。

| Directory   | Harness     | コンテナ内でのログイン |
|-------------|-------------|------------------------|
| `claude/`   | Claude Code | `claude`               |
| `codex/`    | Codex CLI   | `codex login`          |
| `opencode/` | opencode    | `opencode auth login`  |

## 事前準備

- Docker (Docker Desktop など) が起動していること
- Node.js が入っていること

```sh
$ npm install -g @devcontainers/cli
$ devcontainer --version   # インストール確認
```

## 使い方

使いたいハーネスのディレクトリに移動して実行します (以下は `claude/` の例)。

```sh
$ cd claude                                  # または codex / opencode
$ devcontainer build --workspace-folder .    # 任意: イメージだけ先に作る
$ devcontainer up --workspace-folder .       # コンテナを起動
$ devcontainer exec --workspace-folder . bash
vscode ➜ /workspaces/... $                   # コンテナ内のシェル
```

コンテナ内でハーネスを起動してログインします。

```sh
$ claude              # Claude Code
$ codex login         # Codex CLI
$ opencode auth login # opencode
```

ログイン情報は名前付き Docker ボリュームに保存されるため、コンテナを作り直しても維持されます。

| Directory   | ボリューム                                    | マウント先                                                    |
|-------------|-----------------------------------------------|---------------------------------------------------------------|
| `claude/`   | `claude-code-config`                          | `~/.claude`                                                   |
| `codex/`    | `codex-config`                                | `~/.codex`                                                    |
| `opencode/` | `opencode-config`, `opencode-data`            | `~/.config/opencode`, `~/.local/share/opencode`               |

## 動作確認

`up` が成功したら、コンテナ内でバージョンが表示されることを確認します。

```sh
$ devcontainer exec --workspace-folder . claude --version     # claude/
$ devcontainer exec --workspace-folder . codex --version      # codex/
$ devcontainer exec --workspace-folder . opencode --version   # opencode/
```

`codex/` と `opencode/` は `postCreateCommand` で npm から導入しており、バージョンを固定しています
(`@openai/codex@0.159.2`, `opencode-ai@1.18.33`)。更新するときは各 `devcontainer.json` の値を書き換えてください。

## 再構築

`devcontainer.json` を変更したら、コンテナを作り直します。

```sh
$ devcontainer up --workspace-folder . --remove-existing-container
```

## 後片付け

作成したコンテナ、イメージ、ボリュームを削除します。ボリュームを消すとログイン情報も消えます。

```sh
# 対象のコンテナを探す
$ docker ps -a --filter label=devcontainer.local_folder \
    --format '{{.ID}} {{.Label "devcontainer.local_folder"}}'

$ docker rm -f <container-id>
$ docker rmi <image-name>          # build で --image-name を付けた場合
$ docker volume rm codex-config    # 例: codex/ のログイン情報を消す
```

`claude-code-config` は Claude Code のログイン情報を保存するボリュームです。他の Dev Container でも使っていないか確認してから削除してください。

## トラブルシューティング

### `postCreateCommand` が `npm: command not found` で失敗する

Node は nvm 経由で入るため、`sudo` の PATH に `npm` がありません。`sudo npm install -g ...` とせず、
`npm install -g ...` と書いてください (グローバルの `node_modules` は `vscode` ユーザーが書き込めます)。

### `~/.claude` などに書き込めない

名前付きボリュームは root 所有で作られることがあります。`postCreateCommand` の `sudo chown -R vscode:vscode ...` が
その対策です。マウント先を変えたときは `chown` の対象も合わせてください。

### `devcontainer build` は成功したのに `up` で失敗する

`build` はイメージと Feature の組み込みまでで、`postCreateCommand` は `up` のときに実行されます。
`postCreateCommand` の不具合は `up` で初めて分かります。
