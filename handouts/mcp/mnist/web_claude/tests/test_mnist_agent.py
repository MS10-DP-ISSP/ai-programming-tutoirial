import sys
from pathlib import Path

from mnist_agent import (
    MODEL,
    OFF_TOPIC_MARKER,
    SERVER_PATH,
    TOOL_NAME,
    build_options,
    is_off_topic,
    use_subscription_auth,
)


def test_uses_haiku():
    assert MODEL == "claude-haiku-4-5"
    assert build_options().model == "claude-haiku-4-5"


def test_server_is_inside_this_directory():
    # web_claude/ だけで完結する (親ディレクトリの Julia サーバは使わない)
    assert SERVER_PATH == Path(__file__).resolve().parents[1] / "server.py"
    assert SERVER_PATH.exists()


def test_mnist_mcp_server_runs_with_this_python():
    options = build_options()
    assert list(options.mcp_servers) == ["mnist"]
    assert options.mcp_servers["mnist"] == {"command": sys.executable, "args": [str(SERVER_PATH)]}
    assert options.strict_mcp_config is True


def test_only_get_mnist_images_is_allowed():
    options = build_options()
    assert options.allowed_tools == [TOOL_NAME] == ["mcp__mnist__get_mnist_images"]
    for banned in ["Bash", "Write", "Edit", "Read", "WebFetch", "WebSearch", "ToolSearch"]:
        assert banned in options.disallowed_tools


def test_does_not_load_user_or_project_settings():
    assert build_options().setting_sources == []


def test_resume_is_passed_through():
    assert build_options().resume is None
    assert build_options(resume="sess-1").resume == "sess-1"


def test_off_topic_marker():
    assert OFF_TOPIC_MARKER in build_options().system_prompt
    assert is_off_topic(" OFF_TOPIC\n")
    assert not is_off_topic("9 の画像を 5 枚取得しました。")


def test_use_subscription_auth_removes_api_credentials():
    env = {"ANTHROPIC_API_KEY": "x", "ANTHROPIC_AUTH_TOKEN": "y", "HOME": "/h"}
    assert use_subscription_auth(env) == ["ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN"]
    assert env == {"HOME": "/h"}
