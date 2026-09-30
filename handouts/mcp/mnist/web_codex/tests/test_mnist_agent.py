import sys

from openai_codex import ApprovalMode, Sandbox

from mnist_agent import (
    HERE,
    MODEL,
    OFF_TOPIC_MARKER,
    SERVER_PATH,
    SYSTEM_PROMPT,
    config_overrides,
    is_off_topic,
    thread_options,
    use_subscription_auth,
    user_mcp_servers,
)


def test_server_is_inside_this_directory():
    # web_codex/ だけで完結する (親ディレクトリの Julia サーバは使わない)
    assert SERVER_PATH == HERE / "server.py"
    assert SERVER_PATH.exists()


def test_mnist_server_runs_with_this_python_and_is_preapproved(tmp_path):
    overrides = config_overrides(tmp_path / "missing.toml")
    assert f'mcp_servers.mnist.command="{sys.executable}"' in overrides
    assert f'mcp_servers.mnist.args=["{SERVER_PATH}"]' in overrides
    assert 'mcp_servers.mnist.default_tools_approval_mode="approve"' in overrides


def test_user_mcp_servers_are_disabled(tmp_path):
    config = tmp_path / "config.toml"
    config.write_text(
        'model = "x"\n[mcp_servers.node_repl]\ncommand = "node"\n'
        '[mcp_servers.mnist]\ncommand = "julia"\n'
    )
    assert user_mcp_servers(config) == ["node_repl", "mnist"]
    overrides = config_overrides(config)
    assert "mcp_servers.node_repl.enabled=false" in overrides
    # 同じ名前のサーバは無効にせず、こちらの設定で上書きする
    assert "mcp_servers.mnist.enabled=false" not in overrides


def test_shell_hooks_and_other_tools_are_disabled(tmp_path):
    overrides = config_overrides(tmp_path / "missing.toml")
    for feature in ["shell_tool", "unified_exec", "hooks", "apps", "browser_use", "image_generation"]:
        assert f"features.{feature}=false" in overrides
    assert 'web_search="disabled"' in overrides


def test_thread_options():
    options = thread_options()
    assert options["model"] == MODEL
    assert options["developer_instructions"] == SYSTEM_PROMPT
    assert options["sandbox"] == Sandbox.read_only
    assert options["approval_mode"] == ApprovalMode.deny_all


def test_off_topic_marker():
    assert OFF_TOPIC_MARKER in SYSTEM_PROMPT
    assert is_off_topic(" OFF_TOPIC\n")
    assert not is_off_topic("9 の画像を 5 枚取得しました。")


def test_use_subscription_auth_removes_api_credentials():
    env = {"OPENAI_API_KEY": "x", "CODEX_API_KEY": "y", "HOME": "/h"}
    assert use_subscription_auth(env) == ["OPENAI_API_KEY", "CODEX_API_KEY"]
    assert env == {"HOME": "/h"}
