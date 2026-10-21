"""Guest 子进程固定入口 `src/serve.py`。"""

import sys
from pathlib import Path

_SDK = Path(__file__).resolve().parents[2] / "ability-sdk-py" / "src"
if _SDK.is_dir():
    sys.path.insert(0, str(_SDK))

from leaf_ability_sdk import serve_ability


def ping(ctx, _args):
    ctx.emit("pong", {"from": "probe"})
    return {"ok": True, "from": "external-py"}


if __name__ == "__main__":
    serve_ability({"ping": ping})
