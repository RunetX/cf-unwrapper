#!/usr/bin/env python3
"""Exercise Git's actual clean/smudge filters without changing the real index."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import shutil
import uuid

ROOT = Path(__file__).resolve().parents[1]


def check():
    inventory = json.loads((ROOT / "docs/source-inventory.json").read_text(encoding="utf-8"))
    assets = [entry for entry in inventory["files"] if not entry["path"].endswith(".rs")]
    (ROOT / "target").mkdir(exist_ok=True)
    base = ROOT / "target" / f"git-assets-{uuid.uuid4().hex}"
    base.mkdir()
    try:
        env = dict(os.environ, GIT_INDEX_FILE=str(base / "index"))

        def git(*args):
            subprocess.run(["git", *args], cwd=ROOT, env=env, check=True,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE)

        for clean in ["true", "input", "false"]:
            git("read-tree", "--empty")
            git("-c", f"core.autocrlf={clean}", "add", "--", *[a["path"] for a in assets])
            for eol in ["lf", "crlf"]:
                dest = base / f"{clean}-{eol}"
                dest.mkdir()
                git("-c", "core.autocrlf=false", "-c", f"core.eol={eol}",
                    "checkout-index", "--all", f"--prefix={dest.as_posix()}/")
                for entry in assets:
                    actual = hashlib.sha256((dest / entry["path"]).read_bytes()).hexdigest()
                    assert actual == entry["sha256"], (
                        f"Git changed embedded bytes: {entry['path']} "
                        f"(add autocrlf={clean}, checkout eol={eol})")
    finally:
        shutil.rmtree(base)
    print(f"git assets: {len(assets)} exact payloads survive 6 clean/checkout filter combinations")


if __name__ == "__main__":
    check()
