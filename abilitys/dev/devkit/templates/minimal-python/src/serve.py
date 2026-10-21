import sys
from pathlib import Path

for _anc in Path(__file__).resolve().parents:
    _sdk = _anc / "SDK" / "ability-sdk-py" / "src"
    if _sdk.is_dir():
        sys.path.insert(0, str(_sdk))
        break

from leaf_ability_sdk import serve_ability


def run(_ctx, _args):
    return {"ok": True}


if __name__ == "__main__":
    serve_ability({"run": run})
